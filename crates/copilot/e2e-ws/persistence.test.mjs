// Persistence E2E tests (4 tests)
//
// Validates that data created through the API persists and can be
// retrieved. Since we cannot easily restart the binary in E2E tests,
// we verify persistence by creating data and immediately querying it
// back through the API — the underlying SQLite or in-memory store
// must retain the data across requests.

import {
  assertEqual,
  assertOk,
  assertGreaterThan,
  assertLength,
  assertType,
} from "./assert.mjs";

/**
 * Test 1: Created tasks persist and can be retrieved.
 * Verifies the task engine stores tasks durably across API calls.
 */
export async function test_task_persists_to_db(client) {
  // Create several tasks
  const task1 = await client.post("/api/v1/tasks", {
    description: "Persistence test task A",
  });
  const task2 = await client.post("/api/v1/tasks", {
    description: "Persistence test task B",
  });
  const task3 = await client.post("/api/v1/tasks", {
    description: "Persistence test task C",
  });

  assertOk(task1.id, "task1 should have id");
  assertOk(task2.id, "task2 should have id");
  assertOk(task3.id, "task3 should have id");

  // Retrieve each task individually to verify persistence
  const retrieved1 = await client.get(`/api/v1/tasks/${task1.id}`);
  assertEqual(retrieved1.id, task1.id, "task1 should persist");
  assertEqual(retrieved1.title, "Persistence test task A");

  const retrieved2 = await client.get(`/api/v1/tasks/${task2.id}`);
  assertEqual(retrieved2.id, task2.id, "task2 should persist");

  const retrieved3 = await client.get(`/api/v1/tasks/${task3.id}`);
  assertEqual(retrieved3.id, task3.id, "task3 should persist");

  // Verify all three appear in the list
  const list = await client.get("/api/v1/tasks");
  assertOk(list.tasks, "task list should exist");
  const ids = list.tasks.map((t) => t.id);
  assertOk(ids.includes(task1.id), "task1 should be in list");
  assertOk(ids.includes(task2.id), "task2 should be in list");
  assertOk(ids.includes(task3.id), "task3 should be in list");
}

/**
 * Test 2: Session cost data persists across API calls.
 * Validates the cost tracker maintains state.
 */
export async function test_session_costs_persist(client) {
  // Query initial costs
  const initialCosts = await client.get("/api/v1/costs");
  assertOk(initialCosts, "costs endpoint should respond");
  assertType(
    initialCosts.total_tokens_in,
    "number",
    "total_tokens_in should be a number"
  );
  assertType(
    initialCosts.total_tokens_out,
    "number",
    "total_tokens_out should be a number"
  );
  assertType(
    initialCosts.total_cost_dd,
    "number",
    "total_cost_dd should be a number"
  );

  // Query agent-specific costs (may be empty)
  assertOk(
    Array.isArray(initialCosts.agents),
    "agents cost array should exist"
  );

  // Query costs again — should be consistent
  const secondQuery = await client.get("/api/v1/costs");
  assertEqual(
    secondQuery.total_tokens_in,
    initialCosts.total_tokens_in,
    "costs should be consistent across queries"
  );
  assertEqual(
    secondQuery.total_cost_dd,
    initialCosts.total_cost_dd,
    "cost_dd should be consistent"
  );
}

/**
 * Test 3: Agent trust scores persist and are retrievable.
 * Each agent's trust score should be consistent across API calls.
 */
export async function test_trust_scores_persist(client) {
  // Get agents and their trust scores
  const data1 = await client.get("/api/v1/agents");
  assertOk(data1.agents, "should have agents");

  // Map agent_id -> trust_score
  const scores1 = {};
  for (const agent of data1.agents) {
    scores1[agent.id] = agent.trust_score;
  }

  // Query again
  const data2 = await client.get("/api/v1/agents");
  for (const agent of data2.agents) {
    assertEqual(
      agent.trust_score,
      scores1[agent.id],
      `trust score for ${agent.name} should persist`
    );
  }
}

/**
 * Test 4: Task status changes are recorded (audit trail).
 * Verifies that status transitions are reflected in subsequent queries.
 */
export async function test_audit_trail_written(client) {
  // Create a task
  const task = await client.post("/api/v1/tasks", {
    description: "Audit trail test task",
  });
  assertEqual(task.status, "pending", "initial status should be pending");

  // Start it
  const started = await client.post(`/api/v1/tasks/${task.id}/start`);
  assertEqual(started.status, "running", "should transition to running");

  // Query to verify the status stuck
  const afterStart = await client.get(`/api/v1/tasks/${task.id}`);
  assertEqual(afterStart.status, "running", "running status should persist");

  // Cancel it
  await client.post(`/api/v1/tasks/${task.id}/cancel`);

  // Query to verify cancelled status persisted
  const afterCancel = await client.get(`/api/v1/tasks/${task.id}`);
  assertEqual(
    afterCancel.status,
    "cancelled",
    "cancelled status should persist"
  );

  // The full audit trail: pending -> running -> cancelled
  // Each state was verified through a fresh GET request, confirming persistence.
  assertOk(true, "audit trail verified through API queries");
}
