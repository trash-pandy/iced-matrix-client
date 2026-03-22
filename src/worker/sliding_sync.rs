use futures_util::{SinkExt, Stream, StreamExt, pin_mut};
use iced::stream;
use matrix_sdk::ruma::events::StateEventType;
use matrix_sdk::ruma::{OwnedRoomId, RoomId, assign};
use matrix_sdk::{SlidingSyncList, SlidingSyncMode};
use ruma_client_api::sync::sync_events::v5::request::{
    AccountData, E2EE, RoomSubscription, ToDevice,
};

#[derive(Debug, Clone)]
pub enum Response {
    UpdateRooms(Vec<OwnedRoomId>),
}

pub fn sliding_sync(client: matrix_sdk::Client) -> impl Stream<Item = Response> {
    stream::channel(64, async move |mut s| {
        let Ok(sync_builder) = client.sliding_sync("sliding-sync") else {
            return;
        };
        let sync_builder = sync_builder
            .version(matrix_sdk::sliding_sync::Version::Native)
            .with_account_data_extension(assign!(AccountData::default(), { enabled: Some(true) }))
            .with_e2ee_extension(assign!(E2EE::default(), { enabled: Some(true) }))
            .with_to_device_extension(assign!(ToDevice::default(), { enabled: Some(true) }));
        let full_sync_list = SlidingSyncList::builder("full-sync")
            .sync_mode(SlidingSyncMode::new_growing(50).maximum_number_of_rooms_to_fetch(500))
            .required_state(vec![(StateEventType::RoomEncryption, String::new())]);
        let active_list = SlidingSyncList::builder("active-list")
            .sync_mode(SlidingSyncMode::new_selective().add_range(0..=9))
            .timeline_limit(5)
            .required_state(vec![
                (StateEventType::RoomEncryption, String::new()),
                (StateEventType::RoomTopic, String::new()),
                (StateEventType::RoomAvatar, String::new()),
            ]);
        let Ok(sliding_sync) = sync_builder
            .add_list(full_sync_list)
            .add_list(active_list)
            .build()
            .await
        else {
            return;
        };

        let sync = sliding_sync.sync();
        let (mut rooms, room_stream) = client.rooms_stream();
        pin_mut!(sync, room_stream);

        let subscribe_to_rooms = |rooms: &[&RoomId]| {
            sliding_sync.subscribe_to_rooms(
                rooms,
                Some(assign!(RoomSubscription::default(), {
                    timeline_limit: 5u32.into(),
                    required_state: vec![
                        (StateEventType::RoomEncryption, String::new()),
                        (StateEventType::RoomTopic, String::new()),
                        (StateEventType::RoomAvatar, String::new()),
                        (StateEventType::RoomMember, String::new()),
                    ]
                })),
                false,
            );
        };

        subscribe_to_rooms(
            rooms
                .iter()
                .map(|v| v.room_id())
                .collect::<Vec<_>>()
                .as_slice(),
        );
        s.send(Response::UpdateRooms(
            rooms.iter().map(|v| v.room_id().to_owned()).collect(),
        ))
        .await
        .ok();

        loop {
            tokio::select! {
                next = sync.next() => {
                    let Some(Ok(summary)) = next else {
                        break;
                    };

                    s.send(Response::UpdateRooms(summary.rooms.clone())).await.ok();
                }
                diff = room_stream.next() => {
                    if let Some(diff) = diff {
                        for update in diff {
                            update.apply(&mut rooms);
                        }
                    }

                    subscribe_to_rooms(
                        rooms
                            .iter()
                            .map(|v| v.room_id())
                            .collect::<Vec<_>>()
                            .as_slice(),
                    );
                }
            }
        }
    })
}
