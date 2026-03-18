pub mod station;

use std::sync::Arc;

use serde::Serialize;
use station::events::EventBus;
use station::kernel::{AgentKernel, AgentStatus};
use station::runtime::BuiltinRuntime;
use station::server::StationServer;
use station::tasks::task_engine::TaskEngine;
use station::tasks::task_types::CreateTaskRequest;
use station::tasks::decomposer::TaskDecomposer;
use station::tasks::router::TaskRouter;
use tauri::Manager;

/// Shared application state accessible from Tauri commands.
pub struct AppState {
    pub kernel: Arc<AgentKernel>,
    pub event_bus: Arc<EventBus>,
    pub task_engine: Arc<TaskEngine>,
    pub runtime: Arc<BuiltinRuntime>,
    pub decomposer: Arc<TaskDecomposer>,
    pub router: Arc<TaskRouter>,
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
) -> Result<TaskInfo, String> {
    // Decompose the task
    let plan = state.decomposer.decompose(&description);

    // Route the plan (creates parent + subtasks)
    let parent_id = state.router.route_plan(plan).await?;

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

#[tauri::command]
async fn get_runtime_status(
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<serde_json::Value, String> {
    let agent_count = state.kernel.agent_count().await;
    let idle = state
        .kernel
        .agents_by_status(AgentStatus::Idle)
        .await
        .len();
    Ok(serde_json::json!({
        "version": "3.0.0-alpha",
        "agents_total": agent_count,
        "agents_idle": idle,
        "gateway_url": state.runtime.gateway_url(),
    }))
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

            let app_state = Arc::new(AppState {
                kernel: kernel.clone(),
                event_bus: event_bus.clone(),
                task_engine,
                runtime: runtime.clone(),
                decomposer,
                router,
            });

            // Store state for Tauri commands
            app.manage(app_state);

            // Spawn runtime initialization + IPC server in background
            let rt = runtime.clone();
            tauri::async_runtime::spawn(async move {
                // Initialize the 6 built-in agents
                match rt.initialize().await {
                    Ok(ids) => tracing::info!("Dr. Bob's Office initialized: {} agents", ids.len()),
                    Err(e) => tracing::error!("Failed to initialize runtime: {}", e),
                }

                // Start the HTTP + WebSocket IPC server for external adapters
                let server = StationServer::new(StationServer::default_port());
                if let Err(e) = server.start().await {
                    tracing::error!("IPC server error: {}", e);
                }
            });

            tracing::info!("Day1 Copilot setup complete");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_agents,
            get_agent,
            create_task,
            list_tasks,
            get_task_steps,
            pause_task,
            cancel_task,
            get_runtime_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Day1 Copilot");
}
