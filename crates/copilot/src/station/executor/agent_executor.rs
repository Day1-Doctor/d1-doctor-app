use std::sync::Arc;
use tokio::sync::RwLock;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::station::costs::cost_tracker::CostTracker;
use crate::station::events::bus::EventBus;
use crate::station::events::event_types::{AgentEvent, EventType};
use crate::station::kernel::agent::AgentDescriptor;
use crate::station::kernel::agent_state::Trigger;
use crate::station::kernel::kernel::AgentKernel;
use crate::station::llm::client::{ChatMessage, ChatRequest, LlmClient};
use crate::station::permissions::PermissionEngine;
use crate::station::runtime::presets::{builtin_presets, AgentPreset};
use crate::station::skills::executor::SkillExecutor;
use crate::station::skills::skill_registry::SkillRegistry;
use crate::station::skills::skill_types::SkillDefinition;
use crate::station::tasks::task_types::TaskSpec;

/// Token usage summary that can be serialized across the WS boundary.
///
/// This mirrors the gateway's `UsageInfo` but adds `Clone` + `Serialize` so
/// it can be embedded in [`StepResult`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

/// The result of executing a single task step via an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// The LLM-generated output text.
    pub output: String,
    /// Token usage reported by the gateway, if available.
    pub tokens_used: Option<Usage>,
    /// Paths or identifiers of artifacts produced during the step.
    pub artifacts: Vec<String>,
    /// Records of any tool calls made during the step.
    pub tool_calls: Vec<ToolCallRecord>,
}

/// A record of a single tool invocation within a step.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallRecord {
    pub tool_name: String,
    pub input: String,
    pub output: String,
    pub approved: bool,
}

/// The core execution engine that makes agents perform work.
///
/// `AgentExecutor` orchestrates the full lifecycle of a task step:
/// FSM state transitions, skill selection, prompt composition, LLM gateway
/// calls, cost tracking, and event emission.
pub struct AgentExecutor {
    llm_client: Arc<RwLock<LlmClient>>,
    kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
    cost_tracker: Arc<CostTracker>,
    /// Reserved for future tool-call approval flow.
    #[allow(dead_code)]
    permission_engine: Arc<PermissionEngine>,
    /// Retained for direct registry access; skill selection is delegated to
    /// `skill_executor`.
    #[allow(dead_code)]
    skill_registry: Arc<SkillRegistry>,
    skill_executor: SkillExecutor,
}

impl AgentExecutor {
    /// Create a new executor wired to the station subsystems.
    pub fn new(
        llm_client: Arc<RwLock<LlmClient>>,
        kernel: Arc<AgentKernel>,
        event_bus: Arc<EventBus>,
        cost_tracker: Arc<CostTracker>,
        permission_engine: Arc<PermissionEngine>,
        skill_registry: Arc<SkillRegistry>,
    ) -> Self {
        let skill_executor = SkillExecutor::new(skill_registry.clone());
        Self {
            llm_client,
            kernel,
            event_bus,
            cost_tracker,
            permission_engine,
            skill_registry,
            skill_executor,
        }
    }

    /// Execute a single task step using the given agent.
    ///
    /// This drives the full lifecycle:
    /// 1. Transition agent to Working (idle -> working)
    /// 2. Select a matching skill from the registry
    /// 3. Compose a system prompt from agent persona + skill + step context
    /// 4. Transition to Thinking (working -> thinking)
    /// 5. Call the LLM gateway
    /// 6. Track cost
    /// 7. Transition back to Working (thinking -> working)
    /// 8. Complete: transition to Idle (working -> idle)
    /// 9. Emit `task.step_completed` event
    pub async fn execute_step(
        &self,
        step: &TaskSpec,
        agent: &AgentDescriptor,
    ) -> Result<StepResult, String> {
        // 1. Transition agent to Working: idle -> working.
        let (from, to) = self
            .kernel
            .apply_trigger(&agent.id, Trigger::TaskAssign)
            .await?;
        self.emit_state_changed(&agent.id, from.display_name(), to.display_name())
            .await;

        // 2. Select skill from registry based on step title keywords.
        let skill = self.select_skill(step, agent);

        // 3. Compose system prompt.
        let system_prompt = self.compose_prompt(agent, skill.as_ref(), step);

        // 4. Transition to Thinking: working -> thinking.
        let (from, to) = self
            .kernel
            .apply_trigger(&agent.id, Trigger::LlmCallStart)
            .await?;
        self.emit_state_changed(&agent.id, from.display_name(), to.display_name())
            .await;

        // 5. Call LLM gateway.
        let model = self.get_agent_model(agent);
        let request = ChatRequest {
            model: model.clone(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: step.title.clone(),
                },
            ],
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: false,
        };

        let response = self
            .llm_client
            .read()
            .await
            .chat(request, &agent.name)
            .await?;

        // 6. Track cost if usage info is available.
        if let Some(ref usage) = response.usage {
            self.cost_tracker
                .record_usage(
                    &agent.id,
                    "gateway",
                    &model,
                    usage.prompt_tokens as u64,
                    usage.completion_tokens as u64,
                )
                .await;
        }

        // 7. Transition back to Working: thinking -> working.
        let (from, to) = self
            .kernel
            .apply_trigger(&agent.id, Trigger::LlmCallEnd)
            .await?;
        self.emit_state_changed(&agent.id, from.display_name(), to.display_name())
            .await;

        // 8. Parse response content from choices.
        let output = response
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // 8b. Process tool calls if present in the response.
        // TODO(D1D-260): When the LLM response includes tool_calls, iterate
        // over them and for each:
        //   a. Emit tool.started event
        //   b. Execute the tool (via PermissionEngine approval flow)
        //   c. Emit tool.finished event
        // Placeholder: the tool execution loop will be wired in Phase 2
        // (agent execution loop). For now we emit events for any tool calls
        // that a future implementation populates.
        let tool_calls: Vec<ToolCallRecord> = vec![];
        for tool_call in &tool_calls {
            self.emit_tool_started(&agent.id, &tool_call.tool_name, &tool_call.input)
                .await;
            self.emit_tool_finished(
                &agent.id,
                &tool_call.tool_name,
                &tool_call.output,
                0, // duration_ms placeholder
            )
            .await;
        }

        // 8c. Emit artifact.created events for any artifacts produced.
        // TODO(D1D-260): When step execution produces artifacts (files,
        // documents, charts, etc.), populate this vec and emit events.
        let artifacts: Vec<String> = vec![];
        for artifact_path in &artifacts {
            self.emit_artifact_created(&agent.id, &step.id, artifact_path)
                .await;
        }

        // 9. Complete: transition to Idle (working -> idle).
        let (from, to) = self
            .kernel
            .apply_trigger(&agent.id, Trigger::TaskComplete)
            .await?;
        self.emit_state_changed(&agent.id, from.display_name(), to.display_name())
            .await;

        // 10. Emit task.step_completed event.
        let step_index = step.step_index.unwrap_or(0);
        let step_completed_event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent.id.clone(),
            timestamp: Utc::now(),
            event_type: EventType::TaskStepCompleted {
                task_id: step.id.clone(),
                step_index,
                result: serde_json::json!({ "output": &output }),
            },
        };
        self.event_bus.publish(step_completed_event).await;

        let tokens_used = response.usage.map(|u| Usage {
            prompt_tokens: u.prompt_tokens,
            completion_tokens: u.completion_tokens,
            total_tokens: u.total_tokens,
        });

        Ok(StepResult {
            output,
            tokens_used,
            artifacts,
            tool_calls,
        })
    }

    /// Compose a system prompt from agent persona, optional skill instructions,
    /// and step context.
    fn compose_prompt(
        &self,
        agent: &AgentDescriptor,
        skill: Option<&SkillDefinition>,
        step: &TaskSpec,
    ) -> String {
        let preset = self.get_agent_preset(agent);

        let mut parts: Vec<String> = Vec::new();

        // Agent persona from preset.
        if let Some(preset) = preset {
            parts.push(preset.system_prompt.to_string());
        } else {
            parts.push(format!(
                "You are {}, a {:?} agent.",
                agent.name, agent.role
            ));
        }

        // Skill instructions if matched.
        if let Some(skill) = skill {
            parts.push(format!(
                "Active skill: {} - {}",
                skill.name, skill.description
            ));
        }

        // Step context.
        parts.push(format!("Current task: {}", step.title));
        if let Some(ref input) = step.input {
            parts.push(format!("Task input: {}", input));
        }

        parts.join("\n\n")
    }

    /// Select the best matching skill from the registry based on step title
    /// keywords and the agent's role.
    ///
    /// Delegates to [`SkillExecutor`] as the canonical skill selection source.
    fn select_skill(&self, step: &TaskSpec, agent: &AgentDescriptor) -> Option<SkillDefinition> {
        let role_str = role_to_code_name(agent.role);
        self.skill_executor.select_skill(&step.title, role_str)
    }

    /// Determine the LLM model to use for the given agent.
    ///
    /// Looks up the agent's role in the built-in presets and returns the
    /// configured `default_model`. Falls back to `"claude-sonnet-4"` if the
    /// agent's role does not match any preset.
    fn get_agent_model(&self, agent: &AgentDescriptor) -> String {
        self.get_agent_preset(agent)
            .map(|p| p.default_model.to_string())
            .unwrap_or_else(|| "claude-sonnet-4".to_string())
    }

    /// Find the built-in preset matching the agent's role.
    fn get_agent_preset(&self, agent: &AgentDescriptor) -> Option<AgentPreset> {
        let code_name = role_to_code_name(agent.role);
        builtin_presets()
            .into_iter()
            .find(|p| p.code_name == code_name)
    }

    /// Emit an `agent.state_changed` event via the event bus.
    async fn emit_state_changed(&self, agent_id: &str, from: &str, to: &str) {
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            event_type: EventType::AgentStateChanged {
                from: from.to_string(),
                to: to.to_string(),
            },
        };
        self.event_bus.publish(event).await;
    }

    /// Emit a `tool.started` event via the event bus.
    async fn emit_tool_started(&self, agent_id: &str, tool_name: &str, input: &str) {
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            event_type: EventType::ToolStarted {
                tool_name: tool_name.to_string(),
                params: serde_json::json!({ "input": input }),
            },
        };
        self.event_bus.publish(event).await;
    }

    /// Emit a `tool.finished` event via the event bus.
    async fn emit_tool_finished(
        &self,
        agent_id: &str,
        tool_name: &str,
        output: &str,
        duration_ms: u64,
    ) {
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            event_type: EventType::ToolFinished {
                tool_name: tool_name.to_string(),
                result: serde_json::json!({ "output": output }),
                duration_ms,
            },
        };
        self.event_bus.publish(event).await;
    }

    /// Emit an `artifact.created` event via the event bus.
    async fn emit_artifact_created(&self, agent_id: &str, task_id: &str, path: &str) {
        // Infer artifact type from file extension.
        let artifact_type = if path.ends_with(".rs") || path.ends_with(".ts") || path.ends_with(".py") {
            "code"
        } else if path.ends_with(".md") || path.ends_with(".txt") {
            "document"
        } else if path.ends_with(".png") || path.ends_with(".jpg") || path.ends_with(".svg") {
            "image"
        } else if path.ends_with(".json") || path.ends_with(".csv") {
            "data"
        } else {
            "file"
        };

        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            event_type: EventType::ArtifactCreated {
                task_id: task_id.to_string(),
                artifact_type: artifact_type.to_string(),
                path: path.to_string(),
            },
        };
        self.event_bus.publish(event).await;
    }
}

/// Map an `AgentRole` to the corresponding preset `code_name` string.
fn role_to_code_name(role: crate::station::kernel::agent::AgentRole) -> &'static str {
    use crate::station::kernel::agent::AgentRole;
    match role {
        AgentRole::Orchestrator => "orchestrator",
        AgentRole::Researcher => "researcher",
        AgentRole::Analyst => "analyst",
        AgentRole::Writer => "writer",
        AgentRole::Coder => "coder",
        AgentRole::Operator => "operator",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::station::kernel::agent::{AgentRole, Framework};

    fn make_executor() -> AgentExecutor {
        let llm_client = Arc::new(RwLock::new(
            LlmClient::new("https://gateway.day1.doctor"),
        ));
        let kernel = Arc::new(AgentKernel::new());
        let event_bus = Arc::new(EventBus::new(64));
        let cost_tracker = Arc::new(CostTracker::new());
        let permission_engine = Arc::new(PermissionEngine::new());
        let skill_registry = Arc::new(SkillRegistry::new());

        AgentExecutor::new(
            llm_client,
            kernel,
            event_bus,
            cost_tracker,
            permission_engine,
            skill_registry,
        )
    }

    fn make_agent(name: &str, role: AgentRole) -> AgentDescriptor {
        AgentDescriptor::new(name, role, Framework::Builtin, "claude-sonnet-4")
    }

    fn make_step(title: &str) -> TaskSpec {
        TaskSpec {
            id: Uuid::new_v4().to_string(),
            title: title.to_string(),
            status: crate::station::tasks::task_types::TaskStatus::Pending,
            agent_id: None,
            parent_id: None,
            step_index: Some(1),
            priority: 0,
            input: None,
            output: None,
            sub_tasks: Vec::new(),
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        }
    }

    #[test]
    fn test_compose_prompt_includes_persona() {
        let executor = make_executor();
        let agent = make_agent("Dr. Bob", AgentRole::Orchestrator);
        let step = make_step("Plan the project");

        let prompt = executor.compose_prompt(&agent, None, &step);

        // Should contain the orchestrator's system prompt from presets.
        assert!(
            prompt.contains("Dr. Bob"),
            "prompt should contain agent persona"
        );
        assert!(
            prompt.contains("Plan the project"),
            "prompt should contain task title"
        );
    }

    #[test]
    fn test_compose_prompt_with_skill() {
        let executor = make_executor();
        let agent = make_agent("Scout", AgentRole::Researcher);
        let step = make_step("Research competitors");

        let skill = SkillDefinition {
            id: "deep-research".into(),
            name: "Deep Research".into(),
            description: "Multi-hop web search".into(),
            used_by: vec!["researcher".into()],
            steps: vec![],
        };

        let prompt = executor.compose_prompt(&agent, Some(&skill), &step);

        assert!(
            prompt.contains("Deep Research"),
            "prompt should include skill name"
        );
        assert!(
            prompt.contains("Multi-hop web search"),
            "prompt should include skill description"
        );
        assert!(
            prompt.contains("Research competitors"),
            "prompt should include step title"
        );
    }

    #[test]
    fn test_compose_prompt_with_input() {
        let executor = make_executor();
        let agent = make_agent("Pixel", AgentRole::Coder);
        let mut step = make_step("Write unit tests");
        step.input = Some(serde_json::json!({ "file": "main.rs" }));

        let prompt = executor.compose_prompt(&agent, None, &step);

        assert!(
            prompt.contains("main.rs"),
            "prompt should include task input"
        );
    }

    #[test]
    fn test_select_skill_matches_research_keywords() {
        let executor = make_executor();
        let agent = make_agent("Scout", AgentRole::Researcher);
        let step = make_step("Deep research into market trends");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_some(), "should match a research skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "deep-research");
    }

    #[test]
    fn test_select_skill_matches_code_review() {
        let executor = make_executor();
        let agent = make_agent("Pixel", AgentRole::Coder);
        let step = make_step("Code review of the auth module");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_some(), "should match the code review skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "code-review");
    }

    #[test]
    fn test_select_skill_no_match() {
        let executor = make_executor();
        let agent = make_agent("Pixel", AgentRole::Coder);
        let step = make_step("Do something completely unrelated xyz");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_none(), "should not match any skill");
    }

    #[test]
    fn test_select_skill_no_match_for_role() {
        let executor = make_executor();
        // Even with a known role, if step title doesn't match, no skill.
        let agent = make_agent("Atlas", AgentRole::Operator);
        let step = make_step("Fix the typo in readme");

        let skill = executor.select_skill(&step, &agent);

        // Operator has "web-automation" - "fix the typo" won't match.
        assert!(skill.is_none());
    }

    #[test]
    fn test_get_agent_model_orchestrator() {
        let executor = make_executor();
        let agent = make_agent("Dr. Bob", AgentRole::Orchestrator);
        assert_eq!(executor.get_agent_model(&agent), "claude-sonnet-4");
    }

    #[test]
    fn test_get_agent_model_researcher() {
        let executor = make_executor();
        let agent = make_agent("Scout", AgentRole::Researcher);
        assert_eq!(executor.get_agent_model(&agent), "claude-haiku-4-5");
    }

    #[test]
    fn test_get_agent_model_coder() {
        let executor = make_executor();
        let agent = make_agent("Pixel", AgentRole::Coder);
        assert_eq!(executor.get_agent_model(&agent), "claude-sonnet-4");
    }

    #[test]
    fn test_get_agent_model_operator() {
        let executor = make_executor();
        let agent = make_agent("Atlas", AgentRole::Operator);
        assert_eq!(executor.get_agent_model(&agent), "claude-haiku-4-5");
    }

    #[test]
    fn test_role_to_code_name() {
        assert_eq!(role_to_code_name(AgentRole::Orchestrator), "orchestrator");
        assert_eq!(role_to_code_name(AgentRole::Researcher), "researcher");
        assert_eq!(role_to_code_name(AgentRole::Analyst), "analyst");
        assert_eq!(role_to_code_name(AgentRole::Writer), "writer");
        assert_eq!(role_to_code_name(AgentRole::Coder), "coder");
        assert_eq!(role_to_code_name(AgentRole::Operator), "operator");
    }

    #[test]
    fn test_step_result_serde() {
        let result = StepResult {
            output: "Hello world".to_string(),
            tokens_used: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            artifacts: vec!["artifact-1".to_string()],
            tool_calls: vec![ToolCallRecord {
                tool_name: "memory".to_string(),
                input: "store key".to_string(),
                output: "ok".to_string(),
                approved: true,
            }],
        };

        let json = serde_json::to_string(&result).expect("serialize StepResult");
        let parsed: StepResult = serde_json::from_str(&json).expect("deserialize StepResult");

        assert_eq!(parsed.output, "Hello world");
        assert!(parsed.tokens_used.is_some());
        assert_eq!(parsed.tokens_used.unwrap().total_tokens, 150);
        assert_eq!(parsed.artifacts.len(), 1);
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].tool_name, "memory");
    }

    #[test]
    fn test_tool_call_record_serde() {
        let record = ToolCallRecord {
            tool_name: "shell".to_string(),
            input: "ls -la".to_string(),
            output: "file1 file2".to_string(),
            approved: false,
        };

        let json = serde_json::to_string(&record).expect("serialize");
        let parsed: ToolCallRecord = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.tool_name, "shell");
        assert!(!parsed.approved);
    }

    #[test]
    fn test_select_skill_report_writing() {
        let executor = make_executor();
        let agent = make_agent("Quill", AgentRole::Writer);
        let step = make_step("Write a report on Q4 earnings");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_some(), "should match report writing skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "report-writing");
    }

    #[test]
    fn test_select_skill_email_drafting() {
        let executor = make_executor();
        let agent = make_agent("Quill", AgentRole::Writer);
        let step = make_step("Draft an email to the team");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_some(), "should match email drafting skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "email-drafting");
    }

    #[test]
    fn test_select_skill_comparative_analysis() {
        let executor = make_executor();
        let agent = make_agent("Sage", AgentRole::Analyst);
        let step = make_step("Comparative analysis of cloud providers");

        let skill = executor.select_skill(&step, &agent);

        assert!(skill.is_some(), "should match comparative analysis skill");
        let skill = skill.unwrap();
        assert_eq!(skill.id, "comparative-analysis");
    }

    /// Helper that creates an executor with a shared event bus for integration testing.
    fn make_executor_with_bus() -> (AgentExecutor, Arc<EventBus>, Arc<AgentKernel>) {
        let llm_client = Arc::new(RwLock::new(
            LlmClient::new("https://gateway.day1.doctor"),
        ));
        let kernel = Arc::new(AgentKernel::new());
        let event_bus = Arc::new(EventBus::new(64));
        let cost_tracker = Arc::new(CostTracker::with_event_bus(event_bus.clone()));
        let permission_engine = Arc::new(PermissionEngine::new());
        let skill_registry = Arc::new(SkillRegistry::new());

        let executor = AgentExecutor::new(
            llm_client,
            kernel.clone(),
            event_bus.clone(),
            cost_tracker,
            permission_engine,
            skill_registry,
        );

        (executor, event_bus, kernel)
    }

    #[tokio::test]
    async fn test_execute_step_emits_state_changed_events() {
        // This test verifies that execute_step emits the correct sequence
        // of agent.state_changed events. The LLM call will fail (no real
        // gateway), but the first two state transitions should fire before
        // the error.
        let (executor, event_bus, kernel) = make_executor_with_bus();
        let agent = make_agent("Dr. Bob", AgentRole::Orchestrator);
        let step = make_step("Plan the project");

        // Register the agent in the kernel so FSM transitions work.
        kernel.register(agent.clone()).await;

        let mut rx = event_bus.subscribe();

        // execute_step will fail at the LLM call, but state transitions
        // (idle->working, working->thinking) should have emitted first.
        let _result = executor.execute_step(&step, &agent).await;

        // Collect all events that were emitted.
        let mut events = Vec::new();
        while let Ok(evt) = rx.try_recv() {
            events.push(evt);
        }

        // We expect at least 2 state_changed events before the LLM call fails:
        // 1. idle -> working (TaskAssign)
        // 2. working -> thinking (LlmCallStart)
        assert!(
            events.len() >= 2,
            "expected at least 2 events, got {}",
            events.len()
        );

        // All events should have the correct agent_id.
        for evt in &events {
            assert_eq!(evt.agent_id, agent.id, "event agent_id mismatch");
        }

        // First event: idle -> working.
        match &events[0].event_type {
            EventType::AgentStateChanged { from, to } => {
                assert_eq!(from, "idle", "first transition 'from' should be idle");
                assert_eq!(to, "working", "first transition 'to' should be working");
            }
            other => panic!("expected AgentStateChanged, got {:?}", other),
        }

        // Second event: working -> thinking.
        match &events[1].event_type {
            EventType::AgentStateChanged { from, to } => {
                assert_eq!(from, "working", "second transition 'from' should be working");
                assert_eq!(to, "thinking", "second transition 'to' should be thinking");
            }
            other => panic!("expected AgentStateChanged, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_tool_started_publishes_event() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_tool_started("agent-1", "shell", "ls -la")
            .await;

        let evt = rx.try_recv().expect("should receive tool.started event");
        assert_eq!(evt.agent_id, "agent-1");
        match &evt.event_type {
            EventType::ToolStarted { tool_name, params } => {
                assert_eq!(tool_name, "shell");
                assert_eq!(params["input"], "ls -la");
            }
            other => panic!("expected ToolStarted, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_tool_finished_publishes_event() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_tool_finished("agent-1", "shell", "file1 file2", 42)
            .await;

        let evt = rx.try_recv().expect("should receive tool.finished event");
        assert_eq!(evt.agent_id, "agent-1");
        match &evt.event_type {
            EventType::ToolFinished {
                tool_name,
                result,
                duration_ms,
            } => {
                assert_eq!(tool_name, "shell");
                assert_eq!(result["output"], "file1 file2");
                assert_eq!(*duration_ms, 42);
            }
            other => panic!("expected ToolFinished, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_artifact_created_publishes_event() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_artifact_created("agent-1", "task-123", "output/report.md")
            .await;

        let evt = rx
            .try_recv()
            .expect("should receive artifact.created event");
        assert_eq!(evt.agent_id, "agent-1");
        match &evt.event_type {
            EventType::ArtifactCreated {
                task_id,
                artifact_type,
                path,
            } => {
                assert_eq!(task_id, "task-123");
                assert_eq!(artifact_type, "document");
                assert_eq!(path, "output/report.md");
            }
            other => panic!("expected ArtifactCreated, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_artifact_created_infers_code_type() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_artifact_created("agent-1", "task-123", "src/main.rs")
            .await;

        let evt = rx.try_recv().expect("should receive event");
        match &evt.event_type {
            EventType::ArtifactCreated { artifact_type, .. } => {
                assert_eq!(artifact_type, "code");
            }
            other => panic!("expected ArtifactCreated, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_artifact_created_infers_image_type() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_artifact_created("agent-1", "task-123", "assets/chart.png")
            .await;

        let evt = rx.try_recv().expect("should receive event");
        match &evt.event_type {
            EventType::ArtifactCreated { artifact_type, .. } => {
                assert_eq!(artifact_type, "image");
            }
            other => panic!("expected ArtifactCreated, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_emit_artifact_created_infers_data_type() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_artifact_created("agent-1", "task-123", "output/results.json")
            .await;

        let evt = rx.try_recv().expect("should receive event");
        match &evt.event_type {
            EventType::ArtifactCreated { artifact_type, .. } => {
                assert_eq!(artifact_type, "data");
            }
            other => panic!("expected ArtifactCreated, got {:?}", other),
        }
    }
}
