pub mod decomposer;
pub mod handoff;
pub mod parallel;
pub mod router;
pub mod task_engine;
pub mod task_types;

pub use handoff::TaskHandoffManager;
pub use parallel::ParallelExecutor;
pub use task_engine::TaskEngine;
pub use task_types::*;

#[cfg(test)]
mod tests;
