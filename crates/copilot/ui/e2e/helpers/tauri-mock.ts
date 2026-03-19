/**
 * Tauri invoke mock for Playwright E2E tests.
 *
 * Intercepts `window.__TAURI_INTERNALS__.invoke` so the app can run
 * without a real Tauri binary. Call `injectTauriMock(page, overrides)`
 * before navigating.
 */
import type { Page } from "@playwright/test";

export interface TauriMockOverrides {
  /** Override specific invoke commands. Key = command name, value = return value or function. */
  invoke?: Record<string, unknown>;
}

const DEFAULT_AGENTS = [
  { id: "1", name: "Dr. Bob", role: "orchestrator", status: "idle" },
  { id: "2", name: "Scout", role: "researcher", status: "idle" },
  { id: "3", name: "Sage", role: "analyst", status: "idle" },
  { id: "4", name: "Quill", role: "writer", status: "idle" },
  { id: "5", name: "Pixel", role: "coder", status: "idle" },
  { id: "6", name: "Atlas", role: "operator", status: "idle" },
];

const DEFAULT_BALANCE = {
  dd_balance: 42,
  dd_limit: 100,
  subscription_tier: "free",
  tier_max_agents: 1,
  user_id: "user-test-123",
};

/**
 * Inject the Tauri mock into the page before app code runs.
 *
 * Must be called BEFORE page.goto().
 */
export async function injectTauriMock(
  page: Page,
  overrides: TauriMockOverrides = {},
) {
  const invokeOverrides = overrides.invoke ?? {};

  await page.addInitScript(
    ({ agents, balance, overrides: ovr }) => {
      // Build the default responses map
      const responses: Record<string, unknown> = {
        list_agents: agents,
        get_auth_token: null,
        store_auth_token: null,
        clear_auth: null,
        fetch_balance: balance,
        create_task: {
          id: "task-new-1",
          title: "Test task",
          status: "pending",
        },
        respond_approval: null,
        list_providers: [
          { id: "anthropic", name: "Anthropic", endpoint: "https://api.anthropic.com", enabled: true },
        ],
        get_runtime_status: { version: "3.0.0-alpha", uptime_s: 120 },
        handle_auth_callback: "mock-jwt-token-12345",
        refresh_auth_token: "mock-refreshed-token",
        ...ovr,
      };

      // Mock the Tauri internals that @tauri-apps/api/core reads
      (window as any).__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args?: Record<string, unknown>) => {
          // Check if the command has a custom response
          if (cmd in responses) {
            const val = responses[cmd];
            // If the override is "__ERROR__:message", throw instead
            if (typeof val === "string" && val.startsWith("__ERROR__:")) {
              throw val.slice("__ERROR__:".length);
            }
            return val;
          }
          // Default: resolve with null
          return null;
        },
        metadata: {
          currentWindow: { label: "main" },
          currentWebview: { label: "main" },
        },
        convertFileSrc: (path: string) => path,
      };

      // Also mock the Tauri event system (for deep link plugin)
      (window as any).__TAURI_INTERNALS__.invoke.__isTauriMock = true;

      // Provide a stub for @tauri-apps/api/event listen/emit
      (window as any).__TAURI_EVENT_LISTENERS__ = new Map();

      // Skip onboarding wizard by setting the localStorage flag
      localStorage.setItem("d1-copilot-has-onboarded", "true");
    },
    {
      agents: DEFAULT_AGENTS,
      balance: DEFAULT_BALANCE,
      overrides: invokeOverrides,
    },
  );
}

/**
 * Inject the Tauri mock configured for an "authenticated" session.
 */
export async function injectTauriMockAuthenticated(
  page: Page,
  overrides: TauriMockOverrides = {},
) {
  await injectTauriMock(page, {
    invoke: {
      get_auth_token: "mock-jwt-token-12345",
      fetch_balance: {
        dd_balance: 42,
        dd_limit: 100,
        subscription_tier: "free",
        tier_max_agents: 1,
        user_id: "user-test-123",
      },
      ...overrides.invoke,
    },
  });
}
