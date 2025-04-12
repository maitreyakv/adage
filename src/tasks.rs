//! TODO: tasks module documentation

use std::fmt::Debug;

use tokio::{
    sync::broadcast::{Sender, channel},
    task::JoinHandle,
};

use crate::{
    executor::Executor,
    links::{InputReceiver, Linker},
};

pub trait TaskFn: 'static {
    type Input;
    type Output: Clone + Debug + Send + 'static;
    type Error: std::error::Error;

    fn run(input: Self::Input) -> impl Future<Output = Result<Self::Output, Self::Error>> + Send;
}

pub struct PlannedTask<IR, TF>
where
    IR: InputReceiver<Data = TF::Input>,
    TF: TaskFn,
{
    input_receiver: IR,
    output_sender: Sender<TF::Output>,
}
impl<IR, TF> PlannedTask<IR, TF>
where
    IR: InputReceiver<Data = TF::Input>,
    TF: TaskFn,
{
    pub fn new(input_receiver: IR) -> Self {
        let (output_sender, _) = channel(1);
        Self {
            input_receiver,
            output_sender,
        }
    }

    pub fn linker(&self) -> Linker<TF::Output> {
        Linker::new(self.output_sender.clone())
    }

    pub fn submit(self, ex: &mut impl Executor) -> Linker<TF::Output> {
        ex.submit(self)
    }

    pub fn start(self) -> RunningTask {
        let Self {
            input_receiver,
            output_sender,
        } = self;

        let handle = tokio::spawn(async move {
            let input = input_receiver
                .try_recv()
                .await
                // TODO: Better error handling for failed input receives
                .expect("Failed to receive inputs!");

            // TODO: Better error handling for crashed tasks
            let output = TF::run(input).await.expect("Task failed to run!");

            // A [`tokio::sync::broadcast::error::SendError`] will only be returned if no
            // receivers are active. Some tasks are leaf nodes in the DAG and won't have
            // downstream tasks to receive this task's output, so we ignore this error.
            let _ = output_sender.send(output);
        });

        RunningTask { handle }
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
