//! TODO: tasks module documentation

use tokio::{sync::broadcast::Sender, task::JoinHandle};

use crate::links::InputReceiver;

pub trait TaskFn {
    type Input;
    type Output: std::fmt::Debug + Send + 'static;
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
    fn start(self) -> RunningTask {
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
    async fn join(self) -> Result<FinishedTask, FailedTask> {
        if self.handle.await.is_ok() {
            Ok(FinishedTask)
        } else {
            Err(FailedTask)
        }
    }
}

struct FinishedTask;

struct FailedTask;
