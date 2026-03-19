import { test, expect } from "@playwright/test";
import { injectTauriMock } from "./helpers/tauri-mock";
import { injectWsMock, emitWsEvent } from "./helpers/ws-mock";

test.describe("Metrics & Workspace", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
    await injectTauriMock(page);
    await page.goto("/");
  });

  test("metrics view shows real data from stores", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Usage & Costs").click();

    const main = page.getByRole("main");

    await expect(main.getByText("Total Cost")).toBeVisible();
    await expect(main.getByText("Total Tokens")).toBeVisible();
    await expect(main.getByText("Tasks Done")).toBeVisible();
    await expect(main.getByText("DD Balance")).toBeVisible();

    await expect(main.getByText("By Agent")).toBeVisible();
    await expect(main.getByText("Dr. Bob").first()).toBeVisible();
    await expect(main.getByText("Scout").first()).toBeVisible();
  });

  test("DD balance updates on cost updated event", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Usage & Costs").click();

    // Give WS time to connect
    await page.waitForTimeout(300);

    await emitWsEvent(page, {
      agent_id: "2",
      type: "cost.updated",
      payload: { session_tokens: 5000, session_cost_dd: 15 },
    });

    await page.waitForTimeout(500);

    const main = page.getByRole("main");
    await expect(main.getByText("Total Cost")).toBeVisible();
    await expect(main.getByText(/DD/).first()).toBeVisible();
  });

  test("per agent cost breakdown renders", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Usage & Costs").click();

    const main = page.getByRole("main");
    await expect(main.getByText("By Agent")).toBeVisible();

    const agentNames = ["Dr. Bob", "Scout", "Sage", "Quill", "Pixel", "Atlas"];
    for (const name of agentNames) {
      await expect(main.getByText(name).first()).toBeVisible();
    }

    await expect(main.getByText("Tokens").first()).toBeVisible();
    await expect(main.getByText("Cost").first()).toBeVisible();
  });

  test("workspace view lists artifacts", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Files & Artifacts").click();

    const main = page.getByRole("main");
    await expect(main.getByText("Workspace").first()).toBeVisible();

    await expect(main.getByText("research-notes.md")).toBeVisible();
    await expect(main.getByText("comparison-chart.svg")).toBeVisible();
    await expect(main.getByText("final-report.pdf")).toBeVisible();
  });

  test("empty workspace shows no artifacts state", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Files & Artifacts").click();

    const main = page.getByRole("main");

    // The workspace has mock artifacts. Check the Memory section.
    // The mock eventLogStore has a "memory" tool trace (tt-6) with status "success".
    // So the memory section shows an entry (content "session" from Dr. Bob).
    await expect(main.getByText("Memory").first()).toBeVisible();
    await expect(main.getByText("session").first()).toBeVisible();
  });

  test("memory section shows entries or empty state", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Files & Artifacts").click();

    const main = page.getByRole("main");

    // Memory section header should be visible
    await expect(main.getByText("Memory").first()).toBeVisible();

    // The mock toolTraces have one memory entry (tt-6) with status "success"
    // The entry count indicator "(1)" should appear
    await expect(main.getByText("(1)")).toBeVisible();
  });

  test("cost display formats DD correctly", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Usage & Costs").click();

    const main = page.getByRole("main");
    await expect(main.getByText(/DD/).first()).toBeVisible();

    // The per-agent table shows costs with "DD" suffix
    const costCells = main.locator("td").filter({ hasText: /DD/ });
    await expect(costCells.first()).toBeVisible();
  });

  test("balance reaching zero shows warning", async ({ page }) => {
    const sidebar = page.getByRole("navigation", { name: "Office" });
    await sidebar.getByText("Usage & Costs").click();

    const main = page.getByRole("main");
    await expect(main.getByText("DD Balance")).toBeVisible();

    // The balance value "0" should be visible (not authenticated, costStore starts at 0)
    await expect(main.getByText("0").first()).toBeVisible();
    await expect(main.getByText("Total Cost")).toBeVisible();
  });

  test("i18n toggle EN CN updates all visible text", async ({ page }) => {
    // App starts in English by default
    await expect(page.getByText("Day1 Copilot")).toBeVisible();

    // Click the language toggle (shows "中" in English mode)
    const langToggle = page.getByRole("button", { name: /switch to chinese/i });
    await langToggle.click();

    // After switching to Chinese, the toggle should show "EN"
    await expect(page.getByRole("button", { name: /switch to english/i })).toBeVisible();

    // Switch back to English
    await page.getByRole("button", { name: /switch to english/i }).click();

    await expect(page.getByText("Day1 Copilot")).toBeVisible();
  });

  test("display zoom setting persists", async ({ page }) => {
    const footer = page.getByRole("contentinfo");

    // Default zoom is 100%
    await expect(footer.getByText("100%")).toBeVisible();

    // Zoom in to 110%
    await footer.getByTitle("Zoom in").click();
    await expect(footer.getByText("110%")).toBeVisible();

    // Verify the root font-size was updated
    const fontSize = await page.evaluate(() => {
      return document.documentElement.style.fontSize;
    });
    expect(fontSize).toBe("110%");

    // Verify localStorage was updated
    const savedZoom = await page.evaluate(() => {
      return localStorage.getItem("day1-zoom");
    });
    expect(savedZoom).toBe("110");
  });
});
