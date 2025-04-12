//! TODO: executor module documentation

use tokio::{
    sync::broadcast::{Sender, channel},
    task::JoinHandle,
};

use crate::{InputReceiver, PlannedTask, RunningTask, TaskFn, links::Linker};

pub trait Executor {
    fn submit<IR: InputReceiver<Data = TF::Input>, TF: TaskFn>(
        &mut self,
        task: PlannedTask<IR, TF>,
    ) -> Linker<TF::Output>;

    fn run(self) -> impl Future<Output = ()>;
}

pub struct BasicExecutor {
    start_tx: Sender<()>,
    submitted_task_handles: Vec<JoinHandle<RunningTask>>,
}
impl BasicExecutor {
    pub fn new() -> Self {
        let (start_tx, _) = channel(1);
        Self {
            start_tx,
            submitted_task_handles: Vec::new(),
        }
    }
}

impl Default for BasicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for BasicExecutor {
    fn submit<IR: InputReceiver<Data = TF::Input>, TF: TaskFn>(
        &mut self,
        task: PlannedTask<IR, TF>,
    ) -> Linker<TF::Output> {
        let linker = task.linker();

        let handle = {
            let mut start_rx = self.start_tx.subscribe();
            tokio::spawn(async move {
                // TODO: Better error handling here
                start_rx.recv().await.unwrap();
                task.start()
            })
        };
        self.submitted_task_handles.push(handle);

        linker
    }

    async fn run(self) {
        // TODO: Better error handling here
        self.start_tx.send(()).unwrap();

        let mut running_tasks: Vec<RunningTask> = Vec::new();
        for handle in self.submitted_task_handles {
            // TODO: Better error handling here
            running_tasks.push(handle.await.unwrap());
        }

        for task in running_tasks {
            // TODO: Better error handling here
            task.join().await.unwrap();
        }
    }
}
