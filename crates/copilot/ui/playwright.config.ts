import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 15_000,
  retries: 0,
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 15_000,
  },
});
