use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use super::event_types::AgentEvent;

/// A broadcast-based event bus for agent events.
///
/// Every published event is sent to all current subscribers **and**
/// appended to a bounded in-memory history so that late-joining
/// subscribers can replay recent events.
pub struct EventBus {
    sender: broadcast::Sender<AgentEvent>,
    /// Recent events kept for replay.
    history: Arc<RwLock<Vec<AgentEvent>>>,
    max_history: usize,
}

impl EventBus {
    /// Create a new `EventBus`.
    ///
    /// * `capacity` -- broadcast channel buffer size (how many un-consumed
    ///   events a slow subscriber can lag behind before it starts losing
    ///   messages).
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            history: Arc::new(RwLock::new(Vec::new())),
            max_history: 1000,
        }
    }

    /// Create a new `EventBus` with a custom history cap.
    pub fn with_max_history(capacity: usize, max_history: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self {
            sender,
            history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    /// Publish an event to all current subscribers and record it in history.
    pub async fn publish(&self, event: AgentEvent) {
        // Append to history (bounded).
        {
            let mut hist = self.history.write().await;
            if hist.len() >= self.max_history {
                // Remove oldest to make room.
                hist.remove(0);
            }
            hist.push(event.clone());
        }

        // Best-effort broadcast -- if there are no receivers the send
        // returns an error which we intentionally discard.
        let _ = self.sender.send(event);
    }

    /// Subscribe to all future events. The returned receiver will yield
    /// events published **after** this call; use [`history`] for past events.
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.sender.subscribe()
    }

    /// Return the most recent `limit` events from the history buffer.
    pub async fn history(&self, limit: usize) -> Vec<AgentEvent> {
        let hist = self.history.read().await;
        let start = hist.len().saturating_sub(limit);
        hist[start..].to_vec()
    }

    /// Return the most recent `limit` events for a specific agent.
    pub async fn history_for_agent(&self, agent_id: &str, limit: usize) -> Vec<AgentEvent> {
        let hist = self.history.read().await;
        hist.iter()
            .filter(|e| e.agent_id == agent_id)
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Number of active subscribers.
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}
