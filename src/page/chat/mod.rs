mod components;

use std::collections::HashMap;

use iced::widget::{Row, image};
use iced::{Element, Subscription, Task};
use matrix_sdk::ruma::OwnedRoomId;

use crate::app::ViewLike;
use crate::page::PageMessage;
use crate::tasks::get_space_rooms;
use crate::util::Smuggle;
use crate::worker::{self, WorkerSubscription, messages, sliding_sync, verification};

crate::msg_adapter_impl!(Message, PageMessage, Chat);

#[derive(Debug, Clone)]
pub enum Message {
    OpenSpace(OwnedRoomId),
    OpenRoom(OwnedRoomId),
    UpdateMessage(String),

    SyncUpdate(sliding_sync::Response),
    VerificationUpdate(verification::Response),
    MessagesUpdate(messages::Response),

    RoomSynced(OwnedRoomId, bool),
    RoomAvatar(OwnedRoomId, image::Handle),

    AddSpaceRoom {
        space: OwnedRoomId,
        room: OwnedRoomId,
    },
}

#[derive(Debug, Clone)]
pub struct Page {
    client: matrix_sdk::Client,

    avatars: HashMap<OwnedRoomId, image::Handle>,
    current_space: Option<OwnedRoomId>,
    current_room: Option<OwnedRoomId>,

    verif_worker: verification::Worker,
    messages_worker: messages::Worker,

    synced: HashMap<OwnedRoomId, bool>,
    messages: HashMap<OwnedRoomId, Vec<worker::messages::MessageContent>>,

    text: String,
    space_rooms: Vec<OwnedRoomId>,
    needs_verification: bool,
}

impl Page {
    pub fn from_client(client: &matrix_sdk::Client) -> Self {
        let verif_worker = verification::Worker::from_client(client.clone());
        let messages_worker = messages::Worker::from_client(client.clone());
        Self {
            client: client.clone(),

            avatars: HashMap::new(),
            current_space: None,
            current_room: None,

            space_rooms: Vec::new(),

            needs_verification: false,

            verif_worker,
            messages_worker,

            synced: HashMap::new(),

            messages: HashMap::new(),
            text: String::new(),
        }
    }
}

impl ViewLike<PageMessage> for Page {
    type Message = Message;

    fn subscription(&self) -> iced::Subscription<Message> {
        Subscription::batch([
            Subscription::run_with(
                Smuggle::new("sliding-sync-worker", self.client.clone()),
                |d| worker::sliding_sync(d.take()),
            )
            .map(Message::SyncUpdate),
            Subscription::run_with(
                Smuggle::new("verification-worker", self.verif_worker.clone()),
                |d| d.take().subscription(),
            )
            .map(Message::VerificationUpdate),
            Subscription::run_with(
                Smuggle::new("messages-worker", self.messages_worker.clone()),
                |d| d.take().subscription(),
            )
            .map(Message::MessagesUpdate),
        ])
    }

    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::OpenSpace(room_id) => {
                self.space_rooms.clear();
                self.current_space = Some(room_id.clone());

                get_space_rooms(&self.client, &room_id).map(move |room| Message::AddSpaceRoom {
                    space: room_id.clone(),
                    room,
                })
            }
            Message::OpenRoom(room_id) => {
                self.current_room = Some(room_id.clone());
                self.messages_worker
                    .send(messages::Request::LatestMessages(room_id));
                Task::none()
            }
            Message::AddSpaceRoom { space, room } => {
                if self.current_space == Some(space) {
                    self.space_rooms.push(room);
                }
                Task::none()
            }
            Message::UpdateMessage(m) => {
                self.text = m;
                Task::none()
            }
            Message::SyncUpdate(message) => match message {
                sliding_sync::Response::UpdateRooms(_room_ids) => Task::none(),
            },
            Message::VerificationUpdate(_message) => Task::none(),
            Message::RoomSynced(room_id, synced) => {
                self.synced.insert(room_id, synced);
                Task::none()
            }
            Message::RoomAvatar(room_id, avatar) => {
                self.avatars.insert(room_id, avatar);
                Task::none()
            }
            Message::MessagesUpdate(response) => {
                match response {
                    messages::Response::Messages(room_id, messages) => {
                        self.messages.entry(room_id).insert_entry(messages);
                    }
                    messages::Response::NewMessage(room_id, message_content) => {
                        self.messages
                            .entry(room_id)
                            .or_default()
                            .push(message_content);
                    }
                }
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Self::Message> {
        Row::new()
            .push(components::space_list(self))
            .push(components::channel_list(self))
            .push(components::room_pane(self))
            .into()
    }
}
