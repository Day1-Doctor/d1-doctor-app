pub mod adapter_trait;
pub mod claude;

pub use adapter_trait::{AdapterAgentEvent, AgentDescriptor, FrameworkAdapter};
pub use claude::ClaudeAdapter;
