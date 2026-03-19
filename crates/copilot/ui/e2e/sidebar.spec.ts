import { test, expect } from "@playwright/test";
import { injectTauriMock } from "./helpers/tauri-mock";
import { injectWsMock } from "./helpers/ws-mock";

test.describe("Sidebar", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
    await injectTauriMock(page);
    await page.goto("/");
  });

  test("all 4 accordion sections render", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // 4 main sections: Valley, Tasks, Workspace, Metrics
    await expect(sidebar.getByText("Day1 Valley")).toBeVisible();
    await expect(sidebar.getByText("Tasks")).toBeVisible();
    await expect(sidebar.getByText("Workspace")).toBeVisible();
    await expect(sidebar.getByText("Metrics")).toBeVisible();
  });

  test("accordion expand collapse works", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // The Tasks section should be expanded by default (showing "Task List")
    await expect(sidebar.getByText("Task List")).toBeVisible();

    // Workspace section sub-items should be visible
    await expect(sidebar.getByText("Files & Artifacts")).toBeVisible();

    // Click Workspace header to collapse
    await sidebar.getByText("Workspace").click();
    // Clicking again should toggle the expanded state
    await sidebar.getByText("Workspace").click();

    // The Workspace header should still be visible
    await expect(sidebar.getByText("Workspace")).toBeVisible();
  });

  test("view switches correctly", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // Click Tasks -> Task List to switch view
    await sidebar.getByText("Task List").click();

    // TaskView renders a TaskPanel with "New Task" button
    await expect(page.getByRole("main").getByText("New Task").first()).toBeVisible();

    // Click Metrics -> Usage & Costs
    await sidebar.getByText("Usage & Costs").click();

    // MetricsView renders title
    await expect(page.getByRole("main").getByText("Total Cost").first()).toBeVisible();

    // Click Workspace -> Files & Artifacts
    await sidebar.getByText("Files & Artifacts").click();

    // WorkspaceView renders title
    await expect(page.getByRole("main").getByText("Workspace")).toBeVisible();
  });

  test("active view highlights in sidebar", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // Navigate to Tasks
    await sidebar.getByText("Task List").click();

    // The Tasks section should be visible
    const taskListItem = sidebar.getByText("Task List");
    await expect(taskListItem).toBeVisible();

    // Navigate to Metrics
    await sidebar.getByText("Usage & Costs").click();

    // Metrics > Usage & Costs should be visible
    const metricsItem = sidebar.getByText("Usage & Costs");
    await expect(metricsItem).toBeVisible();
  });

  test("developer section visible in debug mode", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // Developer section is always visible in the sidebar (collapsed by default)
    await expect(sidebar.getByText("Developer")).toBeVisible();

    // Click Developer to expand it
    await sidebar.getByText("Developer").click();

    // Debug Console sub-item should appear
    await expect(sidebar.getByText("Debug Console")).toBeVisible();

    // Click Debug Console to navigate
    await sidebar.getByText("Debug Console").click();

    // The DebugView should load (it's lazy loaded)
    await expect(page.getByRole("main").getByText("Event Log").first()).toBeVisible({ timeout: 3000 });
  });

  test("sidebar collapse expand works", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });

    // Sidebar should be expanded (w-[220px]) — text labels visible
    await expect(sidebar.getByText("Day1 Valley")).toBeVisible();

    // Click the collapse toggle button
    const collapseBtn = sidebar.getByRole("button", { name: /collapse sidebar/i });
    await collapseBtn.click();

    // After collapse, text labels should not be visible
    await expect(sidebar.getByText("Day1 Valley")).not.toBeVisible();

    // Expand it back
    const expandBtn = sidebar.getByRole("button", { name: /expand sidebar/i });
    await expandBtn.click();

    // Text should be visible again
    await expect(sidebar.getByText("Day1 Valley")).toBeVisible();
  });
});
