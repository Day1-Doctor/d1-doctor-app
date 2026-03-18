use axum::{routing::get, Router};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use super::{handlers, ws_handler};

/// HTTP + WebSocket IPC server for Station Runtime.
///
/// The React UI connects to this server on `localhost:14200` to drive
/// agent orchestration, task management, and real-time event streaming.
pub struct StationServer {
    port: u16,
}

impl StationServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub fn default_port() -> u16 {
        14200 // Day1 Copilot IPC port
    }

    /// Build the axum Router with all routes.
    pub fn router(&self) -> Router {
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
            // Artifacts
            .route(
                "/api/v1/tasks/:id/artifacts",
                get(handlers::list_artifacts),
            )
            // Cost
            .route("/api/v1/costs", get(handlers::get_costs))
            .route(
                "/api/v1/costs/:agent_id",
                get(handlers::get_agent_costs),
            )
            // WebSocket event stream
            .route("/ws/events", get(ws_handler::ws_upgrade))
            // CORS for localhost
            .layer(CorsLayer::permissive())
    }

    /// Start the server, binding to 127.0.0.1 on the configured port.
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], self.port));
        let router = self.router();
        let listener = tokio::net::TcpListener::bind(addr).await?;
        tracing::info!("Station Runtime IPC server listening on {}", addr);
        axum::serve(listener, router).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::StatusCode;
    use tower::util::ServiceExt; // for oneshot

    #[tokio::test]
    async fn test_health_endpoint() {
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
        let server = StationServer::new(0);
        let app = server.router();
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
    async fn test_get_agent_costs_endpoint() {
        let server = StationServer::new(0);
        let app = server.router();
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
}
