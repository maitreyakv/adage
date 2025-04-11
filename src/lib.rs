pub use adage_macros::task;

use std::sync::Arc;

use tokio::sync::Notify;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinHandle;

pub struct QueuedTask<D: Clone> {
    handle: JoinHandle<()>,
    output_sender: Sender<D>,
    start_notifier: Arc<Notify>,
}
impl<D: Clone> QueuedTask<D> {
    pub fn new(
        handle: JoinHandle<()>,
        output_sender: Sender<D>,
        start_notifier: Arc<Notify>,
    ) -> Self {
        Self {
            handle,
            output_sender,
            start_notifier,
        }
    }

    pub fn link(&self) -> Receiver<D> {
        self.output_sender.subscribe()
    }

    pub fn start(self) -> RunningTask {
        self.start_notifier.notify_last();
        RunningTask {
            handle: self.handle,
        }
    }
}

pub struct RunningTask {
    handle: JoinHandle<()>,
}
impl RunningTask {
    pub async fn join(self) -> CompletedTask {
        self.handle.await.unwrap();
        CompletedTask
    }
}

pub struct CompletedTask;
