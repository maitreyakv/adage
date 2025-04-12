//! TODO: adage crate documentation

mod executor;
mod links;
mod tasks;

pub use executor::{BasicExecutor, Executor};
pub use links::{EmptyReceiver, InputReceiver, Linker};
pub use tasks::{FailedTask, FinishedTask, PlannedTask, RunningTask, TaskFn};

pub use adage_macros::task;
