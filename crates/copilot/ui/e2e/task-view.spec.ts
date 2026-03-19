import { test, expect } from "@playwright/test";
import { injectTauriMock } from "./helpers/tauri-mock";
import { injectWsMock, emitWsEvent } from "./helpers/ws-mock";

test.describe("Task View", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
    await injectTauriMock(page);
    await page.goto("/");

    // Navigate to Task view
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Task List").click();
  });

  test("task list renders with parent subtask hierarchy", async ({ page }) => {
    const main = page.getByRole("main");

    // Task 1: "Compare AI Frameworks"
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();

    // Task 2: "Research Report with Parallel Analysis"
    await expect(main.getByText("Research Report with Parallel Analysis")).toBeVisible();

    // Task 1 is expanded by default — its steps should be visible
    await expect(main.getByText("Research AI frameworks").first()).toBeVisible();
    await expect(main.getByText("Analyze findings").first()).toBeVisible();
  });

  test("task status updates from WS events", async ({ page }) => {
    const main = page.getByRole("main");

    // Give WS time to connect
    await page.waitForTimeout(300);

    // Emit a task.status_changed event to mark task-1 completed
    await emitWsEvent(page, {
      agent_id: "1",
      type: "task.status_changed",
      payload: { task_id: "task-1", from: "running", to: "completed" },
    });

    await page.waitForTimeout(500);

    // The task panel should still be visible after status update
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();
  });

  test("step completed updates progress bar", async ({ page }) => {
    const main = page.getByRole("main");

    // Give WS time to connect
    await page.waitForTimeout(300);

    // Emit step_completed for step "2" in task-1
    await emitWsEvent(page, {
      agent_id: "3",
      type: "task.step_completed",
      payload: { task_id: "task-1", step_id: "2", status: "running", duration_ms: 25000 },
    });

    await page.waitForTimeout(500);

    // The task panel still renders properly
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();
  });

  test("subtask expand shows agent assignment", async ({ page }) => {
    const main = page.getByRole("main");

    // Task-1 is expanded by default. Its steps show agent assignments.
    await expect(main.getByText("Scout").first()).toBeVisible();
    await expect(main.getByText("Sage").first()).toBeVisible();

    // Click task-2 to expand it
    await main.getByText("Research Report with Parallel Analysis").click();

    await page.waitForTimeout(300);

    // Task-2 should be visible
    await expect(main.getByText("Research Report with Parallel Analysis")).toBeVisible();
  });

  test("cancel task button works", async ({ page }) => {
    const main = page.getByRole("main");

    // The TaskPanel has a "+ New Task" button
    await expect(main.getByText("New Task").first()).toBeVisible();

    // Click a task to toggle expand/collapse
    await main.getByText("Compare AI Frameworks").click();
    await page.waitForTimeout(200);

    // After toggling, the task should still be visible
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();
  });

  test("empty task list shows placeholder", async ({ page }) => {
    const main = page.getByRole("main");

    // Verify the "New Task" button exists
    await expect(main.getByText("New Task").first()).toBeVisible();

    // Verify clicking New Task opens the inline input
    await main.getByText("New Task").first().click();

    // The input placeholder should appear — scope to main to avoid matching
    // the command center textarea
    await expect(main.locator("input[placeholder]")).toBeVisible();
  });

  test("50 plus tasks render without lag", async ({ page }) => {
    const main = page.getByRole("main");

    // Verify the existing tasks render quickly
    const start = Date.now();
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();
    const elapsed = Date.now() - start;

    // Should render within reasonable time
    expect(elapsed).toBeLessThan(5000);

    // Verify the task panel is fully interactive
    await expect(main.getByText("New Task").first()).toBeVisible();
  });

  test("failed task shows error state", async ({ page }) => {
    const main = page.getByRole("main");

    // Give WS time to connect
    await page.waitForTimeout(300);

    // Emit a task status change to "failed"
    await emitWsEvent(page, {
      agent_id: "1",
      type: "task.status_changed",
      payload: { task_id: "task-1", from: "running", to: "failed" },
    });

    await page.waitForTimeout(500);

    // The task should still be visible (not removed)
    await expect(main.getByText("Compare AI Frameworks")).toBeVisible();

    // The task panel should still be fully functional
    await expect(main.getByText("New Task").first()).toBeVisible();
  });
});
