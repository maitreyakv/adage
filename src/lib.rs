//! TODO: adage crate documentation

mod executor;
mod links;
mod tasks;

pub mod prelude {
    pub use crate::executor::{BasicExecutor, Executor};
    pub use crate::links::{EmptyReceiver, InputReceiver, Linker};
    pub use crate::tasks::{PlannedTask, TaskFn};
    pub use adage_macros::task;
}
