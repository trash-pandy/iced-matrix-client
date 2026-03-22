use futures_util::{Stream, StreamExt};
use iced::futures::channel::mpsc::Sender;
use iced::stream;
use matrix_sdk::ruma::events::AnyToDeviceEvent;
use tracing::info;

use crate::worker::WorkerSubscription;

pub type Worker = super::Worker<Request>;

impl WorkerSubscription<Response> for Worker {
    fn subscription(self) -> impl Stream<Item = Response> {
        stream::channel(64, async move |mut _s: Sender<Response>| {
            let mut events = self
                .client
                .observe_events::<AnyToDeviceEvent, ()>()
                .subscribe();

            while let Some((ev, ())) = events.next().await {
                info!(?ev);
            }
        })
    }
}

#[derive(Debug, Clone)]
pub enum Response {}

#[derive(Debug, Clone)]
pub enum Request {}
