use futures_util::{StreamExt, pin_mut};
use iced::Task;
use matrix_sdk::ruma::OwnedRoomId;
use matrix_sdk::ruma::room::RoomType;

pub fn get_space_rooms(
    client: &matrix_sdk::Client,
    space_id: Option<&OwnedRoomId>,
) -> Task<OwnedRoomId> {
    Task::batch(client.joined_rooms().iter().map(|room| {
        let room = room.clone();
        let space_id = space_id.cloned();
        Task::future(async move {
            let Ok(parent_stream) = room.parent_spaces().await else {
                return Task::none();
            };
            pin_mut!(parent_stream);

            while let Some(Ok(parent)) = parent_stream.next().await {
                match parent {
                    matrix_sdk::room::ParentSpace::Reciprocal(parent_space)
                    | matrix_sdk::room::ParentSpace::WithPowerlevel(parent_space) => {
                        let pid = parent_space.room_id();
                        if space_id.as_ref().is_some_and(|v| v == pid) {
                            return Task::done(room.room_id().to_owned());
                        } else if space_id.is_none() {
                            return Task::none();
                        }
                    }
                    _ => {} // unverified parents
                }
            }

            if space_id.is_none() && room.room_type() != Some(RoomType::Space) {
                Task::done(room.room_id().to_owned())
            } else {
                Task::none()
            }
        })
        .then(|v| v)
    }))
}
