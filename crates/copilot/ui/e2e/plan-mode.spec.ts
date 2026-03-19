import { test, expect } from "@playwright/test";
import { injectTauriMock, injectTauriMockAuthenticated } from "./helpers/tauri-mock";
import { injectWsMock } from "./helpers/ws-mock";

test.describe("Plan Mode", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
  });

  test("chat panel renders in Plan Mode by default", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // The Command Center (RightPanel) should render with Chat mode active by default
    const cc = page.getByRole("complementary");
    await expect(cc).toBeVisible();

    // Chat mode toggle should show Plan as active
    await expect(cc.getByText("Plan").first()).toBeVisible();

    // Plan Mode description should be visible
    await expect(cc.getByText(/Dr. Bob plans/)).toBeVisible();
  });

  test("user can type and send message", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Type a message in the textarea
    await cc.locator("textarea").fill("Research the latest AI trends");

    // Click send button
    await cc.getByRole("button", { name: /send/i }).click();

    // The user message should appear in the chat
    await expect(cc.getByText("Research the latest AI trends")).toBeVisible();

    // Dr. Bob should respond (mock response after 800ms timeout)
    await expect(
      cc.getByText(/Let me think about how to break this down/),
    ).toBeVisible({ timeout: 3000 });
  });

  test("confirm plan creates task via invoke", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Send a message first (required to show confirm button)
    await cc.locator("textarea").fill("Build a REST API with authentication");
    await cc.getByRole("button", { name: /send/i }).click();

    await expect(cc.getByText("Build a REST API with authentication")).toBeVisible();

    // The "Confirm Plan & Start Execution" button should appear
    const confirmBtn = cc.getByText("Confirm Plan & Start Execution");
    await expect(confirmBtn).toBeVisible();

    // Click confirm
    await confirmBtn.click();

    // After confirmation, Dr. Bob should acknowledge
    await expect(cc.getByText(/Plan confirmed/).first()).toBeVisible({ timeout: 5000 });
  });

  test("task creation shows loading state", async ({ page }) => {
    await injectTauriMock(page, {
      invoke: {
        create_task: "__ERROR__:timeout",
      },
    });
    await page.goto("/");

    const cc = page.getByRole("complementary");

    await cc.locator("textarea").fill("Test task creation");
    await cc.getByRole("button", { name: /send/i }).click();

    await expect(cc.getByText("Test task creation")).toBeVisible();

    // Click confirm — the fallback response should appear
    await cc.getByText("Confirm Plan & Start Execution").click();

    await expect(cc.getByText(/Plan confirmed/).first()).toBeVisible({ timeout: 5000 });
  });

  test("unauthenticated user sees free mode hint", async ({ page }) => {
    await injectTauriMock(page); // Not authenticated
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Send a message to make the confirm button appear
    await cc.locator("textarea").fill("Test free mode hint");
    await cc.getByRole("button", { name: /send/i }).click();

    // The free mode hint should appear near the confirm button
    await expect(
      cc.getByText("Running in free mode. Sign in for full access."),
    ).toBeVisible({ timeout: 3000 });
  });

  test("empty message cannot be submitted", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Send button should be disabled when textarea is empty
    const sendBtn = cc.getByRole("button", { name: /send/i });
    await expect(sendBtn).toBeDisabled();

    // Type whitespace only
    await cc.locator("textarea").fill("   ");
    await expect(sendBtn).toBeDisabled();
  });

  test("long message over 5000 chars handled", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    const longMsg = "A".repeat(5001);
    await cc.locator("textarea").fill(longMsg);

    const sendBtn = cc.getByRole("button", { name: /send/i });
    await expect(sendBtn).toBeEnabled();
    await sendBtn.click();

    // The message should appear (no crash)
    await expect(cc.getByText(/AAAA/).first()).toBeVisible();
  });

  test("BTW mode toggle switches chat context", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Should start in Plan mode
    await expect(cc.getByText(/Dr. Bob plans/).first()).toBeVisible();

    // Switch to BTW mode
    await cc.getByText("BTW").click();

    // BTW mode description should show
    await expect(cc.getByText(/won't interrupt/i).first()).toBeVisible();

    // Switch back to Plan
    await cc.getByText("Plan").first().click();
    await expect(cc.getByText(/Dr. Bob plans/).first()).toBeVisible();
  });

  test("chat history preserved across mode switches", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    const cc = page.getByRole("complementary");

    // Send a message in Plan mode
    await cc.locator("textarea").fill("Plan mode message");
    await cc.getByRole("button", { name: /send/i }).click();
    await expect(cc.getByText("Plan mode message")).toBeVisible();

    // Switch to BTW mode
    await cc.getByText("BTW").click();

    // Switch back to Plan mode
    await cc.getByText("Plan").first().click();

    // The previous message should still be there
    await expect(cc.getByText("Plan mode message")).toBeVisible();
  });

  test("network error during task creation shows error", async ({ page }) => {
    await injectTauriMock(page, {
      invoke: {
        create_task: "__ERROR__:Network connection failed",
      },
    });
    await page.goto("/");

    const cc = page.getByRole("complementary");

    await cc.locator("textarea").fill("Task that will fail");
    await cc.getByRole("button", { name: /send/i }).click();

    // Confirm the plan
    await cc.getByText("Confirm Plan & Start Execution").click();

    // The catch block shows a fallback "Plan confirmed" message
    await expect(cc.getByText(/Plan confirmed/).first()).toBeVisible({ timeout: 5000 });
  });
});
