use std::ops::ControlFlow;

use futures_util::{SinkExt, Stream, StreamExt};
use iced::futures::channel::mpsc;
use iced::stream;
use matrix_sdk::Room;
use matrix_sdk::deserialized_responses::TimelineEventKind;
use matrix_sdk::room::{Messages, MessagesOptions};
use matrix_sdk::ruma::events::room::member::MembershipChange;
use matrix_sdk::ruma::events::{AnySyncMessageLikeEvent, AnySyncStateEvent, AnySyncTimelineEvent};
use matrix_sdk::ruma::serde::Raw;
use matrix_sdk::ruma::{OwnedEventId, OwnedRoomId, OwnedUserId, assign};
use tracing::{error, info, warn};

use crate::worker::WorkerSubscription;

pub type Worker = super::Worker<Request>;

impl WorkerSubscription<Response> for Worker {
    fn subscription(mut self) -> impl Stream<Item = Response> {
        stream::channel(64, async move |mut s: mpsc::Sender<Response>| {
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
                rendered.push(state_message(format!(
                    "unable to decrypt:\n{:?}",
                    utd_info.reason
                )));
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
        AnySyncTimelineEvent::MessageLike(ev) => render_message_event(ev),
        AnySyncTimelineEvent::State(ev) => render_state_event(ev),
    }
}

fn render_message_event(ev: AnySyncMessageLikeEvent) -> Option<MessageContent> {
    match ev {
        AnySyncMessageLikeEvent::RoomEncrypted(ev) => Some(state_message(format!(
            "unable to decrypt message from {}",
            ev.sender()
        ))),
        AnySyncMessageLikeEvent::RoomMessage(ev) => ev.as_original().map(|og| MessageContent {
            display_name: Some(ev.sender().to_string()),
            sender: Some(ev.sender().to_owned()),
            content: og.content.body().to_owned(),
        }),
        AnySyncMessageLikeEvent::RoomRedaction(ev) => ev.as_original().map(|ev| {
            state_message(ev.content.redacts.as_ref().map_or_else(
                || "unknown event redacted".to_owned(),
                |ev_id| format!("redacting event {ev_id}"),
            ))
        }),
        ev => Some(state_message(format!(
            "//// unhandled event type {} from {}",
            ev.event_type(),
            ev.sender()
        ))),
    }
}

fn render_state_event(ev: AnySyncStateEvent) -> Option<MessageContent> {
    match ev {
        AnySyncStateEvent::RoomAliases(_ev) => Some("room aliases have changed".to_owned()),
        AnySyncStateEvent::RoomAvatar(_ev) => Some("room avatar has changed".to_owned()),
        AnySyncStateEvent::RoomCanonicalAlias(_ev) => {
            Some("room canonical alias has changed".to_owned())
        }
        AnySyncStateEvent::RoomCreate(_ev) => Some("room has been created".to_owned()),
        AnySyncStateEvent::RoomEncryption(_ev) => Some("room encryption has changed".to_owned()),
        AnySyncStateEvent::RoomGuestAccess(_ev) => Some("room guest access has changed".to_owned()),
        AnySyncStateEvent::RoomHistoryVisibility(_ev) => {
            Some("room history visibility has changed".to_owned())
        }
        AnySyncStateEvent::RoomJoinRules(_ev) => Some("room join rules have changed".to_owned()),
        AnySyncStateEvent::RoomMember(ev) => {
            ev.as_original().map(|ev| match ev.membership_change() {
                MembershipChange::None => format!("{} is unchanged", ev.sender),
                MembershipChange::Error => {
                    format!("{} had a malformed membership change", ev.sender)
                }
                MembershipChange::Joined => format!("{} joined", ev.sender),
                MembershipChange::Left => format!("{} left", ev.sender),
                MembershipChange::Banned => format!("{} was banned", ev.sender),
                MembershipChange::Unbanned => format!("{} was unbanned", ev.sender),
                MembershipChange::Kicked => format!("{} was kicked", ev.sender),
                MembershipChange::Invited => format!("{} was invited", ev.sender),
                MembershipChange::KickedAndBanned => {
                    format!("{} was kicked and banned", ev.sender)
                }
                MembershipChange::InvitationAccepted => {
                    format!("{} accepted invitation", ev.sender)
                }
                MembershipChange::InvitationRejected => {
                    format!("{} rejected invitation", ev.sender)
                }
                MembershipChange::InvitationRevoked => {
                    format!("{} had their invitation revoked", ev.sender)
                }
                MembershipChange::Knocked => format!("{} knocked", ev.sender),
                MembershipChange::KnockAccepted => format!("{} knock was accepted", ev.sender),
                MembershipChange::KnockRetracted => format!("{} knock was retracted", ev.sender),
                MembershipChange::KnockDenied => format!("{} knock was denied", ev.sender),
                MembershipChange::ProfileChanged {
                    displayname_change,
                    avatar_url_change,
                } => match (displayname_change, avatar_url_change) {
                    (Some(_), Some(_)) => {
                        format!("{} display name and avatar changed", ev.sender)
                    }
                    (Some(_), None) => format!("{} display name changed", ev.sender),
                    (None, Some(_)) => format!("{} avatar changed", ev.sender),
                    (None, None) => format!("{} had no change", ev.sender),
                },
                ev => {
                    info!(?ev);
                    "//// unhandled membership change event".to_owned()
                }
            })
        }
        AnySyncStateEvent::RoomName(ev) => ev
            .as_original()
            .map(|ev| format!("room name has been changed to \"{}\"", ev.content.name)),
        AnySyncStateEvent::RoomPinnedEvents(_ev) => {
            Some("room pinned events have been changed".to_owned())
        }
        AnySyncStateEvent::RoomPowerLevels(_ev) => {
            Some("room power levels have been changed".to_owned())
        }
        AnySyncStateEvent::RoomServerAcl(_ev) => Some("room acl has been changed".to_owned()),
        AnySyncStateEvent::RoomTombstone(ev) => ev.as_original().map(|ev| {
            format!(
                "room has been tombstoned and is moving to {}",
                ev.content.replacement_room,
            )
        }),
        AnySyncStateEvent::RoomTopic(ev) => ev
            .as_original()
            .map(|ev| format!("room topic was changed to \"{}\"", ev.content.topic)),
        ev => Some(format!(
            "//// unhandled state event type {}",
            ev.event_type()
        )),
    }
    .map(state_message)
}

fn state_message(content: impl Into<String>) -> MessageContent {
    MessageContent {
        display_name: None,
        sender: None,
        content: content.into(),
    }
}

#[derive(Debug, Clone)]
pub struct MessageContent {
    pub display_name: Option<String>,
    pub sender: Option<OwnedUserId>,
    pub content: String,
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
