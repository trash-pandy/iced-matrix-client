use futures_util::{SinkExt, Stream, StreamExt};
use iced::futures::channel::mpsc::{self, Sender};
use iced::stream;
use iced::widget::image::Handle;
use matrix_sdk::Room;
use matrix_sdk::media::{MediaFormat, MediaThumbnailSettings};
use matrix_sdk::ruma::events::room::avatar::SyncRoomAvatarEvent;
use matrix_sdk::ruma::{OwnedRoomId, UInt};

use crate::worker::WorkerSubscription;

pub type Worker = super::Worker<Request>;

impl WorkerSubscription<Response> for Worker {
    fn subscription(self) -> impl Stream<Item = Response> {
        stream::channel(64, async move |mut s: Sender<Response>| {
            let mut stream = self
                .client
                .observe_events::<SyncRoomAvatarEvent, Room>()
                .subscribe();

            for room in self.client.joined_rooms() {
                retrieve_avatar(&mut s, &self.client, room.room_id().to_owned()).await;
            }

            let mut last_event = None;
            while let Some((ev, room)) = stream.next().await {
                let eid = Some(ev.event_id().to_owned());
                if last_event == eid {
                    continue;
                }
                last_event = eid;

                retrieve_avatar(&mut s, &self.client, room.room_id().to_owned()).await;
            }
        })
    }
}

async fn retrieve_avatar(
    s: &mut mpsc::Sender<Response>,
    client: &matrix_sdk::Client,
    room_id: OwnedRoomId,
) {
    let Some(room) = client.get_room(&room_id) else {
        return;
    };
    let Ok(avatar) = room
        .avatar(MediaFormat::Thumbnail(MediaThumbnailSettings {
            method: matrix_sdk::ruma::media::Method::Scale,
            width: UInt::new_wrapping(36),
            height: UInt::new_wrapping(36),
            animated: false,
        }))
        .await
    else {
        return;
    };

    s.send(Response::RoomAvatar(
        room_id,
        avatar.map(Handle::from_bytes),
    ))
    .await
    .ok();
}

#[derive(Debug, Clone)]
pub enum Response {
    RoomAvatar(OwnedRoomId, Option<Handle>),
}

#[derive(Debug, Clone)]
pub enum Request {}
