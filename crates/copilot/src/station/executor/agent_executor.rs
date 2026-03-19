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
use crate::station::permissions::risk::classify_risk;
use crate::station::permissions::RiskLevel;
use crate::station::runtime::presets::{builtin_presets, AgentPreset};
use crate::station::skills::executor::SkillExecutor;
use crate::station::skills::skill_registry::SkillRegistry;
use crate::station::skills::skill_types::SkillDefinition;
use crate::station::tasks::task_types::TaskSpec;

use super::audit::{AuditWriter, LlmCallAudit, ToolExecAudit};
use super::tool_dispatch::ToolDispatcher;

/// Maximum number of tool-use rounds per step.
///
/// The LLM may request multiple sequential tool calls; this limit prevents
/// infinite loops if the model never emits a final text answer.
const MAX_TOOL_ROUNDS: usize = 10;

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
/// calls, tool execution loop, cost tracking, and event emission.
pub struct AgentExecutor {
    llm_client: Arc<RwLock<LlmClient>>,
    kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
    cost_tracker: Arc<CostTracker>,
    permission_engine: Arc<PermissionEngine>,
    /// Retained for direct registry access; skill selection is delegated to
    /// `skill_executor`.
    #[allow(dead_code)]
    skill_registry: Arc<SkillRegistry>,
    skill_executor: SkillExecutor,
    tool_dispatcher: ToolDispatcher,
    audit_writer: AuditWriter,
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
        let tool_dispatcher = ToolDispatcher::with_cwd();
        Self {
            llm_client,
            kernel,
            event_bus,
            cost_tracker,
            permission_engine,
            skill_registry,
            skill_executor,
            tool_dispatcher,
            audit_writer: AuditWriter::noop(),
        }
    }

    /// Create a new executor with an [`AuditWriter`] for SQLite audit trails.
    pub fn with_audit(
        llm_client: Arc<RwLock<LlmClient>>,
        kernel: Arc<AgentKernel>,
        event_bus: Arc<EventBus>,
        cost_tracker: Arc<CostTracker>,
        permission_engine: Arc<PermissionEngine>,
        skill_registry: Arc<SkillRegistry>,
        audit_writer: AuditWriter,
    ) -> Self {
        let skill_executor = SkillExecutor::new(skill_registry.clone());
        let tool_dispatcher = ToolDispatcher::with_cwd();
        Self {
            llm_client,
            kernel,
            event_bus,
            cost_tracker,
            permission_engine,
            skill_registry,
            skill_executor,
            tool_dispatcher,
            audit_writer,
        }
    }

    /// Execute a single task step using the given agent.
    ///
    /// This drives the full lifecycle:
    /// 1. Transition agent to Working (idle -> working)
    /// 2. Select a matching skill from the registry
    /// 3. Compose a system prompt from agent persona + skill + step context
    /// 4. Transition to Thinking (working -> thinking)
    /// 5. Call the LLM gateway in a loop:
    ///    - If the LLM returns tool_calls, execute each tool (with permission
    ///      checks), send results back, and loop.
    ///    - If the LLM returns a final text answer, break.
    ///    - Safety limit: max `MAX_TOOL_ROUNDS` iterations.
    /// 6. Track cost (accumulated across all rounds)
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

        // 5. Build initial messages and enter the tool-use loop.
        let model = self.get_agent_model(agent);
        let tool_defs = ToolDispatcher::tool_definitions();

        let mut messages = vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: step.title.clone(),
            },
        ];

        let mut all_tool_calls: Vec<ToolCallRecord> = Vec::new();
        let mut total_prompt_tokens: u32 = 0;
        let mut total_completion_tokens: u32 = 0;
        let mut total_total_tokens: u32 = 0;
        let mut final_output = String::new();
        let mut has_usage = false;

        for round in 0..MAX_TOOL_ROUNDS {
            let request = ChatRequest {
                model: model.clone(),
                messages: messages.clone(),
                max_tokens: Some(4096),
                temperature: Some(0.7),
                stream: false,
                tools: Some(tool_defs.clone()),
            };

            let response = self
                .llm_client
                .read()
                .await
                .chat(request, &agent.name)
                .await?;

            // Accumulate token usage across all rounds.
            if let Some(ref usage) = response.usage {
                has_usage = true;
                total_prompt_tokens += usage.prompt_tokens;
                total_completion_tokens += usage.completion_tokens;
                total_total_tokens += usage.total_tokens;
                self.cost_tracker
                    .record_usage(
                        &agent.id,
                        "gateway",
                        &model,
                        usage.prompt_tokens as u64,
                        usage.completion_tokens as u64,
                    )
                    .await;

                // Audit: record each LLM round.
                self.audit_writer.record_llm_call(LlmCallAudit {
                    agent_id: agent.id.clone(),
                    task_id: Some(step.id.clone()),
                    model: model.clone(),
                    prompt_tokens: usage.prompt_tokens,
                    completion_tokens: usage.completion_tokens,
                    cost_dd: None, // Cost is tracked by CostTracker separately.
                });
            }

            // Extract the first choice.
            let choice = match response.choices.first() {
                Some(c) => c,
                None => {
                    final_output = String::new();
                    break;
                }
            };

            let finish_reason = choice
                .finish_reason
                .as_deref()
                .unwrap_or("stop");

            // Check if this response contains tool calls.
            let response_tool_calls = choice.message.tool_calls.as_ref();
            let has_tool_calls = response_tool_calls
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);

            if !has_tool_calls || finish_reason == "stop" {
                // Final text response -- exit the loop.
                final_output = choice.message.content.clone();
                break;
            }

            // The LLM wants to call tools. Append the assistant message
            // (with tool_calls) to the conversation so the model can see
            // its own request when we send tool results back.
            //
            // We serialise the tool_calls into the assistant content as a
            // JSON marker so the round-trip through ChatMessage (which only
            // has role+content) preserves the information.
            let assistant_content = if choice.message.content.is_empty() {
                // Build a synthetic assistant message containing the tool calls
                // so the next request includes context.
                serde_json::to_string(response_tool_calls.unwrap())
                    .unwrap_or_default()
            } else {
                choice.message.content.clone()
            };
            messages.push(ChatMessage {
                role: "assistant".to_string(),
                content: assistant_content,
            });

            // Execute each tool call.
            let tool_calls = response_tool_calls.unwrap();
            for tc in tool_calls {
                let tool_name = &tc.function.name;
                let arguments: serde_json::Value =
                    serde_json::from_str(&tc.function.arguments)
                        .unwrap_or(serde_json::Value::Null);

                // Permission check.
                let risk = classify_risk(tool_name, &arguments);
                let approved = match risk {
                    RiskLevel::Low => true,
                    RiskLevel::Medium => {
                        // Check permission engine; auto-approve for now with
                        // a log message for medium risk. The full approval UI
                        // flow will be wired in a follow-up.
                        let check = self
                            .permission_engine
                            .check_permission(
                                &agent.id,
                                &agent.name,
                                tool_name,
                                &arguments,
                                &format!("Step {}: {}", step.step_index.unwrap_or(0), step.title),
                            )
                            .await;
                        match check {
                            Ok(()) => true,
                            Err(approval_req) => {
                                // TODO(D1D-272): Wire to approval UI.
                                // For now emit event and auto-approve.
                                self.emit_approval_requested(
                                    &agent.id,
                                    tool_name,
                                    &format!("{:?}", approval_req.risk_level),
                                    &approval_req.context,
                                )
                                .await;
                                true
                            }
                        }
                    }
                    RiskLevel::High | RiskLevel::Critical => {
                        // Emit approval event; auto-approve for now.
                        // TODO(D1D-272): Block until user responds.
                        self.emit_approval_requested(
                            &agent.id,
                            tool_name,
                            &format!("{risk:?}"),
                            &format!("Step {}: {}", step.step_index.unwrap_or(0), step.title),
                        )
                        .await;
                        true
                    }
                };

                // Emit tool.started event.
                self.emit_tool_started(
                    &agent.id,
                    tool_name,
                    &tc.function.arguments,
                )
                .await;

                // Execute the tool.
                let exec_result = if approved {
                    self.tool_dispatcher.execute(tool_name, arguments.clone()).await
                } else {
                    super::tool_dispatch::ToolExecResult {
                        output: serde_json::json!({
                            "error": "Tool call rejected by permission engine"
                        }),
                        success: false,
                        duration_ms: 0,
                    }
                };

                // Emit tool.finished event.
                let output_str = serde_json::to_string(&exec_result.output)
                    .unwrap_or_else(|_| exec_result.output.to_string());
                self.emit_tool_finished(
                    &agent.id,
                    tool_name,
                    &output_str,
                    exec_result.duration_ms,
                )
                .await;

                // Record the tool call.
                all_tool_calls.push(ToolCallRecord {
                    tool_name: tool_name.clone(),
                    input: tc.function.arguments.clone(),
                    output: output_str.clone(),
                    approved,
                });

                // Audit: record the tool execution to SQLite.
                self.audit_writer.record_tool_execution(ToolExecAudit {
                    agent_id: agent.id.clone(),
                    task_id: Some(step.id.clone()),
                    tool_name: tool_name.clone(),
                    input: Some(tc.function.arguments.clone()),
                    output: Some(output_str.clone()),
                    approved,
                    duration_ms: exec_result.duration_ms,
                });

                // Add the tool result to messages so the LLM can see it.
                messages.push(ChatMessage {
                    role: "tool".to_string(),
                    content: output_str,
                });
            }

            // Safety: if we've hit the last allowed round, use whatever
            // content we have so far.
            if round == MAX_TOOL_ROUNDS - 1 {
                tracing::warn!(
                    agent_id = %agent.id,
                    "tool loop hit max rounds ({MAX_TOOL_ROUNDS}), returning last content"
                );
                final_output = choice.message.content.clone();
            }
        }

        // 7. Transition back to Working: thinking -> working.
        let (from, to) = self
            .kernel
            .apply_trigger(&agent.id, Trigger::LlmCallEnd)
            .await?;
        self.emit_state_changed(&agent.id, from.display_name(), to.display_name())
            .await;

        // 8. Emit artifact.created events for any file-producing tool calls.
        let artifacts: Vec<String> = all_tool_calls
            .iter()
            .filter_map(|tc| {
                if tc.tool_name == "write_file"
                    || tc.tool_name == "filesystem.write"
                    || tc.tool_name == "create_markdown"
                {
                    // Try to extract the path from the tool output.
                    let v: serde_json::Value =
                        serde_json::from_str(&tc.output).unwrap_or_default();
                    v.get("path")
                        .and_then(|p| p.as_str())
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();

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
                result: serde_json::json!({ "output": &final_output }),
            },
        };
        self.event_bus.publish(step_completed_event).await;

        let tokens_used = if has_usage {
            Some(Usage {
                prompt_tokens: total_prompt_tokens,
                completion_tokens: total_completion_tokens,
                total_tokens: total_total_tokens,
            })
        } else {
            None
        };

        Ok(StepResult {
            output: final_output,
            tokens_used,
            artifacts,
            tool_calls: all_tool_calls,
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

    /// Emit an `approval.requested` event via the event bus.
    async fn emit_approval_requested(
        &self,
        agent_id: &str,
        action: &str,
        risk_level: &str,
        context: &str,
    ) {
        let event = AgentEvent {
            id: Uuid::new_v4().to_string(),
            agent_id: agent_id.to_string(),
            timestamp: Utc::now(),
            event_type: EventType::ApprovalRequested {
                action: action.to_string(),
                risk_level: risk_level.to_string(),
                context: context.to_string(),
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

    // -----------------------------------------------------------------------
    // Tool execution loop tests (D1D-272)
    // -----------------------------------------------------------------------

    #[test]
    fn test_max_tool_rounds_constant() {
        // Verify the safety limit is set to a reasonable value.
        assert_eq!(MAX_TOOL_ROUNDS, 10);
    }

    #[test]
    fn test_tool_dispatcher_available_in_executor() {
        // Verify the executor has a tool dispatcher.
        let executor = make_executor();
        let tools = executor.tool_dispatcher.supported_tools();
        assert!(!tools.is_empty(), "tool dispatcher should have tools");
        assert!(
            tools.contains(&"read_file"),
            "should support read_file"
        );
        assert!(
            tools.contains(&"write_file"),
            "should support write_file"
        );
        assert!(
            tools.contains(&"web_search"),
            "should support web_search"
        );
    }

    #[test]
    fn test_tool_definitions_provided_to_llm() {
        let defs = ToolDispatcher::tool_definitions();
        assert!(!defs.is_empty(), "should have tool definitions");

        // Each definition should have the expected OpenAI function format.
        for def in &defs {
            assert_eq!(def["type"], "function");
            assert!(def["function"]["name"].is_string());
            assert!(def["function"]["description"].is_string());
            assert!(def["function"]["parameters"].is_object());
        }
    }

    #[tokio::test]
    async fn test_emit_approval_requested_publishes_event() {
        let (executor, event_bus, _kernel) = make_executor_with_bus();
        let mut rx = event_bus.subscribe();

        executor
            .emit_approval_requested("agent-1", "shell", "High", "running command")
            .await;

        let evt = rx.try_recv().expect("should receive approval.requested event");
        assert_eq!(evt.agent_id, "agent-1");
        match &evt.event_type {
            EventType::ApprovalRequested {
                action,
                risk_level,
                context,
            } => {
                assert_eq!(action, "shell");
                assert_eq!(risk_level, "High");
                assert_eq!(context, "running command");
            }
            other => panic!("expected ApprovalRequested, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_tool_dispatcher_execute_roundtrip() {
        // Test that the executor's tool dispatcher can run a real tool.
        let dir = std::env::temp_dir().join("d1d-executor-dispatch-test");
        std::fs::create_dir_all(&dir).ok();
        let dispatcher = ToolDispatcher::new(dir.clone());

        let file_path = dir.join("executor_test.txt");
        let path_str = file_path.to_str().unwrap();

        // Write via dispatcher.
        let write_result = dispatcher
            .execute(
                "write_file",
                serde_json::json!({ "path": path_str, "content": "executor test" }),
            )
            .await;
        assert!(write_result.success, "write should succeed");

        // Read back.
        let read_result = dispatcher
            .execute("read_file", serde_json::json!({ "path": path_str }))
            .await;
        assert!(read_result.success, "read should succeed");
        assert_eq!(read_result.output["content"], "executor test");

        // Cleanup.
        std::fs::remove_file(&file_path).ok();
        std::fs::remove_dir(&dir).ok();
    }

    #[tokio::test]
    async fn test_tool_dispatcher_unknown_tool_graceful_error() {
        let dispatcher = ToolDispatcher::with_cwd();
        let result = dispatcher
            .execute("nonexistent", serde_json::json!({}))
            .await;
        assert!(!result.success, "unknown tool should fail gracefully");
        assert!(
            result.output["error"]
                .as_str()
                .unwrap()
                .contains("unknown tool"),
            "error should mention unknown tool"
        );
    }

    #[test]
    fn test_tool_call_record_includes_approved_field() {
        let record = ToolCallRecord {
            tool_name: "read_file".to_string(),
            input: r#"{"path":"test.txt"}"#.to_string(),
            output: r#"{"content":"hello"}"#.to_string(),
            approved: true,
        };
        let json = serde_json::to_value(&record).unwrap();
        assert_eq!(json["approved"], true);

        let rejected = ToolCallRecord {
            tool_name: "shell".to_string(),
            input: "rm -rf /".to_string(),
            output: "rejected".to_string(),
            approved: false,
        };
        let json = serde_json::to_value(&rejected).unwrap();
        assert_eq!(json["approved"], false);
    }

    #[test]
    fn test_chat_request_includes_tools_field() {
        let defs = ToolDispatcher::tool_definitions();
        let request = ChatRequest {
            model: "claude-sonnet-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: false,
            tools: Some(defs.clone()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json["tools"].is_array(), "tools should be serialized");
        assert_eq!(json["tools"].as_array().unwrap().len(), defs.len());
    }

    #[test]
    fn test_chat_request_without_tools() {
        let request = ChatRequest {
            model: "claude-sonnet-4".to_string(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "hello".to_string(),
            }],
            max_tokens: Some(4096),
            temperature: Some(0.7),
            stream: false,
            tools: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        // tools: None should be omitted from serialization.
        assert!(
            json.get("tools").is_none(),
            "tools should be omitted when None"
        );
    }
}
