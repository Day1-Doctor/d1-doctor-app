use axum::extract::Path;
use axum::Json;
use serde_json::{json, Value};

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "3.0.0-alpha" }))
}

pub async fn list_agents() -> Json<Value> {
    // Placeholder -- will be wired to AgentKernel
    Json(json!({ "agents": [] }))
}

pub async fn get_agent(Path(id): Path<String>) -> Json<Value> {
    Json(json!({ "error": "not_found", "agent_id": id }))
}

pub async fn list_tasks() -> Json<Value> {
    Json(json!({ "tasks": [] }))
}

pub async fn create_task(Json(body): Json<Value>) -> Json<Value> {
    Json(json!({ "status": "created", "task": body }))
}

pub async fn get_task(Path(id): Path<String>) -> Json<Value> {
    Json(json!({ "error": "not_found", "task_id": id }))
}

pub async fn pause_task(Path(id): Path<String>) -> Json<Value> {
    Json(json!({ "status": "paused", "task_id": id }))
}

pub async fn cancel_task(Path(id): Path<String>) -> Json<Value> {
    Json(json!({ "status": "cancelled", "task_id": id }))
}

pub async fn list_artifacts(Path(id): Path<String>) -> Json<Value> {
    Json(json!({ "task_id": id, "artifacts": [] }))
}

pub async fn get_costs() -> Json<Value> {
    Json(json!({ "total_tokens": 0, "total_cost_dd": 0.0, "agents": {} }))
}

pub async fn get_agent_costs(Path(agent_id): Path<String>) -> Json<Value> {
    Json(json!({ "agent_id": agent_id, "tokens_in": 0, "tokens_out": 0, "cost_dd": 0.0 }))
}
