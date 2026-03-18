use axum::extract::Path;
use axum::{Extension, Json};
use serde_json::{json, Value};

use super::state::ServerState;
use crate::station::tasks::task_types::CreateTaskRequest;

pub async fn health() -> Json<Value> {
    Json(json!({ "status": "ok", "version": "3.0.0-alpha" }))
}

pub async fn list_agents(Extension(state): Extension<ServerState>) -> Json<Value> {
    let agents = state.kernel.list_agents().await;
    let agent_list: Vec<Value> = agents
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "name": a.name,
                "role": format!("{:?}", a.role).to_lowercase(),
                "status": a.status.display_name(),
                "trust_score": a.trust_score,
                "room": a.room,
            })
        })
        .collect();
    Json(json!({ "agents": agent_list }))
}

pub async fn get_agent(
    Extension(state): Extension<ServerState>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.kernel.get_agent(&id).await {
        Some(a) => Json(json!({
            "id": a.id,
            "name": a.name,
            "role": format!("{:?}", a.role).to_lowercase(),
            "status": a.status.display_name(),
            "trust_score": a.trust_score,
            "room": a.room,
        })),
        None => Json(json!({ "error": "not_found", "agent_id": id })),
    }
}

pub async fn list_tasks(Extension(state): Extension<ServerState>) -> Json<Value> {
    let tasks = state.task_engine.list(None).await;
    let task_list: Vec<Value> = tasks
        .into_iter()
        .map(|t| {
            json!({
                "id": t.id,
                "title": t.title,
                "status": format!("{:?}", t.status).to_lowercase(),
                "agent_id": t.agent_id,
                "parent_id": t.parent_id,
            })
        })
        .collect();
    Json(json!({ "tasks": task_list }))
}

pub async fn create_task(
    Extension(state): Extension<ServerState>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let description = body
        .get("description")
        .or_else(|| body.get("title"))
        .and_then(|v| v.as_str())
        .unwrap_or("Unnamed task");

    let task = state
        .task_engine
        .create(CreateTaskRequest {
            description: description.to_string(),
            priority: None,
        })
        .await;

    Json(json!({
        "id": task.id,
        "title": task.title,
        "status": format!("{:?}", task.status).to_lowercase(),
    }))
}

pub async fn get_task(
    Extension(state): Extension<ServerState>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.task_engine.status(&id).await {
        Some(t) => Json(json!({
            "id": t.id,
            "title": t.title,
            "status": format!("{:?}", t.status).to_lowercase(),
        })),
        None => Json(json!({ "error": "not_found", "task_id": id })),
    }
}

pub async fn pause_task(
    Extension(state): Extension<ServerState>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.task_engine.pause(&id).await {
        Ok(_) => Json(json!({ "status": "paused", "task_id": id })),
        Err(e) => Json(json!({ "error": e, "task_id": id })),
    }
}

pub async fn cancel_task(
    Extension(state): Extension<ServerState>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.task_engine.cancel(&id).await {
        Ok(_) => Json(json!({ "status": "cancelled", "task_id": id })),
        Err(e) => Json(json!({ "error": e, "task_id": id })),
    }
}

pub async fn list_artifacts(
    Extension(state): Extension<ServerState>,
    Path(id): Path<String>,
) -> Json<Value> {
    let artifacts = state.task_engine.get_artifacts(&id).await;
    let artifact_list: Vec<Value> = artifacts
        .into_iter()
        .map(|a| {
            json!({
                "id": a.id,
                "task_id": a.task_id,
                "agent_id": a.agent_id,
                "artifact_type": a.artifact_type,
                "name": a.name,
                "path": a.path,
            })
        })
        .collect();
    Json(json!({ "task_id": id, "artifacts": artifact_list }))
}

pub async fn get_costs(Extension(state): Extension<ServerState>) -> Json<Value> {
    let total = state.cost_tracker.get_session_total().await;
    let all = state.cost_tracker.get_all_costs().await;
    let agent_costs: Vec<Value> = all
        .into_iter()
        .map(|c| {
            json!({
                "agent_id": c.agent_id,
                "tokens_in": c.tokens_in,
                "tokens_out": c.tokens_out,
                "cost_dd": c.cost_dd,
                "request_count": c.request_count,
            })
        })
        .collect();
    Json(json!({
        "total_tokens_in": total.tokens_in,
        "total_tokens_out": total.tokens_out,
        "total_cost_dd": total.cost_dd,
        "agents": agent_costs,
    }))
}

pub async fn get_agent_costs(
    Extension(state): Extension<ServerState>,
    Path(agent_id): Path<String>,
) -> Json<Value> {
    match state.cost_tracker.get_agent_cost(&agent_id).await {
        Some(cost) => Json(json!({
            "agent_id": cost.agent_id,
            "tokens_in": cost.tokens_in,
            "tokens_out": cost.tokens_out,
            "cost_dd": cost.cost_dd,
            "cost_usd": cost.cost_usd,
            "request_count": cost.request_count,
        })),
        None => Json(json!({ "error": "no_costs", "agent_id": agent_id })),
    }
}
