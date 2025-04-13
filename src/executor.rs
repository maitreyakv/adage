//! TODO: executor module documentation

use std::{cell::RefCell, collections::VecDeque};

use crate::tasks::QueuedTask;

pub trait Executor {
    fn submit(&self, task: QueuedTask);

    fn run(&self, flow: impl FnOnce(&Self)) -> impl Future<Output = ()>;
}

pub struct BasicExecutor {
    queue: RefCell<VecDeque<QueuedTask>>,
}
impl BasicExecutor {
    pub fn new() -> Self {
        Self {
            queue: RefCell::new(VecDeque::new()),
        }
    }

    fn pop_queue(&self) -> Option<QueuedTask> {
        self.queue.borrow_mut().pop_front()
    }
}

impl Default for BasicExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor for BasicExecutor {
    fn submit(&self, task: QueuedTask) {
        self.queue.borrow_mut().push_back(task);
    }

    async fn run(&self, flow: impl FnOnce(&Self)) {
        flow(self);

        let mut running = Vec::new();
        while let Some(queued_task) = self.pop_queue() {
            running.push(queued_task.start().await)
        }

        for running_task in running.drain(..) {
            running_task.join().await.unwrap();
        }
    }
}
