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
        AnySyncTimelineEvent::MessageLike(ev) => render_message_event(ev),
        AnySyncTimelineEvent::State(ev) => render_state_event(ev),
    }
}

fn render_message_event(ev: AnySyncMessageLikeEvent) -> Option<MessageContent> {
    match ev {
        AnySyncMessageLikeEvent::RoomEncrypted(ev) => Some(MessageContent {
            sender: Some(ev.sender().to_owned()),
            message_content: "encrypted message".to_owned(),
        }),
        AnySyncMessageLikeEvent::RoomMessage(ev) => ev.as_original().map(|og| MessageContent {
            sender: Some(ev.sender().to_owned()),
            message_content: og.content.body().to_owned(),
        }),
        AnySyncMessageLikeEvent::RoomRedaction(ev) => ev.as_original().map(|ev| MessageContent {
            sender: None,
            message_content: ev.content.redacts.as_ref().map_or_else(
                || "unknown event redacted".to_owned(),
                |ev_id| format!("redacting event {ev_id}"),
            ),
        }),
        ev => Some(MessageContent {
            sender: None,
            message_content: format!(
                "//// unhandled event type {} from {}",
                ev.event_type(),
                ev.sender()
            ),
        }),
    }
}

fn render_state_event(ev: AnySyncStateEvent) -> Option<MessageContent> {
    match ev {
        AnySyncStateEvent::RoomAliases(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room aliases have changed".to_owned(),
        }),
        AnySyncStateEvent::RoomAvatar(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room avatar has changed".to_owned(),
        }),
        AnySyncStateEvent::RoomCanonicalAlias(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room canonical alias has changed".to_owned(),
        }),
        AnySyncStateEvent::RoomCreate(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room has been created".to_owned(),
        }),
        AnySyncStateEvent::RoomEncryption(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room encryption has changed".to_owned(),
        }),
        AnySyncStateEvent::RoomGuestAccess(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room guest access has changed".to_owned(),
        }),
        AnySyncStateEvent::RoomHistoryVisibility(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room history visibility has changed".to_owned(),
        }),
        AnySyncStateEvent::RoomJoinRules(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room join rules have changed".to_owned(),
        }),
        AnySyncStateEvent::RoomMember(ev) => {
            ev.as_original().map(|ev| match ev.membership_change() {
                MembershipChange::None => MessageContent {
                    sender: None,
                    message_content: format!("{} is unchanged", ev.sender),
                },
                MembershipChange::Error => MessageContent {
                    sender: None,
                    message_content: format!("{} had a malformed membership change", ev.sender),
                },
                MembershipChange::Joined => MessageContent {
                    sender: None,
                    message_content: format!("{} joined", ev.sender),
                },
                MembershipChange::Left => MessageContent {
                    sender: None,
                    message_content: format!("{} left", ev.sender),
                },
                MembershipChange::Banned => MessageContent {
                    sender: None,
                    message_content: format!("{} was banned", ev.sender),
                },
                MembershipChange::Unbanned => MessageContent {
                    sender: None,
                    message_content: format!("{} was unbanned", ev.sender),
                },
                MembershipChange::Kicked => MessageContent {
                    sender: None,
                    message_content: format!("{} was kicked", ev.sender),
                },
                MembershipChange::Invited => MessageContent {
                    sender: None,
                    message_content: format!("{} was invited", ev.sender),
                },
                MembershipChange::KickedAndBanned => MessageContent {
                    sender: None,
                    message_content: format!("{} was kicked and banned", ev.sender),
                },
                MembershipChange::InvitationAccepted => MessageContent {
                    sender: None,
                    message_content: format!("{} accepted invitation", ev.sender),
                },
                MembershipChange::InvitationRejected => MessageContent {
                    sender: None,
                    message_content: format!("{} rejected invitation", ev.sender),
                },
                MembershipChange::InvitationRevoked => MessageContent {
                    sender: None,
                    message_content: format!("{} had their invitation revoked", ev.sender),
                },
                MembershipChange::Knocked => MessageContent {
                    sender: None,
                    message_content: format!("{} knocked", ev.sender),
                },
                MembershipChange::KnockAccepted => MessageContent {
                    sender: None,
                    message_content: format!("{} knock was accepted", ev.sender),
                },
                MembershipChange::KnockRetracted => MessageContent {
                    sender: None,
                    message_content: format!("{} knock was retracted", ev.sender),
                },
                MembershipChange::KnockDenied => MessageContent {
                    sender: None,
                    message_content: format!("{} knock was denied", ev.sender),
                },
                MembershipChange::ProfileChanged {
                    displayname_change,
                    avatar_url_change,
                } => MessageContent {
                    sender: None,
                    message_content: match (displayname_change, avatar_url_change) {
                        (Some(_), Some(_)) => {
                            format!("{} display name and avatar changed", ev.sender)
                        }
                        (Some(_), None) => format!("{} display name changed", ev.sender),
                        (None, Some(_)) => format!("{} avatar changed", ev.sender),
                        (None, None) => format!("{} had no change", ev.sender),
                    },
                },
                ev => {
                    info!(?ev);
                    MessageContent {
                        sender: None,
                        message_content: "//// unhandled membership change event".to_owned(),
                    }
                }
            })
        }
        AnySyncStateEvent::RoomName(ev) => ev.as_original().map(|ev| MessageContent {
            sender: None,
            message_content: format!("room name has been changed to \"{}\"", ev.content.name),
        }),
        AnySyncStateEvent::RoomPinnedEvents(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room pinned events have been changed".to_owned(),
        }),
        AnySyncStateEvent::RoomPowerLevels(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room power levels have been changed".to_owned(),
        }),
        AnySyncStateEvent::RoomServerAcl(_ev) => Some(MessageContent {
            sender: None,
            message_content: "room acl has been changed".to_owned(),
        }),
        AnySyncStateEvent::RoomTombstone(ev) => ev.as_original().map(|ev| MessageContent {
            sender: None,
            message_content: format!(
                "room has been tombstoned and is moving to {}",
                ev.content.replacement_room,
            ),
        }),
        AnySyncStateEvent::RoomTopic(ev) => ev.as_original().map(|ev| MessageContent {
            sender: None,
            message_content: format!("room topic was changed to \"{}\"", ev.content.topic),
        }),
        ev => Some(MessageContent {
            sender: None,
            message_content: format!("//// unhandled state event type {}", ev.event_type()),
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
