//! TODO: tasks module documentation

use std::{fmt::Debug, sync::Arc};

use tokio::{
    sync::{Barrier, broadcast},
    task::JoinHandle,
};

use crate::executor::Executor;
use crate::links::{InputReceiver, Linker};

pub trait TaskFn: 'static {
    type Input;
    type Output: Clone + Debug + Send + 'static;

    fn run(input: Self::Input) -> impl Future<Output = Self::Output> + Send;
}

pub struct PlannedTask<TF, IR>
where
    TF: TaskFn,
    IR: InputReceiver<Data = TF::Input>,
{
    input_rx: IR,
    output_tx: broadcast::Sender<TF::Output>,
}
impl<TF, IR> PlannedTask<TF, IR>
where
    TF: TaskFn,
    IR: InputReceiver<Data = TF::Input>,
{
    pub fn new(input_rx: IR) -> Self {
        let (output_tx, _) = broadcast::channel(1);
        Self {
            input_rx,
            output_tx,
        }
    }

    pub fn submit(self, ex: &impl Executor) -> Linker<TF::Output> {
        let linker = Linker::new(self.output_tx.clone());
        ex.submit(self.queue());
        linker
    }

    fn queue(self) -> QueuedTask {
        let PlannedTask {
            input_rx,
            output_tx,
        } = self;

        let start_gate = Arc::new(Barrier::new(2));

        let handle = {
            let start_gate = start_gate.clone();
            tokio::spawn(async move {
                start_gate.wait().await;

                let input = input_rx
                    .try_recv()
                    .await
                    // TODO: Better error handling for failed input receives
                    .expect("Failed to receive inputs!");

                let output = TF::run(input).await;

                // A [`tokio::sync::broadcast::error::SendError`] will only be returned if no
                // receivers are active. Some tasks are leaf nodes in the DAG and won't have
                // downstream tasks to receive this task's output, so we ignore this error.
                let _ = output_tx.send(output);
            })
        };

        QueuedTask { start_gate, handle }
    }
}

pub struct QueuedTask {
    start_gate: Arc<Barrier>,
    handle: JoinHandle<()>,
}
impl QueuedTask {
    pub async fn start(self) -> RunningTask {
        self.start_gate.wait().await;
        RunningTask {
            handle: self.handle,
        }
    }
}

pub struct RunningTask {
    handle: JoinHandle<()>,
}
impl RunningTask {
    pub async fn join(self) -> Result<FinishedTask, FailedTask> {
        if self.handle.await.is_ok() {
            Ok(FinishedTask)
        } else {
            Err(FailedTask)
        }
    }
}

#[derive(Debug)]
pub struct FinishedTask;

#[derive(Debug)]
pub struct FailedTask;
