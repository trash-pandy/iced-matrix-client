use futures_util::{StreamExt, pin_mut};
use iced::Task;
use matrix_sdk::ruma::OwnedRoomId;

pub fn get_space_rooms(client: &matrix_sdk::Client, space_id: &OwnedRoomId) -> Task<OwnedRoomId> {
    Task::batch(client.joined_rooms().iter().map(|room| {
        let room = room.clone();
        let space_id = space_id.clone();
        Task::future(async move {
            let Ok(parent_stream) = room.parent_spaces().await else {
                return Task::none();
            };
            pin_mut!(parent_stream);

            while let Some(Ok(parent)) = parent_stream.next().await {
                match parent {
                    matrix_sdk::room::ParentSpace::Reciprocal(parent_space)
                    | matrix_sdk::room::ParentSpace::WithPowerlevel(parent_space) => {
                        if parent_space.room_id() == space_id {
                            return Task::done(room.room_id().to_owned());
                        }
                    }
                    _ => {} // unverified parents
                }
            }

            Task::none()
        })
        .then(|v| v)
    }))
}
