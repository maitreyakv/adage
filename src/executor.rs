//! TODO: executor module documentation

use crate::tasks::{QueuedTask, RunningTask};

pub trait Executor {
    fn submit(&mut self, task: QueuedTask);

    fn run(self) -> impl Future<Output = ()>;
}

pub struct BasicExecutor {
    queue: Vec<QueuedTask>,
    running: Vec<RunningTask>,
}
impl BasicExecutor {
    pub fn new() -> Self {
        Self {
            queue: Vec::new(),
            running: Vec::new(),
        }
    }
}

impl Default for BasicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for BasicExecutor {
    fn submit(&mut self, task: QueuedTask) {
        self.queue.push(task);
    }

    async fn run(mut self) {
        for queued_task in self.queue.drain(..) {
            self.running.push(queued_task.start().await);
        }

        for running_task in self.running {
            running_task.join().await.unwrap();
        }
    }
}
