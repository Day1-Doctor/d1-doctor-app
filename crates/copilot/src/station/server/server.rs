use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use super::state::ServerState;
use super::{handlers, ws_handler};

/// HTTP + WebSocket IPC server for Station Runtime.
///
/// The React UI connects to this server on `localhost:14200` to drive
/// agent orchestration, task management, and real-time event streaming.
pub struct StationServer;

impl StationServer {
    pub fn default_port() -> u16 {
        14200 // Day1 Copilot IPC port
    }

    /// Build the axum Router with all routes, wired to real state.
    pub fn router(state: ServerState) -> Router {
        Router::new()
            // Health
            .route("/health", get(handlers::health))
            // Agent routes
            .route("/api/v1/agents", get(handlers::list_agents))
            .route("/api/v1/agents/:id", get(handlers::get_agent))
            // Task routes
            .route(
                "/api/v1/tasks",
                get(handlers::list_tasks).post(handlers::create_task),
            )
            .route("/api/v1/tasks/:id", get(handlers::get_task))
            .route(
                "/api/v1/tasks/:id/pause",
                axum::routing::post(handlers::pause_task),
            )
            .route(
                "/api/v1/tasks/:id/cancel",
                axum::routing::post(handlers::cancel_task),
            )
            .route(
                "/api/v1/tasks/:id/start",
                axum::routing::post(handlers::start_task),
            )
            // Artifacts
            .route("/api/v1/tasks/:id/artifacts", get(handlers::list_artifacts))
            // Cost
            .route("/api/v1/costs", get(handlers::get_costs))
            .route("/api/v1/costs/:agent_id", get(handlers::get_agent_costs))
            // WebSocket event stream
            .route("/ws/events", get(ws_handler::ws_upgrade))
            // CORS for localhost
            .layer(CorsLayer::permissive())
            .layer(axum::Extension(state))
    }

    /// Start the server, binding to 127.0.0.1 on the configured port.
    pub async fn start(port: u16, state: ServerState) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let router = Self::router(state);
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Station Runtime IPC server listening on {}", addr);
        axum::serve(listener, router).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::costs::CostTracker;
    use crate::station::events::EventBus;
    use crate::station::kernel::AgentKernel;
    use crate::station::tasks::task_engine::TaskEngine;
    use axum::body::Body;
    use axum::http::StatusCode;
    use std::sync::Arc;
    use tower::util::ServiceExt; // for oneshot

    fn test_state() -> ServerState {
        let event_bus = Arc::new(EventBus::new(64));
        ServerState {
            kernel: Arc::new(AgentKernel::new()),
            task_engine: Arc::new(TaskEngine::new()),
            event_bus: event_bus.clone(),
            cost_tracker: Arc::new(CostTracker::with_event_bus(event_bus)),
        }
    }

    #[tokio::test]
    async fn test_health_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_agents_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_create_task_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"title": "Test task"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_costs_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/costs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_agent_404() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/agents/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_pause_task_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/task-123/pause")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_cancel_task_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/task-123/cancel")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_artifacts_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/tasks/task-123/artifacts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_start_task_endpoint() {
        let state = test_state();

        // First create a task so there's something to start.
        use crate::station::tasks::task_types::CreateTaskRequest;
        let task = state
            .task_engine
            .create(CreateTaskRequest {
                description: "Task to start".to_string(),
                priority: None,
            })
            .await;

        let app = StationServer::router(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri(&format!("/api/v1/tasks/{}/start", task.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["status"], "running");
        assert_eq!(json["id"], task.id);
    }

    #[tokio::test]
    async fn test_start_task_not_found() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks/nonexistent/start")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["error"].as_str().unwrap().contains("not found"));
    }

    #[tokio::test]
    async fn test_get_agent_costs_endpoint() {
        let app = StationServer::router(test_state());
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/costs/agent-abc")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_agents_returns_registered_agents() {
        let state = test_state();
        // Register an agent
        use crate::station::kernel::{AgentDescriptor, AgentRole, Framework};
        let agent = AgentDescriptor::new("test-bob", AgentRole::Orchestrator, Framework::Builtin, "claude-sonnet-4");
        let _id = state.kernel.register(agent).await;

        let app = StationServer::router(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/agents")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let agents = json["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
        assert_eq!(agents[0]["name"], "test-bob");
        assert_eq!(agents[0]["status"], "idle");
    }

    #[tokio::test]
    async fn test_create_and_get_task() {
        let state = test_state();
        let app = StationServer::router(state.clone());

        // Create a task
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .method("POST")
                    .uri("/api/v1/tasks")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"description": "Write unit tests"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["title"], "Write unit tests");
        assert_eq!(json["status"], "pending");

        // List tasks to verify it's there
        let app2 = StationServer::router(state);
        let response2 = app2
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/tasks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let body2 = axum::body::to_bytes(response2.into_body(), usize::MAX)
            .await
            .unwrap();
        let json2: serde_json::Value = serde_json::from_slice(&body2).unwrap();
        let tasks = json2["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), 1);
    }

    #[tokio::test]
    async fn test_costs_with_usage() {
        let state = test_state();

        // Record some usage
        state
            .cost_tracker
            .record_usage("agent-1", "anthropic", "claude-opus-4", 100_000, 50_000)
            .await;

        let app = StationServer::router(state);
        let response = app
            .oneshot(
                axum::http::Request::builder()
                    .uri("/api/v1/costs")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["total_tokens_in"], 100_000);
        assert_eq!(json["total_tokens_out"], 50_000);
        assert!(json["total_cost_dd"].as_f64().unwrap() > 0.0);
        let agents = json["agents"].as_array().unwrap();
        assert_eq!(agents.len(), 1);
    }
}
