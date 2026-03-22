use std::ops::ControlFlow;

use futures_util::{SinkExt, Stream, StreamExt};
use iced::futures::channel::mpsc;
use iced::stream;
use matrix_sdk::Room;
use matrix_sdk::deserialized_responses::TimelineEventKind;
use matrix_sdk::room::{Messages, MessagesOptions};
use matrix_sdk::ruma::events::AnySyncTimelineEvent;
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::{OwnedEventId, OwnedRoomId, OwnedUserId, assign};
use tracing::{error, info, warn};

use crate::worker::WorkerSubscription;

pub type Worker = super::Worker<Request>;

impl WorkerSubscription<Response> for Worker {
    fn subscription(mut self) -> impl Stream<Item = Response> {
        stream::channel(64, async move |mut s: mpsc::Sender<Response>| {
            info!("created messages worker");
            let observer = self
                .client
                .observe_events::<Raw<AnySyncTimelineEvent>, Room>();
            let mut stream = observer.subscribe();

            let mut last_event_id = None;
            loop {
                tokio::select! {
                    req = self.request_stream.recv() => {
                        if handle_request(&self.client, &mut s, req).await == ControlFlow::Break(()) {
                            return;
                        }
                    }
                    msg = stream.next() => {
                        let Some((msg, room)) = msg else {
                            return;
                        };

                        info!(?msg);
                        let Some(content) = handle_message(&mut last_event_id, &msg) else {
                            continue;
                        };

                        s.send(Response::NewMessage(room.room_id().to_owned(), content)).await.ok();
                    }
                };
            }
        })
    }
}

async fn handle_request(
    client: &matrix_sdk::Client,
    s: &mut mpsc::Sender<Response>,
    req: Result<Request, tokio::sync::broadcast::error::RecvError>,
) -> ControlFlow<()> {
    match req {
        Ok(Request::LatestMessages(room_id)) => {
            if let Some(response) = retrieve_latest_messages(client, room_id).await {
                s.send(response).await.ok();
            }
        }
        Err(e) => {
            error!(error = ?e);
            return ControlFlow::Break(());
        }
    }
    ControlFlow::Continue(())
}

async fn retrieve_latest_messages(
    client: &matrix_sdk::Client,
    room_id: OwnedRoomId,
) -> Option<Response> {
    let Some(room) = client.get_room(&room_id) else {
        warn!(?room_id, "room not found");
        return None;
    };
    let mut rendered = Vec::new();
    let options = assign!(MessagesOptions::backward(), {
        limit: 50u32.into(),
    });
    match room.messages(options).await {
        Ok(messages) => {
            handle_messages(&mut rendered, &messages);
        }
        Err(e) => {
            error!(error = ?e, "failed to retrieve messages");
            return None;
        }
    }

    Some(Response::Messages(room_id, rendered))
}

fn handle_messages(rendered: &mut Vec<MessageContent>, messages: &Messages) {
    for message in messages.chunk.iter().rev() {
        match &message.kind {
            TimelineEventKind::Decrypted(room_event) => {
                if let Some(msg) = handle_message(&mut None, room_event.event.cast_ref()) {
                    rendered.push(msg);
                }
            }
            TimelineEventKind::UnableToDecrypt { utd_info, .. } => {
                rendered.push(MessageContent {
                    sender: None,
                    message_content: format!("{:?}", utd_info.reason),
                });
            }
            TimelineEventKind::PlainText { event } => {
                if let Some(msg) = handle_message(&mut None, event) {
                    rendered.push(msg);
                }
            }
        }
    }
}

fn handle_message(
    last_event_id: &mut Option<OwnedEventId>,
    raw: &Raw<AnySyncTimelineEvent>,
) -> Option<MessageContent> {
    let Ok(msg) = raw.deserialize() else {
        return None;
    };

    let eid = Some(msg.event_id().to_owned());
    if *last_event_id == eid {
        return None;
    }
    *last_event_id = eid;

    match msg {
        AnySyncTimelineEvent::MessageLike(any_message_like_event) => match any_message_like_event {
            matrix_sdk::ruma::events::AnySyncMessageLikeEvent::RoomEncrypted(ev) => {
                Some(MessageContent {
                    sender: Some(ev.sender().to_owned()),
                    message_content: "encrypted message".to_owned(),
                })
            }
            matrix_sdk::ruma::events::AnySyncMessageLikeEvent::RoomMessage(ev) => {
                ev.as_original().map(|og| MessageContent {
                    sender: Some(ev.sender().to_owned()),
                    message_content: og.content.body().to_owned(),
                })
            }
            matrix_sdk::ruma::events::AnySyncMessageLikeEvent::RoomRedaction(ev) => {
                Some(MessageContent {
                    sender: Some(ev.sender().to_owned()),
                    message_content: "redacted event".to_owned(),
                })
            }
            ev => Some(MessageContent {
                sender: Some(ev.sender().to_owned()),
                message_content: format!("unhandled event type {}", ev.event_type()),
            }),
        },
        AnySyncTimelineEvent::State(ev) => Some(MessageContent {
            sender: None,
            message_content: format!("unhandled state event type {}", ev.event_type()),
        }),
    }
}

#[derive(Debug, Clone)]
pub struct MessageContent {
    pub sender: Option<OwnedUserId>,
    pub message_content: String,
}

#[derive(Debug, Clone)]
pub enum Response {
    Messages(OwnedRoomId, Vec<MessageContent>),
    NewMessage(OwnedRoomId, MessageContent),
}

#[derive(Debug, Clone)]
pub enum Request {
    LatestMessages(OwnedRoomId),
}
