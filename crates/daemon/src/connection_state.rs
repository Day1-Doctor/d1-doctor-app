//! Connection state machine for daemon ↔ cloud orchestrator link.
//!
//! Tracks connectivity, queues outbound messages while offline,
//! flushes the queue on reconnection, and emits state-change events
//! so the UI (Tauri) layer can update in real-time.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex, RwLock};
use tokio::time;

// ---------------------------------------------------------------------------
// Connection state enum
// ---------------------------------------------------------------------------

/// Represents the daemon's connection to the cloud orchestrator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionState {
    /// Fully connected and operational.
    Online,
    /// Connection lost; actively trying to re-establish.
    Reconnecting,
    /// All reconnection attempts exhausted or explicitly disconnected.
    Offline,
    /// Connected but experiencing issues (high latency, partial failures).
    Degraded,
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Online => write!(f, "online"),
            Self::Reconnecting => write!(f, "reconnecting"),
            Self::Offline => write!(f, "offline"),
            Self::Degraded => write!(f, "degraded"),
        }
    }
}

// ---------------------------------------------------------------------------
// State-change event
// ---------------------------------------------------------------------------

/// Event emitted whenever the connection state changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStateEvent {
    pub previous: ConnectionState,
    pub current: ConnectionState,
    pub message: String,
    /// Unix-millis timestamp of the transition.
    pub timestamp: i64,
}

// ---------------------------------------------------------------------------
// Queued message
// ---------------------------------------------------------------------------

/// An outbound message buffered while the daemon is offline.
#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub id: String,
    pub payload: Vec<u8>,
    pub queued_at: Instant,
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Tuning knobs for the connection state machine.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// How often the health-check ping fires.
    pub ping_interval: Duration,
    /// How long to wait for a pong before declaring degraded / offline.
    pub ping_timeout: Duration,
    /// Maximum number of reconnection attempts before giving up.
    pub max_reconnect_attempts: u32,
    /// Base delay between reconnection attempts (doubles each attempt).
    pub reconnect_base_delay: Duration,
    /// Maximum number of messages to buffer while offline.
    pub max_queue_size: usize,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(30),
            ping_timeout: Duration::from_secs(10),
            max_reconnect_attempts: 10,
            reconnect_base_delay: Duration::from_secs(1),
            max_queue_size: 500,
        }
    }
}

// ---------------------------------------------------------------------------
// Connection state machine
// ---------------------------------------------------------------------------

/// Manages connection lifecycle, offline message queue, and event broadcast.
pub struct ConnectionStateMachine {
    state: RwLock<ConnectionState>,
    queue: Mutex<VecDeque<QueuedMessage>>,
    event_tx: broadcast::Sender<ConnectionStateEvent>,
    config: ConnectionConfig,
    reconnect_attempts: Mutex<u32>,
}

impl ConnectionStateMachine {
    /// Create a new state machine starting in `Offline`.
    pub fn new(config: ConnectionConfig) -> (Arc<Self>, broadcast::Receiver<ConnectionStateEvent>) {
        let (event_tx, event_rx) = broadcast::channel(64);
        let sm = Arc::new(Self {
            state: RwLock::new(ConnectionState::Offline),
            queue: Mutex::new(VecDeque::new()),
            event_tx,
            config,
            reconnect_attempts: Mutex::new(0),
        });
        (sm, event_rx)
    }

    // -- Accessors ----------------------------------------------------------

    pub async fn state(&self) -> ConnectionState {
        *self.state.read().await
    }

    pub async fn queue_len(&self) -> usize {
        self.queue.lock().await.len()
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConnectionStateEvent> {
        self.event_tx.subscribe()
    }

    // -- Transitions --------------------------------------------------------

    /// Transition to a new state if the transition is valid.
    /// Returns `Ok(event)` on success or `Err(reason)` if the transition is
    /// not allowed.
    pub async fn transition(
        &self,
        target: ConnectionState,
        message: impl Into<String>,
    ) -> Result<ConnectionStateEvent, String> {
        let mut current = self.state.write().await;
        let prev = *current;

        if prev == target {
            return Err(format!("already in state {target}"));
        }

        if !Self::is_valid_transition(prev, target) {
            return Err(format!("invalid transition {prev} → {target}"));
        }

        *current = target;

        // Reset reconnect counter when going Online.
        if target == ConnectionState::Online {
            *self.reconnect_attempts.lock().await = 0;
        }

        let event = ConnectionStateEvent {
            previous: prev,
            current: target,
            message: message.into(),
            timestamp: chrono::Utc::now().timestamp_millis(),
        };

        // Best-effort broadcast — if no receivers, that's fine.
        let _ = self.event_tx.send(event.clone());

        tracing::info!(
            previous = %prev,
            current = %target,
            "connection state changed"
        );

        Ok(event)
    }

    /// Which transitions are legal?
    ///
    /// ```text
    ///  Online ──► Reconnecting ──► Offline
    ///    │  ▲          │              │
    ///    │  └──────────┘              │
    ///    ▼                            │
    ///  Degraded ──► Reconnecting      │
    ///    ▲                            │
    ///    └────── Online ◄─────────────┘
    /// ```
    pub fn is_valid_transition(from: ConnectionState, to: ConnectionState) -> bool {
        use ConnectionState::*;
        matches!(
            (from, to),
            // Normal lifecycle
            (Offline, Online)
                | (Offline, Reconnecting)
                | (Online, Reconnecting)
                | (Online, Degraded)
                | (Degraded, Reconnecting)
                | (Degraded, Online)
                | (Reconnecting, Online)
                | (Reconnecting, Offline)
                // Edge: direct offline from degraded (give up)
                | (Degraded, Offline)
        )
    }

    // -- Offline message queue ----------------------------------------------

    /// Enqueue a message for later delivery. If already online the caller
    /// should send directly; this is for offline/reconnecting buffering.
    pub async fn enqueue(&self, msg: QueuedMessage) -> Result<(), String> {
        let mut q = self.queue.lock().await;
        if q.len() >= self.config.max_queue_size {
            return Err("offline message queue full".into());
        }
        q.push_back(msg);
        Ok(())
    }

    /// Drain all queued messages (oldest first). Typically called after
    /// transitioning to `Online`.
    pub async fn flush_queue(&self) -> Vec<QueuedMessage> {
        let mut q = self.queue.lock().await;
        q.drain(..).collect()
    }

    // -- Reconnection logic -------------------------------------------------

    /// Attempt one reconnection cycle.  Returns the delay the caller should
    /// wait before retrying, or `None` if max attempts exceeded (caller
    /// should transition to `Offline`).
    pub async fn next_reconnect_delay(&self) -> Option<Duration> {
        let mut attempts = self.reconnect_attempts.lock().await;
        if *attempts >= self.config.max_reconnect_attempts {
            return None;
        }
        let delay = self.config.reconnect_base_delay * 2u32.saturating_pow(*attempts);
        *attempts += 1;
        Some(delay)
    }

    pub async fn reconnect_attempts(&self) -> u32 {
        *self.reconnect_attempts.lock().await
    }

    // -- Health-check ping task ---------------------------------------------

    /// Spawn a background task that pings on `ping_interval`.
    ///
    /// `ping_fn` should send a ping and return `true` if a pong was received
    /// within the timeout, or `false` otherwise. The state machine will
    /// transition accordingly.
    pub fn spawn_health_check<F, Fut>(self: &Arc<Self>, ping_fn: F) -> tokio::task::JoinHandle<()>
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = bool> + Send,
    {
        let sm = Arc::clone(self);
        let interval = sm.config.ping_interval;

        tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            // Skip the first immediate tick.
            ticker.tick().await;

            loop {
                ticker.tick().await;

                let current = sm.state().await;
                if current == ConnectionState::Offline {
                    // No point pinging when we know we're offline.
                    continue;
                }

                let pong = ping_fn().await;

                match (current, pong) {
                    (ConnectionState::Online, false) => {
                        let _ = sm
                            .transition(ConnectionState::Degraded, "health check: ping timeout")
                            .await;
                    }
                    (ConnectionState::Degraded, true) => {
                        let _ = sm
                            .transition(ConnectionState::Online, "health check: recovered")
                            .await;
                    }
                    (ConnectionState::Degraded, false) => {
                        let _ = sm
                            .transition(
                                ConnectionState::Reconnecting,
                                "health check: consecutive failures",
                            )
                            .await;
                    }
                    _ => {}
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> ConnectionConfig {
        ConnectionConfig {
            ping_interval: Duration::from_millis(50),
            ping_timeout: Duration::from_millis(20),
            max_reconnect_attempts: 3,
            reconnect_base_delay: Duration::from_millis(10),
            max_queue_size: 5,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_initial_state_is_offline() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());
        assert_eq!(sm.state().await, ConnectionState::Offline);
    }

    #[tokio::test]
    async fn test_valid_transitions() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        // Offline → Online
        let ev = sm
            .transition(ConnectionState::Online, "connected")
            .await
            .unwrap();
        assert_eq!(ev.previous, ConnectionState::Offline);
        assert_eq!(ev.current, ConnectionState::Online);
        assert_eq!(sm.state().await, ConnectionState::Online);

        // Online → Degraded
        sm.transition(ConnectionState::Degraded, "high latency")
            .await
            .unwrap();
        assert_eq!(sm.state().await, ConnectionState::Degraded);

        // Degraded → Reconnecting
        sm.transition(ConnectionState::Reconnecting, "retry")
            .await
            .unwrap();
        assert_eq!(sm.state().await, ConnectionState::Reconnecting);

        // Reconnecting → Online
        sm.transition(ConnectionState::Online, "recovered")
            .await
            .unwrap();
        assert_eq!(sm.state().await, ConnectionState::Online);
    }

    #[tokio::test]
    async fn test_invalid_transition_rejected() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());
        // Offline → Degraded is not valid
        let result = sm.transition(ConnectionState::Degraded, "bad").await;
        assert!(result.is_err());
        assert_eq!(sm.state().await, ConnectionState::Offline);
    }

    #[tokio::test]
    async fn test_same_state_transition_rejected() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());
        let result = sm.transition(ConnectionState::Offline, "noop").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_event_broadcast() {
        let (sm, mut rx) = ConnectionStateMachine::new(cfg());

        sm.transition(ConnectionState::Online, "up").await.unwrap();
        let ev = rx.recv().await.unwrap();
        assert_eq!(ev.current, ConnectionState::Online);
        assert_eq!(ev.message, "up");
    }

    #[tokio::test]
    async fn test_offline_queue_enqueue_and_flush() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        for i in 0..3 {
            sm.enqueue(QueuedMessage {
                id: format!("msg-{i}"),
                payload: vec![i as u8],
                queued_at: Instant::now(),
            })
            .await
            .unwrap();
        }
        assert_eq!(sm.queue_len().await, 3);

        let msgs = sm.flush_queue().await;
        assert_eq!(msgs.len(), 3);
        assert_eq!(msgs[0].id, "msg-0");
        assert_eq!(sm.queue_len().await, 0);
    }

    #[tokio::test]
    async fn test_queue_overflow_rejected() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        for i in 0..5 {
            sm.enqueue(QueuedMessage {
                id: format!("msg-{i}"),
                payload: vec![],
                queued_at: Instant::now(),
            })
            .await
            .unwrap();
        }

        // 6th should fail
        let result = sm
            .enqueue(QueuedMessage {
                id: "overflow".into(),
                payload: vec![],
                queued_at: Instant::now(),
            })
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_reconnect_delay_exponential_backoff() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        // attempt 0 → 10ms * 2^0 = 10ms
        let d = sm.next_reconnect_delay().await.unwrap();
        assert_eq!(d, Duration::from_millis(10));

        // attempt 1 → 10ms * 2^1 = 20ms
        let d = sm.next_reconnect_delay().await.unwrap();
        assert_eq!(d, Duration::from_millis(20));

        // attempt 2 → 10ms * 2^2 = 40ms
        let d = sm.next_reconnect_delay().await.unwrap();
        assert_eq!(d, Duration::from_millis(40));

        // attempt 3 → exhausted
        assert!(sm.next_reconnect_delay().await.is_none());
    }

    #[tokio::test]
    async fn test_reconnect_counter_resets_on_online() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        // Go online first
        sm.transition(ConnectionState::Online, "up").await.unwrap();
        // Then reconnecting
        sm.transition(ConnectionState::Reconnecting, "lost")
            .await
            .unwrap();

        // Consume one attempt
        sm.next_reconnect_delay().await.unwrap();
        assert_eq!(sm.reconnect_attempts().await, 1);

        // Back online resets counter
        sm.transition(ConnectionState::Online, "back")
            .await
            .unwrap();
        assert_eq!(sm.reconnect_attempts().await, 0);
    }

    #[tokio::test]
    async fn test_offline_to_reconnecting_to_offline() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        sm.transition(ConnectionState::Reconnecting, "trying")
            .await
            .unwrap();
        assert_eq!(sm.state().await, ConnectionState::Reconnecting);

        sm.transition(ConnectionState::Offline, "gave up")
            .await
            .unwrap();
        assert_eq!(sm.state().await, ConnectionState::Offline);
    }

    #[tokio::test]
    async fn test_health_check_degrades_on_ping_failure() {
        let (sm, mut rx) = ConnectionStateMachine::new(cfg());

        // Start online
        sm.transition(ConnectionState::Online, "up").await.unwrap();
        let _ = rx.recv().await;

        // Health check that always fails
        let handle = sm.spawn_health_check(|| async { false });

        // Wait for at least one ping cycle
        time::sleep(Duration::from_millis(80)).await;

        let state = sm.state().await;
        assert!(
            state == ConnectionState::Degraded || state == ConnectionState::Reconnecting,
            "expected degraded or reconnecting, got {state}"
        );
        handle.abort();
    }

    #[tokio::test]
    async fn test_full_lifecycle() {
        let (sm, _rx) = ConnectionStateMachine::new(cfg());

        // Offline → Online (initial connect)
        sm.transition(ConnectionState::Online, "connected")
            .await
            .unwrap();

        // Online → Degraded (latency spike)
        sm.transition(ConnectionState::Degraded, "slow")
            .await
            .unwrap();

        // Degraded → Online (recovered)
        sm.transition(ConnectionState::Online, "ok").await.unwrap();

        // Online → Reconnecting (disconnect)
        sm.transition(ConnectionState::Reconnecting, "lost")
            .await
            .unwrap();

        // Queue messages while reconnecting
        sm.enqueue(QueuedMessage {
            id: "cmd-1".into(),
            payload: b"hello".to_vec(),
            queued_at: Instant::now(),
        })
        .await
        .unwrap();

        // Reconnecting → Online (success)
        sm.transition(ConnectionState::Online, "reconnected")
            .await
            .unwrap();

        // Flush queue
        let msgs = sm.flush_queue().await;
        assert_eq!(msgs.len(), 1);
        assert_eq!(msgs[0].id, "cmd-1");
    }

    #[test]
    fn test_state_serialization() {
        let json = serde_json::to_string(&ConnectionState::Online).unwrap();
        assert_eq!(json, "\"online\"");

        let state: ConnectionState = serde_json::from_str("\"reconnecting\"").unwrap();
        assert_eq!(state, ConnectionState::Reconnecting);
    }

    #[test]
    fn test_event_serialization() {
        let event = ConnectionStateEvent {
            previous: ConnectionState::Online,
            current: ConnectionState::Reconnecting,
            message: "link down".into(),
            timestamp: 1709827200000,
        };
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"previous\":\"online\""));
        assert!(json.contains("\"current\":\"reconnecting\""));
    }

    #[test]
    fn test_all_valid_transitions_exhaustive() {
        use ConnectionState::*;
        let valid = [
            (Offline, Online),
            (Offline, Reconnecting),
            (Online, Reconnecting),
            (Online, Degraded),
            (Degraded, Reconnecting),
            (Degraded, Online),
            (Degraded, Offline),
            (Reconnecting, Online),
            (Reconnecting, Offline),
        ];
        for (from, to) in valid {
            assert!(
                ConnectionStateMachine::is_valid_transition(from, to),
                "{from} → {to} should be valid"
            );
        }

        let invalid = [
            (Offline, Degraded),
            (Online, Offline),
            (Reconnecting, Degraded),
        ];
        for (from, to) in invalid {
            assert!(
                !ConnectionStateMachine::is_valid_transition(from, to),
                "{from} → {to} should be invalid"
            );
        }
    }
}
