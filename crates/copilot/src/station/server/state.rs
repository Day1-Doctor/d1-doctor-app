use std::sync::Arc;

use crate::station::costs::CostTracker;
use crate::station::events::EventBus;
use crate::station::kernel::AgentKernel;
use crate::station::tasks::task_engine::TaskEngine;

/// Shared state accessible from all HTTP and WebSocket handlers.
///
/// Each field is an `Arc` so the state can be cheaply cloned into
/// axum's `Extension` layer and shared across concurrent requests.
#[derive(Clone)]
pub struct ServerState {
    pub kernel: Arc<AgentKernel>,
    pub task_engine: Arc<TaskEngine>,
    pub event_bus: Arc<EventBus>,
    pub cost_tracker: Arc<CostTracker>,
}
