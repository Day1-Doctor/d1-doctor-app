pub mod auth;
pub mod station;

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use station::costs::CostTracker;
use station::events::EventBus;
use station::executor::{AgentExecutor, StepRunner, TaskOrchestrator};
use station::kernel::{AgentKernel, AgentRole, AgentStatus};
use station::llm::{LlmClient, LlmDecomposer};
use station::permissions::{ApprovalDecision, ApprovalResponse, PermissionEngine};
use station::runtime::BuiltinRuntime;
use station::server::{ServerState, StationServer};
use station::skills::SkillRegistry;
use station::tasks::decomposer::TaskDecomposer;
use station::tasks::handoff::TaskHandoffManager;
use station::tasks::router::TaskRouter;
use station::tasks::task_engine::TaskEngine;
use tauri::Manager;
use tokio::sync::RwLock;

/// Shared application state accessible from Tauri commands.
pub struct AppState {
    pub kernel: Arc<AgentKernel>,
    pub event_bus: Arc<EventBus>,
    pub task_engine: Arc<TaskEngine>,
    pub cost_tracker: Arc<CostTracker>,
    pub runtime: Arc<BuiltinRuntime>,
    pub decomposer: Arc<TaskDecomposer>,
    pub router: Arc<TaskRouter>,
    pub llm_client: Arc<RwLock<LlmClient>>,
    pub orchestrator: Arc<TaskOrchestrator>,
    pub permission_engine: Arc<PermissionEngine>,
}

// --- Serializable response types for Tauri commands ---

#[derive(Serialize, Clone)]
struct AgentInfo {
    id: String,
    name: String,
    role: String,
    status: String,
    trust_score: f64,
    room: String,
}

#[derive(Serialize, Clone)]
struct TaskInfo {
    id: String,
    title: String,
    status: String,
    agent_id: Option<String>,
    parent_id: Option<String>,
    step_index: Option<u32>,
}

// --- Tauri Commands ---

#[tauri::command]
async fn list_agents(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<AgentInfo>, String> {
    let agents = state.kernel.list_agents().await;
    Ok(agents
        .into_iter()
        .map(|a| AgentInfo {
            id: a.id,
            name: a.name,
            role: format!("{:?}", a.role).to_lowercase(),
            status: a.status.display_name().to_string(),
            trust_score: a.trust_score,
            room: a.room,
        })
        .collect())
}

#[tauri::command]
async fn get_agent(
    state: tauri::State<'_, Arc<AppState>>,
    agent_id: String,
) -> Result<Option<AgentInfo>, String> {
    let agent = state.kernel.get_agent(&agent_id).await;
    Ok(agent.map(|a| AgentInfo {
        id: a.id,
        name: a.name,
        role: format!("{:?}", a.role).to_lowercase(),
        status: a.status.display_name().to_string(),
        trust_score: a.trust_score,
        room: a.room,
    }))
}

#[tauri::command]
async fn create_task(
    state: tauri::State<'_, Arc<AppState>>,
    description: String,
    #[allow(unused_variables)] max_agents: Option<usize>,
) -> Result<TaskInfo, String> {
    // Decompose the task using LLM-powered decomposer (falls back to keyword-based)
    let client = state.llm_client.read().await.clone();
    let llm_decomposer = LlmDecomposer::new(client);
    let mut plan = llm_decomposer.decompose(&description).await;

    // Office Spot enforcement: if the plan requires more agent roles than
    // the user's tier allows, collapse all steps to the orchestrator role
    // (single-agent fallback via Dr. Bob).
    if let Some(limit) = max_agents {
        let mut roles: Vec<&str> = plan
            .steps
            .iter()
            .map(|s| s.suggested_role.as_str())
            .collect();
        roles.sort();
        roles.dedup();
        if roles.len() > limit {
            for step in &mut plan.steps {
                step.suggested_role = "orchestrator".to_string();
            }
        }
    }

    // Route the plan (creates parent + subtasks)
    let parent_id = state.router.route_plan(plan).await?;

    // Spawn background execution via the orchestrator
    let orchestrator = state.orchestrator.clone();
    let parent_id_clone = parent_id.clone();
    let task_engine_clone = state.task_engine.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = orchestrator.orchestrate(&parent_id_clone).await {
            tracing::error!("Task execution failed for {}: {}", parent_id_clone, e);
            // Mark parent task as failed (if not already)
            let _ = task_engine_clone.cancel(&parent_id_clone).await;
        }
    });

    // Get the created task
    let task = state
        .task_engine
        .status(&parent_id)
        .await
        .ok_or("Task not found after creation")?;

    Ok(TaskInfo {
        id: task.id,
        title: task.title,
        status: format!("{:?}", task.status).to_lowercase(),
        agent_id: task.agent_id,
        parent_id: task.parent_id,
        step_index: task.step_index,
    })
}

#[tauri::command]
async fn list_tasks(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<TaskInfo>, String> {
    let tasks = state.task_engine.list(None).await;
    Ok(tasks
        .into_iter()
        .map(|t| TaskInfo {
            id: t.id,
            title: t.title,
            status: format!("{:?}", t.status).to_lowercase(),
            agent_id: t.agent_id,
            parent_id: t.parent_id,
            step_index: t.step_index,
        })
        .collect())
}

#[tauri::command]
async fn get_task_steps(
    state: tauri::State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<Vec<TaskInfo>, String> {
    let subtasks = state.task_engine.get_subtasks(&task_id).await;
    Ok(subtasks
        .into_iter()
        .map(|t| TaskInfo {
            id: t.id,
            title: t.title,
            status: format!("{:?}", t.status).to_lowercase(),
            agent_id: t.agent_id,
            parent_id: t.parent_id,
            step_index: t.step_index,
        })
        .collect())
}

#[tauri::command]
async fn pause_task(
    state: tauri::State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<String, String> {
    state.task_engine.pause(&task_id).await?;
    Ok("paused".to_string())
}

#[tauri::command]
async fn cancel_task(
    state: tauri::State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<String, String> {
    state.task_engine.cancel(&task_id).await?;
    Ok("cancelled".to_string())
}

/// Result of a tier-limit check before activating agents.
#[derive(Serialize, Clone)]
struct TierCheckResult {
    allowed: bool,
    required_agents: usize,
    max_agents: usize,
    /// The distinct agent roles the plan requires.
    required_roles: Vec<String>,
}

/// Check whether the user's subscription tier allows activating the required
/// number of agents for a given task description.
///
/// Returns `TierCheckResult` so the UI can decide whether to show an upgrade
/// prompt or fall back to single-agent mode (Dr. Bob only).
#[tauri::command]
async fn check_tier_limit(
    state: tauri::State<'_, Arc<AppState>>,
    description: String,
    max_agents: usize,
) -> Result<TierCheckResult, String> {
    // Decompose the task to discover the required roles.
    let plan = state.decomposer.decompose(&description);

    let mut roles: Vec<String> = plan
        .steps
        .iter()
        .map(|s| s.suggested_role.clone())
        .collect();
    roles.sort();
    roles.dedup();

    let required = roles.len();
    // Default to 1 (Free Man tier) if 0 is passed.
    let limit = if max_agents == 0 { 1 } else { max_agents };

    Ok(TierCheckResult {
        allowed: required <= limit,
        required_agents: required,
        max_agents: limit,
        required_roles: roles,
    })
}

/// List available LLM providers/models from the gateway.
///
/// Returns an empty list if the user is not authenticated or the gateway is
/// unreachable, allowing the UI to degrade gracefully.
#[tauri::command]
async fn list_providers(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    let client = state.llm_client.read().await;
    let models = client.list_models().await;
    Ok(models)
}

#[tauri::command]
async fn start_task(
    state: tauri::State<'_, Arc<AppState>>,
    task_id: String,
) -> Result<TaskInfo, String> {
    let task = state.task_engine.start(&task_id).await?;
    Ok(TaskInfo {
        id: task.id,
        title: task.title,
        status: format!("{:?}", task.status).to_lowercase(),
        agent_id: task.agent_id,
        parent_id: task.parent_id,
        step_index: task.step_index,
    })
}

#[tauri::command]
async fn get_runtime_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let agent_count = state.kernel.agent_count().await;
    let idle = state.kernel.agents_by_status(AgentStatus::Idle).await.len();
    Ok(serde_json::json!({
        "version": "3.0.0-alpha",
        "agents_total": agent_count,
        "agents_idle": idle,
        "gateway_url": state.runtime.gateway_url(),
    }))
}

// --- Auth Commands ---

/// Store a JWT token obtained from the OAuth callback.
///
/// Optionally accepts a refresh token to enable automatic token renewal.
#[tauri::command]
async fn store_auth_token(
    state: tauri::State<'_, Arc<AppState>>,
    token: String,
    refresh_token: Option<String>,
) -> Result<(), String> {
    auth::write_auth_file(&token, refresh_token.as_deref())?;

    // Update LLM client with the JWT as bearer token
    let mut client = state.llm_client.write().await;
    *client = client.clone().with_api_key(&token);

    Ok(())
}

/// Process an OAuth deep link callback URL.
///
/// Parses `day1copilot://auth/callback?token=...&refresh_token=...`, stores the
/// credentials, and updates the LLM client. Returns the access token on success.
#[tauri::command]
async fn handle_auth_callback(
    state: tauri::State<'_, Arc<AppState>>,
    url: String,
) -> Result<String, String> {
    let callback = auth::parse_callback_url(&url)?;

    auth::write_auth_file(&callback.token, callback.refresh_token.as_deref())?;

    // Update LLM client with the new JWT
    let mut client = state.llm_client.write().await;
    *client = client.clone().with_api_key(&callback.token);

    tracing::info!("OAuth callback processed successfully");
    Ok(callback.token)
}

/// Refresh the stored JWT using the refresh token.
///
/// Reads the refresh token from `~/.day1copilot/auth.json`, calls the Supabase
/// auth token refresh endpoint via the gateway, stores the new credentials,
/// and returns the new access token.
///
/// Returns `None` if no refresh token is stored (non-fatal).
#[tauri::command]
async fn refresh_auth_token(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Option<String>, String> {
    let auth_data = match auth::read_auth_file() {
        Ok(data) => data,
        Err(_) => return Ok(None),
    };

    let refresh_token = match auth_data.get("refresh_token").and_then(|v| v.as_str()) {
        Some(rt) => rt.to_string(),
        None => {
            tracing::debug!("No refresh token stored, skipping refresh");
            return Ok(None);
        }
    };

    // Call Supabase-compatible token refresh via gateway
    let http = reqwest::Client::new();
    let resp = http
        .post("https://gateway.day1.doctor/dr-agent/v1/auth/refresh")
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        tracing::warn!("Token refresh failed ({}): {}", status, body);
        return Err(format!("Token refresh failed ({}): {}", status, body));
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse refresh response: {}", e))?;

    let new_token = body
        .get("access_token")
        .or_else(|| body.get("token"))
        .and_then(|v| v.as_str())
        .ok_or("Refresh response missing access_token")?
        .to_string();

    let new_refresh = body
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .unwrap_or(&refresh_token);

    // Persist updated credentials
    auth::write_auth_file(&new_token, Some(new_refresh))?;

    // Update the LLM client
    let mut client = state.llm_client.write().await;
    *client = client.clone().with_api_key(&new_token);

    tracing::info!("JWT token refreshed successfully");
    Ok(Some(new_token))
}

/// Read the stored JWT token, if present.
#[tauri::command]
async fn get_auth_token() -> Result<Option<String>, String> {
    let auth_file = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .join(".day1copilot/auth.json");
    if !auth_file.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&auth_file).map_err(|e| e.to_string())?;
    let data: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;
    Ok(data
        .get("token")
        .and_then(|v| v.as_str())
        .map(String::from))
}

#[tauri::command]
async fn fetch_balance(api_key: String) -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let resp = client
        .get("https://gateway.day1.doctor/dr-agent/v1/balance")
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if resp.status() == 401 {
        return Err("Invalid API key".into());
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(body)
}

/// Respond to a pending approval request from the UI.
///
/// The frontend calls this when a user clicks "Approve" or "Deny" on an
/// approval card. The decision is forwarded to the [`PermissionEngine`] which
/// unblocks the waiting agent execution.
#[tauri::command]
async fn respond_approval(
    state: tauri::State<'_, Arc<AppState>>,
    request_id: String,
    decision: String, // "approve", "approve_always", or "deny"
) -> Result<(), String> {
    let decision = match decision.as_str() {
        "approve" => ApprovalDecision::AllowOnce,
        "approve_always" => ApprovalDecision::AllowAlways,
        "deny" => ApprovalDecision::Reject,
        other => return Err(format!("invalid decision: {other}, expected approve|approve_always|deny")),
    };

    let response = ApprovalResponse {
        request_id: request_id.clone(),
        decision,
    };

    // Try the oneshot-channel pending map first (blocking executor flow).
    let result = state.permission_engine.respond(response.clone()).await;
    if result.is_ok() {
        return Ok(());
    }

    // Fall back to the FIFO queue (non-blocking UI queue).
    state
        .permission_engine
        .respond_queued(response)
        .await
        .map(|_| ())
}

/// List all pending approval requests for the UI to render.
#[tauri::command]
async fn list_pending_approvals(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<Vec<serde_json::Value>, String> {
    // Collect from both the oneshot pending map and the FIFO queue.
    let pending = state.permission_engine.get_pending().await;
    let queued = state.permission_engine.get_pending_ordered().await;

    let mut all: Vec<serde_json::Value> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for req in pending.into_iter().chain(queued.into_iter()) {
        if seen.insert(req.id.clone()) {
            all.push(serde_json::json!({
                "id": req.id,
                "agent_id": req.agent_id,
                "agent_name": req.agent_name,
                "tool_name": req.tool_name,
                "risk_level": format!("{:?}", req.risk_level),
                "context": req.context,
                "created_at": req.created_at.to_rfc3339(),
            }));
        }
    }

    Ok(all)
}

#[tauri::command]
async fn clear_auth() -> Result<(), String> {
    let auth_file = dirs::home_dir()
        .ok_or("Cannot determine home directory")?
        .join(".day1copilot/auth.json");
    if auth_file.exists() {
        std::fs::remove_file(&auth_file).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn run() {
    // Initialize tracing for logging
    tracing_subscriber::fmt()
        .with_env_filter("d1_copilot=info,station=debug")
        .with_target(false)
        .init();

    tracing::info!("Starting Day1 Copilot v3.0-alpha");

    tauri::Builder::default()
        .setup(|app| {
            // Create shared runtime components
            let event_bus = Arc::new(EventBus::new(1024));
            let kernel = Arc::new(AgentKernel::new());
            let task_engine = Arc::new(TaskEngine::new());
            let cost_tracker = Arc::new(CostTracker::with_event_bus(event_bus.clone()));
            let runtime = Arc::new(BuiltinRuntime::new(
                kernel.clone(),
                event_bus.clone(),
                "https://gateway.day1.doctor/v1",
            ));
            let decomposer = Arc::new(TaskDecomposer::new());
            let router = Arc::new(TaskRouter::new(
                kernel.clone(),
                task_engine.clone(),
                event_bus.clone(),
            ));

            // Create LLM client and try to load stored API key
            let mut llm_client = LlmClient::new("https://gateway.day1.doctor");
            if let Err(e) = llm_client.load_api_key() {
                tracing::info!("LLM client: no stored key ({}), will use fallback decomposer until authenticated", e);
            }
            let llm_client = Arc::new(RwLock::new(llm_client));

            // Create the execution pipeline components
            let permission_engine = Arc::new(PermissionEngine::new());
            let skill_registry = Arc::new(SkillRegistry::new());

            let agent_executor = Arc::new(AgentExecutor::new(
                llm_client.clone(),
                kernel.clone(),
                event_bus.clone(),
                cost_tracker.clone(),
                permission_engine.clone(),
                skill_registry,
            ));

            let step_runner = Arc::new(StepRunner::new(agent_executor.clone(), 3));

            let handoff_manager = Arc::new(TaskHandoffManager::new(
                task_engine.clone(),
                kernel.clone(),
                event_bus.clone(),
            ));

            let orchestrator = Arc::new(TaskOrchestrator::new(
                agent_executor,
                step_runner,
                task_engine.clone(),
                kernel.clone(),
                event_bus.clone(),
                handoff_manager,
            ));

            let app_state = Arc::new(AppState {
                kernel: kernel.clone(),
                event_bus: event_bus.clone(),
                task_engine: task_engine.clone(),
                cost_tracker: cost_tracker.clone(),
                runtime: runtime.clone(),
                decomposer,
                router,
                llm_client,
                orchestrator,
                permission_engine: permission_engine.clone(),
            });

            // Store state for Tauri commands
            app.manage(app_state);

            // Build server state from shared components
            let server_state = ServerState {
                kernel: kernel.clone(),
                task_engine,
                event_bus,
                cost_tracker,
            };

            // Spawn runtime initialization + IPC server in background
            let rt = runtime.clone();
            tauri::async_runtime::spawn(async move {
                // Initialize the 6 built-in agents
                match rt.initialize().await {
                    Ok(ids) => tracing::info!("Dr. Bob's Office initialized: {} agents", ids.len()),
                    Err(e) => tracing::error!("Failed to initialize runtime: {}", e),
                }

                // Start the HTTP + WebSocket IPC server for external adapters
                if let Err(e) =
                    StationServer::start(StationServer::default_port(), server_state).await
                {
                    tracing::error!("IPC server error: {}", e);
                }
            });

            tracing::info!("Day1 Copilot setup complete");
            Ok(())
        })
        .plugin(tauri_plugin_deep_link::init())
        .invoke_handler(tauri::generate_handler![
            list_agents,
            get_agent,
            create_task,
            list_tasks,
            get_task_steps,
            pause_task,
            cancel_task,
            start_task,
            list_providers,
            get_runtime_status,
            store_auth_token,
            get_auth_token,
            fetch_balance,
            clear_auth,
            check_tier_limit,
            respond_approval,
            list_pending_approvals,
            handle_auth_callback,
            refresh_auth_token,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Day1 Copilot");
}
