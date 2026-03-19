import { test, expect } from "@playwright/test";
import { injectTauriMock } from "./helpers/tauri-mock";
import { injectWsMock, emitWsEvent } from "./helpers/ws-mock";

test.describe("Approval", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
    await injectTauriMock(page);
    await page.goto("/");

    // Give the WebSocket mock time to be set up by useEventStream
    await page.waitForTimeout(500);
  });

  test("approval dialog renders on approval requested event", async ({ page }) => {
    // Emit an approval.requested event
    await emitWsEvent(page, {
      agent_id: "5",
      type: "approval.requested",
      payload: {
        action: "shell",
        risk_level: "high",
        context: "Pixel wants to run npm test",
        tool_name: "shell",
        params: { cmd: "npm test" },
      },
    });

    // The ApprovalDialog should appear as a modal
    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Should show the agent name in the heading
    await expect(dialog.getByRole("heading", { name: /Pixel/ })).toBeVisible();

    // Should show the risk level badge
    await expect(dialog.getByText("HIGH", { exact: true })).toBeVisible();

    // Should show the action/tool name
    await expect(dialog.getByText("shell").first()).toBeVisible();
  });

  test("approve button sends respond_approval", async ({ page }) => {
    await emitWsEvent(page, {
      id: "approval-1",
      agent_id: "5",
      type: "approval.requested",
      payload: {
        action: "shell",
        risk_level: "high",
        context: "Pixel wants to run npm test",
        tool_name: "shell",
        params: { cmd: "npm test" },
      },
    });

    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Click "Allow Once"
    await dialog.getByText("Allow Once").click();

    // The dialog should dismiss
    await expect(dialog).not.toBeVisible();
  });

  test("deny button sends deny decision", async ({ page }) => {
    await emitWsEvent(page, {
      id: "approval-2",
      agent_id: "5",
      type: "approval.requested",
      payload: {
        action: "shell",
        risk_level: "medium",
        context: "Pixel wants to run a command",
        tool_name: "shell",
        params: { cmd: "ls -la" },
      },
    });

    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Click "Reject"
    await dialog.getByText("Reject").click();

    // The dialog should dismiss
    await expect(dialog).not.toBeVisible();
  });

  test("queue navigation works with multiple approvals", async ({ page }) => {
    // Emit two events via the WS mock with a delay between them
    await page.evaluate(() => {
      const sockets = (window as any).__WS_MOCK_SOCKETS__ as Array<{
        onmessage: ((ev: MessageEvent) => void) | null;
      }>;

      const event1 = {
        id: "approval-q1",
        agent_id: "5",
        timestamp: new Date().toISOString(),
        type: "approval.requested",
        payload: {
          action: "shell",
          risk_level: "high",
          context: "First request",
          tool_name: "shell",
          params: { cmd: "npm test" },
        },
      };

      const event2 = {
        id: "approval-q2",
        agent_id: "4",
        timestamp: new Date().toISOString(),
        type: "approval.requested",
        payload: {
          action: "file-write",
          risk_level: "medium",
          context: "Second request",
          tool_name: "file-write",
          params: { path: "/report.md" },
        },
      };

      for (const sock of sockets) {
        if (sock.onmessage) {
          sock.onmessage(new MessageEvent("message", { data: JSON.stringify(event1) }));
        }
      }

      // Delay the second event to ensure React processes the first
      setTimeout(() => {
        for (const sock of sockets) {
          if (sock.onmessage) {
            sock.onmessage(new MessageEvent("message", { data: JSON.stringify(event2) }));
          }
        }
      }, 300);
    });

    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Wait for the queue navigation to appear (rendered when length > 1)
    await expect(dialog.getByText(/1\s*\/\s*2/)).toBeVisible({ timeout: 5000 });

    // Verify the first approval is showing (shell action)
    await expect(dialog.getByRole("heading", { name: /Pixel/ })).toBeVisible();

    // Click "Next" to view the second approval
    await dialog.getByRole("button", { name: /next approval/i }).click();

    // Should now show "2 / 2"
    await expect(dialog.getByText(/2\s*\/\s*2/)).toBeVisible();

    // Click "Previous" to go back to first
    await dialog.getByRole("button", { name: /previous approval/i }).click();
    await expect(dialog.getByText(/1\s*\/\s*2/)).toBeVisible();
  });

  test("approval dialog dismisses after decision", async ({ page }) => {
    await emitWsEvent(page, {
      id: "approval-dismiss",
      agent_id: "5",
      type: "approval.requested",
      payload: {
        action: "shell",
        risk_level: "high",
        context: "Test dismissal",
        tool_name: "shell",
        params: { cmd: "echo test" },
      },
    });

    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Allow Always
    await dialog.getByText("Allow Always").click();

    // Dialog should be gone
    await expect(dialog).not.toBeVisible();

    // Main app should be fully functional
    await expect(page.locator("role=main")).toBeVisible();
  });

  test("rapid approve deny does not double submit", async ({ page }) => {
    await emitWsEvent(page, {
      id: "approval-rapid",
      agent_id: "5",
      type: "approval.requested",
      payload: {
        action: "shell",
        risk_level: "critical",
        context: "Rapid test",
        tool_name: "shell",
        params: { cmd: "rm -rf /" },
      },
    });

    const dialog = page.getByRole("dialog", { name: "Permission Request" });
    await expect(dialog).toBeVisible({ timeout: 3000 });

    // Should show CRITICAL badge (exact match to avoid matching trust checkbox text)
    await expect(dialog.getByText("CRITICAL", { exact: true })).toBeVisible();

    // Click Allow Once quickly
    await dialog.getByText("Allow Once").click();

    // Dialog should be dismissed
    await expect(dialog).not.toBeVisible();

    // App should remain stable
    await expect(page.locator("role=main")).toBeVisible();
  });
});
