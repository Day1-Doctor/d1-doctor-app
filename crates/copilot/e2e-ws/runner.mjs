#!/usr/bin/env node
// E2E WebSocket Test Runner for Station Runtime IPC server.
//
// Discovers *.test.mjs files, optionally launches the Tauri binary,
// waits for the IPC server to respond, runs each test, and reports results.
//
// Usage:
//   node runner.mjs              # Auto-detect binary or skip
//   node runner.mjs --port 14200 # Override port
//   node runner.mjs --binary /path/to/binary

import { readdir } from "fs/promises";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { spawn } from "child_process";
import { isPortListening, createClient } from "./client.mjs";

const __dirname = dirname(fileURLToPath(import.meta.url));

// --- Configuration ---
const DEFAULT_PORT = 14200;
const STARTUP_TIMEOUT_MS = 10000;
const STARTUP_POLL_MS = 500;

function parseArgs() {
  const args = process.argv.slice(2);
  const config = { port: DEFAULT_PORT, binary: null };
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--port" && args[i + 1]) {
      config.port = parseInt(args[i + 1], 10);
      i++;
    } else if (args[i] === "--binary" && args[i + 1]) {
      config.binary = args[i + 1];
      i++;
    }
  }
  return config;
}

/**
 * Try to find the copilot binary in typical build output locations.
 */
async function findBinary() {
  const { existsSync } = await import("fs");
  const candidates = [
    resolve(__dirname, "../../target/debug/d1-copilot"),
    resolve(__dirname, "../../target/release/d1-copilot"),
    resolve(__dirname, "../../target/debug/d1_copilot"),
    resolve(__dirname, "../../target/release/d1_copilot"),
  ];
  for (const candidate of candidates) {
    try {
      if (existsSync(candidate)) return candidate;
    } catch {
      // ignore
    }
  }
  return null;
}

/**
 * Wait for the server to become responsive.
 */
async function waitForServer(port, timeoutMs) {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await isPortListening(port, 1000)) return true;
    await new Promise((r) => setTimeout(r, STARTUP_POLL_MS));
  }
  return false;
}

/**
 * Discover all test files in the current directory.
 */
async function discoverTests() {
  const files = await readdir(__dirname);
  return files.filter((f) => f.endsWith(".test.mjs")).sort();
}

/**
 * Load and run all exported test functions from a test file.
 * Returns an array of { name, status, error? }.
 */
async function runTestFile(filePath, port, serverAvailable) {
  const results = [];
  const mod = await import(filePath);

  // Each export named test_* is a test function
  const testFns = Object.entries(mod).filter(([name]) =>
    name.startsWith("test_")
  );

  for (const [name, fn] of testFns) {
    if (!serverAvailable) {
      results.push({ name, status: "SKIP", error: "binary not running" });
      continue;
    }

    let client;
    try {
      client = await createClient(port);
      // Wait briefly for the "connected" welcome event
      await new Promise((r) => setTimeout(r, 200));
      await fn(client);
      results.push({ name, status: "PASS" });
    } catch (err) {
      results.push({ name, status: "FAIL", error: err.message });
    } finally {
      if (client) {
        try {
          client.close();
        } catch {
          // ignore close errors
        }
      }
    }
  }

  return results;
}

// --- Main ---

async function main() {
  const config = parseArgs();
  let childProcess = null;

  console.log("========================================");
  console.log("  Station Runtime E2E WebSocket Tests   ");
  console.log("========================================\n");

  // Check if server is already running
  let serverAvailable = await isPortListening(config.port, 2000);

  if (serverAvailable) {
    console.log(`Server already running on port ${config.port}\n`);
  } else {
    // Try to launch binary
    const binary = config.binary || (await findBinary());
    if (binary) {
      console.log(`Launching binary: ${binary}`);
      childProcess = spawn(binary, [], {
        stdio: "pipe",
        env: { ...process.env, STATION_PORT: String(config.port) },
      });
      childProcess.on("error", (err) => {
        console.error(`Binary launch error: ${err.message}`);
      });

      console.log(`Waiting for server on port ${config.port}...`);
      serverAvailable = await waitForServer(config.port, STARTUP_TIMEOUT_MS);
      if (serverAvailable) {
        console.log("Server is ready.\n");
      } else {
        console.log("Server did not start within timeout.\n");
      }
    } else {
      console.log(
        `No server on port ${config.port} and no binary found.`
      );
      console.log("Tests will be SKIPPED (binary not running).\n");
    }
  }

  // Discover and run tests
  const testFiles = await discoverTests();
  if (testFiles.length === 0) {
    console.log("No test files found (*.test.mjs).");
    process.exit(0);
  }

  let totalPass = 0;
  let totalFail = 0;
  let totalSkip = 0;
  const allResults = [];

  for (const file of testFiles) {
    const filePath = resolve(__dirname, file);
    console.log(`--- ${file} ---`);
    const results = await runTestFile(filePath, config.port, serverAvailable);

    for (const r of results) {
      const icon =
        r.status === "PASS" ? "  PASS" : r.status === "SKIP" ? "  SKIP" : "  FAIL";
      const detail = r.error ? ` (${r.error})` : "";
      console.log(`${icon}: ${r.name}${detail}`);

      if (r.status === "PASS") totalPass++;
      else if (r.status === "FAIL") totalFail++;
      else totalSkip++;
    }
    console.log();
    allResults.push(...results);
  }

  // Summary
  console.log("========================================");
  console.log("  Summary");
  console.log("========================================");
  console.log(`  PASS:  ${totalPass}`);
  console.log(`  FAIL:  ${totalFail}`);
  console.log(`  SKIP:  ${totalSkip}`);
  console.log(`  TOTAL: ${totalPass + totalFail + totalSkip}`);
  console.log("========================================\n");

  // Cleanup
  if (childProcess) {
    console.log("Shutting down binary...");
    childProcess.kill("SIGTERM");
    // Give it a moment to exit gracefully
    await new Promise((r) => setTimeout(r, 1000));
    if (!childProcess.killed) {
      childProcess.kill("SIGKILL");
    }
  }

  // Exit code: 0 if all tests passed or were skipped, 1 if any failed
  process.exit(totalFail > 0 ? 1 : 0);
}

main().catch((err) => {
  console.error("Runner error:", err);
  process.exit(2);
});
