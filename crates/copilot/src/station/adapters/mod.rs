pub mod adapter_trait;
pub mod claude;
pub mod discovery;
pub mod generic;
pub mod openclaw;

pub use adapter_trait::{AdapterAgentEvent, AgentDescriptor, FrameworkAdapter};
pub use claude::ClaudeAdapter;
pub use discovery::{discover_runtimes, AdapterKind, RuntimeInfo};
pub use generic::GenericAdapter;
pub use openclaw::OpenClawAdapter;
