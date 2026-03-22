pub mod messages;
pub mod sliding_sync;
pub mod verification;

use futures_util::Stream;
pub use sliding_sync::sliding_sync;
use tokio::sync::broadcast;

pub trait WorkerSubscription<R> {
    fn subscription(self) -> impl Stream<Item = R>
    where
        Self: Sized;
}

#[derive(Debug)]
pub struct Worker<R> {
    client: matrix_sdk::Client,
    request_sink: broadcast::Sender<R>,
    request_stream: broadcast::Receiver<R>,
}

impl<R: Clone> Clone for Worker<R> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            request_sink: self.request_sink.clone(),
            request_stream: self.request_stream.resubscribe(),
        }
    }
}

impl<R: Clone> Worker<R> {
    pub fn from_client(client: matrix_sdk::Client) -> Self {
        let (request_sink, request_stream) = broadcast::channel(64);
        Self {
            client,
            request_sink,
            request_stream,
        }
    }

    pub fn send(&self, rq: R) {
        self.request_sink.send(rq).ok();
    }
}
