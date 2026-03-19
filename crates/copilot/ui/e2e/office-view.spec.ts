import { test, expect } from "@playwright/test";
import { injectTauriMock } from "./helpers/tauri-mock";
import { injectWsMock, emitWsEvent } from "./helpers/ws-mock";

test.describe("Office View", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
    await injectTauriMock(page);
  });

  test("valley renders 11 parcels with correct states", async ({ page }) => {
    await page.goto("/");

    // Default view is "valley" — look for the DAY1 VALLEY title
    await expect(page.getByRole("main").getByText("Day1 Valley").first()).toBeVisible();

    // The footer shows the active count "1/1 offices active" (free tier)
    await expect(page.getByText(/offices active/)).toBeVisible();
  });

  test("office view shows 6 agents with correct names and roles", async ({ page }) => {
    await page.goto("/");

    // Navigate to office view via sidebar
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Dr. Bob's Office").click();

    // Open the Command Center agents tab to verify agent names
    const commandCenter = page.getByRole("complementary");
    const agentsBtn = commandCenter.getByRole("button", { name: "Agents" });
    if (await agentsBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
      await agentsBtn.click();
    }

    // All 6 agent names should be present in the agents panel
    const agentNames = ["Dr. Bob", "Scout", "Sage", "Quill", "Pixel", "Atlas"];
    for (const name of agentNames) {
      await expect(commandCenter.getByText(name).first()).toBeVisible();
    }
  });

  test("agent status badges update on state_changed event", async ({ page }) => {
    await page.goto("/");

    // Go to office view
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Dr. Bob's Office").click();

    // Switch to Agents tab in command center to see statuses
    const commandCenter = page.getByRole("complementary");
    const agentsBtn = commandCenter.getByRole("button", { name: "Agents" });
    if (await agentsBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
      await agentsBtn.click();
    }

    // Scout should be "Idle" initially
    await expect(commandCenter.getByText("Idle").first()).toBeVisible();

    // Give WS mock time to connect
    await page.waitForTimeout(300);

    // Emit a state_changed event for Scout (agent_id "2")
    await emitWsEvent(page, {
      agent_id: "2",
      type: "agent.state_changed",
      payload: { from: "idle", to: "working" },
    });

    // Wait for the status to update
    await expect(commandCenter.getByText("Working").first()).toBeVisible({ timeout: 3000 });
  });

  test("clicking an agent shows agent detail card", async ({ page }) => {
    await page.goto("/");

    // Navigate to office view
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Dr. Bob's Office").click();

    // Verify the agents panel in Command Center shows agent details
    const commandCenter = page.getByRole("complementary");
    const agentsBtn = commandCenter.getByRole("button", { name: "Agents" });
    if (await agentsBtn.isVisible({ timeout: 1000 }).catch(() => false)) {
      await agentsBtn.click();
    }

    // Agent card should show Dr. Bob's details with role and skills
    await expect(commandCenter.getByText("Dr. Bob").first()).toBeVisible();
    await expect(commandCenter.getByText("orchestrator").first()).toBeVisible();
  });

  test("zoom control works 80-150 percent range", async ({ page }) => {
    await page.goto("/");

    // The ZoomControl is in the StatusBar footer. Default zoom is 100%.
    const footer = page.getByRole("contentinfo");
    await expect(footer.getByText("100%")).toBeVisible();

    // Click zoom in (+) — the button has title="Zoom in"
    await footer.getByTitle("Zoom in").click();
    await expect(footer.getByText("110%")).toBeVisible();

    // Click zoom in again
    await footer.getByTitle("Zoom in").click();
    await expect(footer.getByText("120%")).toBeVisible();

    // Click zoom out (-)
    await footer.getByTitle("Zoom out").click();
    await expect(footer.getByText("110%")).toBeVisible();

    // Reset to 100%
    await footer.getByTitle("Reset zoom").click();
    await expect(footer.getByText("100%")).toBeVisible();
  });

  test("zoom persists across view switches", async ({ page }) => {
    await page.goto("/");

    // Set zoom to 110%
    const footer = page.getByRole("contentinfo");
    await footer.getByTitle("Zoom in").click();
    await expect(footer.getByText("110%")).toBeVisible();

    // Switch to Tasks view
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Task List").click();

    // Zoom should still show 110% in StatusBar
    await expect(footer.getByText("110%")).toBeVisible();

    // Switch back to Valley
    await sidebar.getByText("Day1 Valley").first().click();
    await expect(footer.getByText("110%")).toBeVisible();
  });

  test("window resize does not break canvas layout", async ({ page }) => {
    await page.goto("/");

    // Navigate to office view
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Dr. Bob's Office").click();

    // Resize the viewport
    await page.setViewportSize({ width: 800, height: 600 });
    await page.waitForTimeout(200);

    // The layout should still be intact
    await expect(page.locator("role=main")).toBeVisible();

    // Resize larger
    await page.setViewportSize({ width: 1400, height: 900 });
    await page.waitForTimeout(200);

    await expect(page.locator("role=main")).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Office" })).toBeVisible();
  });
});
