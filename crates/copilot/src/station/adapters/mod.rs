pub mod adapter_trait;
pub mod claude;
pub mod generic;
pub mod openclaw;

pub use adapter_trait::{AdapterAgentEvent, AgentDescriptor, FrameworkAdapter};
pub use claude::ClaudeAdapter;
pub use generic::GenericAdapter;
pub use openclaw::OpenClawAdapter;
