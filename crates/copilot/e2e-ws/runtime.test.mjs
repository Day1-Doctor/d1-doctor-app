// Runtime E2E tests (5 tests)
//
// Validates that the IPC server starts correctly, agents are registered,
// health endpoint works, and agent models match expected configuration.

import {
  assertEqual,
  assertOk,
  assertGreaterThan,
  assertLength,
  assertMatch,
  assertType,
} from "./assert.mjs";

/**
 * Test 1: IPC server health endpoint responds with OK status.
 */
export async function test_health_endpoint(client) {
  const data = await client.get("/health");
  assertOk(data, "health endpoint should return data");
  assertEqual(data.status, "ok", "health status should be ok");
  assertOk(data.version, "health should include version");
}

/**
 * Test 2: Six agents are registered at startup (per PRD spec).
 * Dr. Bob (orchestrator), and 5 specialists.
 */
export async function test_six_agents_registered(client) {
  const data = await client.get("/api/v1/agents");
  assertOk(data.agents, "response should have agents array");
  assertLength(data.agents, 6, "should have exactly 6 agents");
}

/**
 * Test 3: All agents start in idle state.
 */
export async function test_all_agents_idle(client) {
  const data = await client.get("/api/v1/agents");
  assertOk(data.agents, "response should have agents array");
  for (const agent of data.agents) {
    assertEqual(
      agent.status,
      "idle",
      `agent "${agent.name}" should be idle at startup`
    );
  }
}

/**
 * Test 4: Agent models match the PRD-specified configuration.
 * At minimum, each agent should have a valid name, role, and trust_score.
 */
export async function test_agent_models_match_prd(client) {
  const data = await client.get("/api/v1/agents");
  assertOk(data.agents, "response should have agents array");

  const expectedRoles = new Set([
    "orchestrator",
    "researcher",
    "coder",
    "reviewer",
    "writer",
    "analyst",
    "operator",
    "designer",
    "monitor",
    "planner",
  ]);

  for (const agent of data.agents) {
    assertOk(agent.id, `agent should have an id`);
    assertOk(agent.name, `agent should have a name`);
    assertOk(agent.role, `agent "${agent.name}" should have a role`);
    assertOk(
      expectedRoles.has(agent.role),
      `agent "${agent.name}" role "${agent.role}" should be a known role`
    );
    assertType(
      agent.trust_score,
      "number",
      `agent "${agent.name}" trust_score should be a number`
    );
    assertGreaterThan(
      agent.trust_score,
      0,
      `agent "${agent.name}" trust_score should be > 0`
    );
  }
}

/**
 * Test 5: IPC server responds to the status endpoint (alias for health).
 * Also validates the version string format.
 */
export async function test_ipc_server_starts(client) {
  const data = await client.get("/health");
  assertOk(data, "server should respond to /health");
  assertEqual(data.status, "ok", "status should be ok");
  assertOk(
    data.version.startsWith("3."),
    "version should start with 3.x (v3.0.0-alpha)"
  );
}
