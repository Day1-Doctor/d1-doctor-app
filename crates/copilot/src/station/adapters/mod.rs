pub mod adapter_trait;
pub mod claude;
pub mod openclaw;

pub use adapter_trait::{AdapterAgentEvent, AgentDescriptor, FrameworkAdapter};
pub use claude::ClaudeAdapter;
pub use openclaw::OpenClawAdapter;
