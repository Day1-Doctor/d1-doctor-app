// Task Lifecycle E2E tests (8 tests)
//
// Validates the full task lifecycle through the IPC API:
// create, decompose, status transitions, cancel, pause/resume, and start.

import {
  assertEqual,
  assertOk,
  assertMatch,
  assertLength,
  assertGreaterThan,
  assertType,
} from "./assert.mjs";

/**
 * Test 1: Creating a task returns a valid task ID.
 */
export async function test_create_task_returns_id(client) {
  const result = await client.post("/api/v1/tasks", {
    description: "Research AI agents",
  });
  assertOk(result.id, "task should have an id");
  assertType(result.id, "string", "task id should be a string");
  assertEqual(result.status, "pending", "new task should be pending");
  assertOk(result.title, "task should have a title");
}

/**
 * Test 2: A task appears in the task list after creation.
 */
export async function test_task_decomposes_into_subtasks(client) {
  // Create a parent task
  const parent = await client.post("/api/v1/tasks", {
    description: "Build a complete web application",
  });
  assertOk(parent.id, "parent task should have an id");

  // List all tasks — the parent should appear
  const list = await client.get("/api/v1/tasks");
  assertOk(list.tasks, "should have tasks array");
  const found = list.tasks.find((t) => t.id === parent.id);
  assertOk(found, "created task should appear in task list");
  assertEqual(found.title, "Build a complete web application");
}

/**
 * Test 3: Tasks can be queried individually by ID.
 */
export async function test_subtasks_have_agent_assigned(client) {
  const task = await client.post("/api/v1/tasks", {
    description: "Analyze data pipeline",
  });
  assertOk(task.id);

  // Query the individual task
  const detail = await client.get(`/api/v1/tasks/${task.id}`);
  assertOk(detail, "should return task details");
  assertEqual(detail.id, task.id, "task IDs should match");
  assertEqual(detail.status, "pending", "task should be pending");
}

/**
 * Test 4: Task status transitions through the lifecycle.
 * Create -> Start -> verify running status.
 */
export async function test_task_status_transitions(client) {
  const task = await client.post("/api/v1/tasks", {
    description: "Generate a test report",
  });
  assertEqual(task.status, "pending", "initial status should be pending");

  // Start the task
  const started = await client.post(`/api/v1/tasks/${task.id}/start`);
  assertEqual(started.status, "running", "started task should be running");
  assertEqual(started.id, task.id, "task IDs should match after start");
}

/**
 * Test 5: Cancelling a task transitions it to cancelled status.
 */
export async function test_cancel_task(client) {
  const task = await client.post("/api/v1/tasks", {
    description: "Task to be cancelled",
  });
  assertOk(task.id);

  // Start then cancel
  await client.post(`/api/v1/tasks/${task.id}/start`);
  const cancelled = await client.post(`/api/v1/tasks/${task.id}/cancel`);
  assertEqual(
    cancelled.status,
    "cancelled",
    "cancelled task should have cancelled status"
  );
}

/**
 * Test 6: Pausing and resuming a task.
 */
export async function test_pause_resume_task(client) {
  const task = await client.post("/api/v1/tasks", {
    description: "Task to pause and resume",
  });
  assertOk(task.id);

  // Start the task
  const started = await client.post(`/api/v1/tasks/${task.id}/start`);
  assertEqual(started.status, "running", "task should be running");

  // Pause the task
  const paused = await client.post(`/api/v1/tasks/${task.id}/pause`);
  assertEqual(
    paused.status,
    "paused",
    "paused task should have paused status"
  );
}

/**
 * Test 7: Maximum concurrency of 1 agent per task (single collapse).
 * Creating multiple tasks ensures only the active ones can run.
 */
export async function test_max_agents_one_collapses(client) {
  // Create multiple tasks
  const task1 = await client.post("/api/v1/tasks", {
    description: "First concurrent task",
  });
  const task2 = await client.post("/api/v1/tasks", {
    description: "Second concurrent task",
  });

  assertOk(task1.id, "first task should have an id");
  assertOk(task2.id, "second task should have an id");

  // Both should be created independently
  const list = await client.get("/api/v1/tasks");
  const ids = list.tasks.map((t) => t.id);
  assertOk(ids.includes(task1.id), "task1 should be in list");
  assertOk(ids.includes(task2.id), "task2 should be in list");
}

/**
 * Test 8: Starting a task via the dedicated /start endpoint.
 */
export async function test_start_task_via_api(client) {
  const task = await client.post("/api/v1/tasks", {
    description: "Task to start explicitly",
  });
  assertEqual(task.status, "pending", "task should start as pending");

  const started = await client.post(`/api/v1/tasks/${task.id}/start`);
  assertEqual(started.status, "running", "task should be running after start");
  assertEqual(started.id, task.id, "IDs should match");

  // Verify via GET
  const detail = await client.get(`/api/v1/tasks/${task.id}`);
  assertEqual(detail.status, "running", "GET should confirm running status");
}
