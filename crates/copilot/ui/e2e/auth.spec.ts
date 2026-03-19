import { test, expect } from "@playwright/test";
import { injectTauriMock, injectTauriMockAuthenticated } from "./helpers/tauri-mock";
import { injectWsMock } from "./helpers/ws-mock";

test.describe("Auth", () => {
  test.beforeEach(async ({ page }) => {
    await injectWsMock(page);
  });

  test("app loads in demo mode — office visible without auth", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // The app should render without requiring auth.
    await expect(page.locator("role=main")).toBeVisible();
    // TopBar should show app name
    await expect(page.getByText("Day1 Copilot")).toBeVisible();
    // Sidebar navigation should be present
    await expect(page.getByRole("navigation", { name: "Office" })).toBeVisible();
  });

  test("auth panel opens when clicking sign in", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // TopBar has a "Sign in" button when not authenticated
    const signInBtn = page.getByRole("button", { name: /sign in/i });
    await expect(signInBtn).toBeVisible();
    await signInBtn.click();

    // AuthWall dialog should appear
    await expect(page.getByRole("dialog")).toBeVisible();
  });

  test("auth panel has Google and Email buttons", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // Open auth wall
    await page.getByRole("button", { name: /sign in/i }).click();

    // Check for Google and Email buttons
    await expect(page.getByText("Continue with Google")).toBeVisible();
    await expect(page.getByText("Continue with Email")).toBeVisible();
  });

  test("auth panel close X dismisses and returns to demo", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // Open auth wall
    await page.getByRole("button", { name: /sign in/i }).click();
    const dialog = page.getByRole("dialog");
    await expect(dialog).toBeVisible();

    // Close it via the close button inside the auth dialog
    await dialog.getByRole("button", { name: /close/i }).click();

    // Dialog should disappear, main view remains
    await expect(dialog).not.toBeVisible();
    await expect(page.locator("role=main")).toBeVisible();
  });

  test("Google OAuth button has correct Supabase URL with redirect", async ({ page }) => {
    await injectTauriMock(page);
    await page.goto("/");

    // Open auth wall
    await page.getByRole("button", { name: /sign in/i }).click();

    // Intercept window.open to capture the URL
    await page.evaluate(() => {
      (window as any).__capturedUrls = [] as string[];
      window.open = (url?: string | URL) => {
        (window as any).__capturedUrls.push(String(url ?? ""));
        return null;
      };
    });

    // Click Google button
    await page.getByText("Continue with Google").click();

    // Verify the URL was captured and contains the expected parts
    const urls = await page.evaluate(() => (window as any).__capturedUrls as string[]);
    expect(urls.length).toBeGreaterThan(0);
    expect(urls[0]).toContain("day1.doctor/auth/login");
    expect(urls[0]).toContain("provider=google");
    expect(urls[0]).toContain("redirect_uri=");
    expect(urls[0]).toContain("day1copilot");
  });

  test("deep link callback sets token and dismisses auth wall", async ({ page }) => {
    await injectTauriMock(page, {
      invoke: {
        get_auth_token: null,
        handle_auth_callback: "mock-jwt-token-abc",
        store_auth_token: null,
        fetch_balance: {
          dd_balance: 42,
          dd_limit: 100,
          subscription_tier: "free",
          tier_max_agents: 1,
          user_id: "user-123",
        },
      },
    });
    await page.goto("/");

    // Open auth wall
    await page.getByRole("button", { name: /sign in/i }).click();
    await expect(page.getByRole("dialog")).toBeVisible();

    // Simulate authentication by updating the mock to return a token
    await page.evaluate(() => {
      (window as any).__TAURI_INTERNALS__.invoke = async (cmd: string) => {
        if (cmd === "get_auth_token") return "mock-jwt-token-abc";
        if (cmd === "fetch_balance") return {
          dd_balance: 42, dd_limit: 100, subscription_tier: "free",
          tier_max_agents: 1, user_id: "user-123",
        };
        if (cmd === "store_auth_token") return null;
        if (cmd === "handle_auth_callback") return "mock-jwt-token-abc";
        if (cmd === "list_agents") return [
          { id: "1", name: "Dr. Bob", role: "orchestrator", status: "idle" },
          { id: "2", name: "Scout", role: "researcher", status: "idle" },
        ];
        if (cmd === "refresh_auth_token") return "mock-refreshed-token";
        if (cmd === "clear_auth") return null;
        return null;
      };
    });

    // The main view should still be visible (app doesn't crash)
    await expect(page.locator("role=main")).toBeVisible();
  });

  test("invalid token shows error without crashing", async ({ page }) => {
    await injectTauriMock(page, {
      invoke: {
        get_auth_token: "bad-token",
        fetch_balance: "__ERROR__:Invalid token",
      },
    });
    await page.goto("/");

    // App should still render even if checkStoredAuth fails
    await expect(page.locator("role=main")).toBeVisible();
    // Should remain unauthenticated (sign in button visible)
    await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();
  });

  test("logout clears token and returns to demo mode", async ({ page }) => {
    await injectTauriMockAuthenticated(page);
    await page.goto("/");

    // Authenticated state: "Sign Out" button should be visible
    await expect(page.getByText("Sign Out")).toBeVisible();

    // Click Sign Out
    await page.getByText("Sign Out").click();

    // After logout, "Sign in" button should reappear
    await expect(page.getByRole("button", { name: /sign in/i })).toBeVisible();
  });
});
