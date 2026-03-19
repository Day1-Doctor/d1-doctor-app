# Day1 Copilot v3.0 — Full Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Complete ALL remaining gaps between the PRD v3.0 and the current implementation — make agents actually work, tools execute, billing correct, and the full pipeline functional end-to-end.

**Architecture:** The Copilot desktop app (Tauri v2 + React 19 + Rust Station Runtime) communicates with the Day1 Gateway (`gateway.day1.doctor/dr-agent/v1/*`) for LLM calls and DD billing. Agents execute tasks by composing prompts from skills, calling the gateway, handling tool_use responses, and emitting events that drive the UI.

**Tech Stack:** Rust (Station Runtime, Tauri commands), TypeScript/React (UI), Python/FastAPI (Gateway), Supabase (auth + profile + vault), SQLite (local state)

---

## Phase 0: Critical Fixes (billing, pricing, TS errors)

### Task 0.1: Fix pricing to match PRD §19

**Files:**
- Modify: `crates/copilot/src/station/billing/tiers.rs`
- Modify: `crates/copilot/ui/src/stores/billingStore.ts`
- Modify: `crates/copilot/ui/src/components/shared/PricingPage.tsx`
- Modify: `crates/copilot/ui/src/components/valley/ValleyRenderer.ts`

**Changes:**
| Tier | Field | Current (WRONG) | PRD (CORRECT) |
|------|-------|-----------------|---------------|
| Free Man | monthly_dd | 100 | 0 (pay-per-use) |
| Free Man | max_agents | 1 | 1 (correct) |
| Mini Shop | price_monthly | $19 | $30 |
| Mini Shop | price_annual_mo | $15.83 | $25 |
| Mini Shop | monthly_dd | 1,000 | 3,500 |
| Mini Shop | max_agents | 3 | 3 (correct) |
| Rocket Inc | price_monthly | $49 | $100 |
| Rocket Inc | price_annual_mo | $40.83 | $80 |
| Rocket Inc | monthly_dd | 5,000 | 15,000 |
| Rocket Inc | max_agents | 8 | 10 |

Update ALL files to match PRD values. Rocket Inc back to 10 agents (user changed to 8 earlier, but PRD says 10 — confirm with user).

### Task 0.2: Fix remaining TS compilation errors

**Files:** `crates/copilot/ui/src/components/office/OfficeRenderer.ts`

Find all `drawPixelCharacter` calls with wrong argument count and fix to match the SpriteRenderer.ts signature. Run `npx tsc --noEmit` until 0 errors.

### Task 0.3: Fix auth to OAuth-only (remove API key from backend)

**Files:**
- Modify: `crates/copilot/src/lib.rs` — remove `store_api_key`, `get_stored_api_key` commands
- Modify: `crates/copilot/ui/src/stores/authStore.ts` — replace API key flow with OAuth session
- Modify: `crates/copilot/ui/src/components/shared/OnboardingWizard.tsx` — remove API key step, replace with "Sign in with Day1 Account"

---

## Phase 1: Gateway + Auth + Profile Schema

### Task 1.1: Add subscription tier columns to Supabase profiles

**Files:**
- Create: `d1-doctor-platform/supabase/migrations/20260319000001_add_subscription_tier.sql`

```sql
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS subscription_tier TEXT DEFAULT 'free';
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS tier_max_agents INTEGER DEFAULT 1;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS tier_monthly_dd INTEGER DEFAULT 0;
ALTER TABLE profiles ADD COLUMN IF NOT EXISTS tier_expires_at TIMESTAMPTZ;
```

### Task 1.2: Update gateway balance endpoint to read real tier

**Files:** `d1-doctor-platform/gateway/api/dr_agent.py`

Replace hardcoded `"free"` and `1` with real profile column reads:
```python
profile = await supabase.table("profiles").select("dd_balance, subscription_tier, tier_max_agents").eq("id", user_id).single()
```

### Task 1.3: Deploy dr-agent endpoints to production

**Steps:**
1. Merge platform migration + code to main
2. Run `supabase db push` for the migration
3. Deploy gateway via CD pipeline
4. Verify: `curl https://gateway.day1.doctor/dr-agent/v1/balance -H "Authorization: Bearer ..." `

### Task 1.4: Implement OAuth deep link callback in Copilot

**Files:**
- Modify: `crates/copilot/tauri.conf.json` — register `day1copilot://` URL scheme
- Create: `crates/copilot/src/auth.rs` — handle OAuth callback, extract JWT
- Modify: `crates/copilot/ui/src/stores/authStore.ts` — add `authenticateWithJwt(token)`
- Modify: `crates/copilot/ui/src/components/shared/AuthWall.tsx` — handle callback

### Task 1.5: Wire JWT token storage + refresh

**Files:**
- Modify: `crates/copilot/src/lib.rs` — Tauri commands for JWT storage/retrieval
- Modify: `crates/copilot/ui/src/stores/authStore.ts` — JWT refresh interval (every 50 min)

---

## Phase 2: Agent Execution Loop (THE CORE)

### Task 2.1: Create the execution engine

**Files:**
- Create: `crates/copilot/src/station/executor/mod.rs`
- Create: `crates/copilot/src/station/executor/agent_executor.rs`
- Create: `crates/copilot/src/station/executor/step_runner.rs`

The executor takes a task step + agent + skill, composes a prompt, calls the gateway LLM, parses the response, handles tool_use calls, and loops until done.

```rust
pub struct AgentExecutor {
    llm_client: Arc<RwLock<LlmClient>>,
    kernel: Arc<AgentKernel>,
    event_bus: Arc<EventBus>,
    cost_tracker: Arc<CostTracker>,
    permission_engine: Arc<PermissionEngine>,
    skill_registry: Arc<SkillRegistry>,
}

impl AgentExecutor {
    /// Execute a single task step with an assigned agent
    pub async fn execute_step(
        &self,
        step: &TaskSpec,
        agent: &AgentDescriptor,
    ) -> Result<StepResult, String> {
        // 1. Transition agent to Working
        self.kernel.apply_trigger(&agent.id, Trigger::TaskAssign).await?;
        self.emit_state_changed(&agent.id, "idle", "working").await;

        // 2. Select skill based on step type
        let skill = self.select_skill(&step, &agent);

        // 3. Compose system prompt (agent persona + skill instructions + context)
        let system_prompt = self.compose_prompt(&agent, &skill, &step);

        // 4. Call gateway LLM
        self.kernel.apply_trigger(&agent.id, Trigger::LlmCallStart).await?;
        self.emit_state_changed(&agent.id, "working", "thinking").await;

        let response = self.llm_client.read().await
            .chat(ChatRequest {
                model: self.get_agent_model(&agent),
                messages: vec![
                    ChatMessage { role: "system".into(), content: system_prompt },
                    ChatMessage { role: "user".into(), content: step.title.clone() },
                ],
                max_tokens: Some(4096),
                temperature: Some(0.7),
                stream: false,
            }, &agent.name).await?;

        // 5. Track DD cost
        if let Some(usage) = &response.usage {
            self.cost_tracker.record_usage(
                &agent.id, "anthropic",
                &self.get_agent_model(&agent),
                usage.prompt_tokens, usage.completion_tokens,
            ).await;
        }

        // 6. Transition back to Working
        self.kernel.apply_trigger(&agent.id, Trigger::LlmCallEnd).await?;
        self.emit_state_changed(&agent.id, "thinking", "working").await;

        // 7. Parse response — check for tool_use (future: handle tool calls)
        let content = response.choices.first()
            .map(|c| c.message.content.clone())
            .unwrap_or_default();

        // 8. Complete the step
        self.kernel.apply_trigger(&agent.id, Trigger::TaskComplete).await?;
        self.emit_state_changed(&agent.id, "working", "idle").await;

        Ok(StepResult { output: content, tokens_used: response.usage })
    }
}
```

### Task 2.2: Create the task orchestrator

**Files:**
- Create: `crates/copilot/src/station/executor/orchestrator.rs`

Orchestrates the full task pipeline:
1. Decompose task (via LlmDecomposer → gateway)
2. Route steps to agents (via TaskRouter)
3. Execute steps sequentially or in parallel (via AgentExecutor)
4. Handle hand-offs (output of step N → input of step N+1)
5. Emit task.step_completed events
6. Complete parent task when all steps done

### Task 2.3: Wire executor into Tauri create_task command

**Files:** `crates/copilot/src/lib.rs`

When `create_task` is called:
1. Decompose (LLM or keyword fallback)
2. Route
3. Spawn execution in background tokio task
4. Return immediately with task ID
5. Execution emits events → UI updates via WS

### Task 2.4: Wire Event Bus emissions at each execution step

**Files:** `crates/copilot/src/station/executor/agent_executor.rs`

Emit proper events at each point:
- `agent.state_changed` — on every FSM transition
- `tool.started` / `tool.finished` — when tool calls happen
- `task.step_completed` — when a step finishes
- `cost.updated` — after each LLM call with token counts
- `artifact.created` — when step produces output files

### Task 2.5: Wire DD deduction tracking

**Files:**
- Modify: `crates/copilot/src/station/costs/cost_tracker.rs` — emit cost.updated events
- Modify: `crates/copilot/ui/src/hooks/useEventStream.ts` — handle cost.updated → update costStore
- Modify: `crates/copilot/ui/src/stores/costStore.ts` — update balance from events

### Task 2.6: Add per-agent model assignments

**Files:**
- Modify: `crates/copilot/src/station/runtime/presets.rs` — add `default_model` field
- Modify: `crates/copilot/src/station/executor/agent_executor.rs` — use agent's model

Model assignments per PRD §18.3:
- Dr. Bob, Sage, Quill, Pixel → `claude-sonnet-4` (Medium tier, 5 DD/1M tokens)
- Scout, Atlas → `claude-haiku-4-5` (Light tier, 1 DD/1M tokens)

---

## Phase 3: MCP Tool Implementations

### Task 3.1: Implement mcp-filesystem (real)

**Files:** `crates/copilot/src/station/tools/filesystem.rs`

Real implementations:
- `read(path)` → `std::fs::read_to_string`
- `write(path, content)` → `std::fs::write` (needs approval for MEDIUM risk)
- `glob(pattern)` → `glob` crate
- `list_dir(path)` → `std::fs::read_dir`

### Task 3.2: Implement mcp-web-search (real)

**Files:** `crates/copilot/src/station/tools/web_search.rs`

Use Brave Search API or Tavily:
- `search(query, limit)` → HTTP GET to search API → parse results
- Requires API key (stored in settings or env)

### Task 3.3: Implement mcp-web-fetch (real)

**Files:** `crates/copilot/src/station/tools/web_fetch.rs`

- `fetch_url(url)` → `reqwest::get(url)` → return HTML
- `extract_text(html)` → basic HTML → text extraction (strip tags)

### Task 3.4: Implement mcp-document (real)

**Files:** `crates/copilot/src/station/tools/document.rs`

- `create_markdown(content, path)` → write .md file to workspace
- Other formats (PDF/DOCX) remain stubs for now

### Task 3.5: Wire tool execution into agent executor

**Files:** `crates/copilot/src/station/executor/agent_executor.rs`

When LLM response contains `tool_use`:
1. Parse tool name + params from response
2. Check permission via PermissionEngine
3. If approved: execute tool, get result
4. If needs approval: emit `approval.requested` event, wait for user response
5. Send tool result back to LLM in next message
6. Loop until LLM gives final answer (no more tool_use)

### Task 3.6: Wire approval events to UI

**Files:**
- Modify: `crates/copilot/ui/src/hooks/useEventStream.ts` — handle `approval.requested` → add to approvalStore
- Modify: `crates/copilot/ui/src/stores/approvalStore.ts` — add from WS events
- Create: Tauri command `respond_approval(id, decision)` → send response back to PermissionEngine

### Task 3.7: Write audit trail to SQLite

**Files:**
- Modify: `crates/copilot/src/station/executor/agent_executor.rs`
- After each tool execution, write to `tool_executions` table
- After each LLM call, write to `session_costs` table

---

## Phase 4: Skill Execution Engine

### Task 4.1: Create skill executor

**Files:**
- Create: `crates/copilot/src/station/skills/executor.rs`

```rust
pub struct SkillExecutor {
    registry: Arc<SkillRegistry>,
    agent_executor: Arc<AgentExecutor>,
}

impl SkillExecutor {
    /// Select the best skill for a task step based on agent role + step keywords
    pub fn select_skill(&self, step: &PlannedStep, agent_role: &str) -> Option<&SkillDefinition> {
        // Match by role + keywords in step title
    }

    /// Execute a skill's multi-step workflow
    pub async fn execute_skill(
        &self, skill: &SkillDefinition, context: &str, agent: &AgentDescriptor,
    ) -> Result<String, String> {
        let mut accumulated_context = context.to_string();
        for skill_step in &skill.steps {
            let prompt = skill_step.prompt_template
                .replace("{{topic}}", &accumulated_context)
                .replace("{{sources}}", &accumulated_context)
                .replace("{{findings}}", &accumulated_context);
            // Execute via agent executor
            let result = self.agent_executor.execute_with_prompt(agent, &prompt).await?;
            accumulated_context = format!("{}\n\n{}", accumulated_context, result);
        }
        Ok(accumulated_context)
    }
}
```

### Task 4.2: Wire skills into task decomposition

When a task is decomposed and routed, match each step to an appropriate skill:
- "Research..." → Deep Research skill
- "Analyze..." → Comparative Analysis skill
- "Write report..." → Report Writing skill
- etc.

---

## Phase 5: Frontend ↔ Backend Bridge Completion

### Task 5.1: Wire real agent data to all UI stores

**Files:**
- Modify: `useEventStream.ts` — handle ALL 8 event types with real store updates:
  - `agent.state_changed` → agentStore.updateAgentStatus ✓ (done)
  - `token.stream` → costStore.updateSessionTokens (NEW)
  - `tool.started` → eventLogStore.addEvent (NEW)
  - `tool.finished` → eventLogStore.addEvent (NEW)
  - `approval.requested` → approvalStore.addApproval (NEW)
  - `artifact.created` → artifactStore.addArtifact (NEW)
  - `task.step_completed` → taskStore.updateStepStatus (NEW)
  - `cost.updated` → costStore.updateBalance + costStore.updateAgentCost (NEW)

### Task 5.2: Wire task creation from Plan Mode chat

**Files:**
- Modify: `RightPanel.tsx` ChatPanel — when user confirms plan:
  1. Call Tauri `create_task` with the description
  2. Task decomposes → creates subtasks → agents start executing
  3. TaskProgressBar in OfficeView updates from events
  4. Agent animations change in real-time

### Task 5.3: Wire real artifacts to Workspace view

**Files:**
- Modify: `WorkspaceView.tsx` — fetch real artifacts from Tauri `list_artifacts` command
- Modify: `artifactStore.ts` — add `fetchArtifacts()` from Tauri

### Task 5.4: Wire real metrics

**Files:**
- Modify: `MetricsView.tsx` — replace placeholder numbers with real data from costStore + taskStore
- Add Tauri command `get_metrics()` returning real totals

### Task 5.5: Wire approval response back to runtime

**Files:**
- Create: Tauri command `respond_approval(request_id, decision)`
- Modify: `ApprovalDialog.tsx` — call Tauri command on button click
- Modify: `PermissionEngine` — receive response and unblock waiting agent

---

## Phase 6: Persistence + Session Management

### Task 6.1: Persist tasks to SQLite

**Files:** `crates/copilot/src/station/tasks/task_engine.rs`

On every task state change, write to SQLite `tasks` table. On app launch, load pending tasks from DB.

### Task 6.2: Persist trust scores to SQLite

**Files:** `crates/copilot/src/station/permissions/approval.rs`

After each trust score update, write to `agents.trust_score`. On launch, load scores from DB.

### Task 6.3: Persist session costs to SQLite

**Files:** `crates/copilot/src/station/costs/cost_tracker.rs`

After each LLM call, write to `session_costs` table.

---

## Phase 7: Missing Interface Methods

### Task 7.1: Implement memory.link() and memory.summarize()

**Files:** `crates/copilot/src/station/memory/memory_store.rs`

- `link(id1, id2, relationship)` — create a `memory_links` table and insert
- `summarize(session_id)` — aggregate session memory entries and call LLM for summary

### Task 7.2: Implement provider.list()

**Files:** Create Tauri command `list_providers()` returning available models from gateway `/v1/models`

### Task 7.3: Implement explicit task.start() route

**Files:** `crates/copilot/src/station/server/handlers.rs`

Add `POST /api/v1/tasks/:id/start` that triggers the execution orchestrator.

---

## Phase 8: E2E Testing

### Task 8.1: Rust integration tests (target: 40+)

**Files:** `crates/copilot/tests/`

- `e2e_execution.rs` — test full pipeline: decompose → route → execute (mock LLM) → artifacts
- `e2e_permissions.rs` — test tool call → approval → resume
- `e2e_costs.rs` — test DD deduction flow
- `e2e_handoff.rs` — test step A → hand-off → step B → complete parent

### Task 8.2: Frontend integration tests

- Test Plan Mode → confirm → task created
- Test approval dialog → approve → agent continues
- Test view switching preserves context
- Test zoom control persistence

### Task 8.3: Gateway integration tests

- Test `/dr-agent/v1/decompose` returns valid plan
- Test `/dr-agent/v1/chat/completions` deducts DD
- Test balance endpoint returns real tier

---

## Phase 9: UI Polish Remaining

### Task 9.1: Fix agent animations to be more distinct

5 clearly distinct animations per PRD AC-1:
- idle: subtle bob
- thinking: thought bubble with "..."
- typing: arm movement + key particles
- executing: gear rotation
- error: red flash + shake

### Task 9.2: Wire approval bridge (backend → frontend)

Complete the path: PermissionEngine → EventBus → WS → approvalStore → ApprovalDialog

### Task 9.3: Implement "preview then auth" demo mode

Allow users to see the Valley and office without auth. Auth-gate only on task creation.

### Task 9.4: Fix MetricsView with real data

Replace all placeholder numbers with real store data.

### Task 9.5: Fix WorkspaceView memory section

Replace hardcoded mock entries with real memory store data via Tauri command.

---

## Priority Order (Critical Path)

1. **Phase 0** — Fix billing/pricing (10 min) + TS errors
2. **Phase 2** — Agent execution loop (THE CORE — everything depends on this)
3. **Phase 3** — MCP tools (agents need tools to be useful)
4. **Phase 5** — Frontend bridge (make UI reflect real execution)
5. **Phase 1** — Gateway deploy + auth (needed for production)
6. **Phase 4** — Skill engine (enriches agent behavior)
7. **Phase 6** — Persistence (data survives restart)
8. **Phase 7** — Missing interface methods
9. **Phase 8** — E2E tests (release gate)
10. **Phase 9** — UI polish

**Estimated total: 53 tasks across 9 phases**
