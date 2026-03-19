// WebSocket + HTTP client helper for E2E tests against Station Runtime IPC.
//
// Connects to the axum server on localhost:14200 (configurable).
// Provides HTTP GET/POST helpers and WebSocket event collection.

import WebSocket from "ws";
import http from "http";

/**
 * Make an HTTP request and parse the JSON response.
 * @param {string} method - HTTP method
 * @param {string} url - Full URL
 * @param {object|null} body - JSON body for POST/PUT
 * @returns {Promise<{status: number, data: any}>}
 */
function httpRequest(method, url, body = null) {
  return new Promise((resolve, reject) => {
    const parsed = new URL(url);
    const options = {
      hostname: parsed.hostname,
      port: parsed.port,
      path: parsed.pathname + parsed.search,
      method,
      headers: { "Content-Type": "application/json" },
      timeout: 10000,
    };

    const req = http.request(options, (res) => {
      let data = "";
      res.on("data", (chunk) => (data += chunk));
      res.on("end", () => {
        try {
          const parsed = JSON.parse(data);
          resolve({ status: res.statusCode, data: parsed });
        } catch {
          resolve({ status: res.statusCode, data });
        }
      });
    });

    req.on("error", reject);
    req.on("timeout", () => {
      req.destroy();
      reject(new Error(`HTTP ${method} ${url} timed out`));
    });

    if (body) {
      req.write(JSON.stringify(body));
    }
    req.end();
  });
}

/**
 * Create a test client connected to the Station Runtime IPC server.
 *
 * @param {number} port - Server port (default 14200)
 * @returns {Promise<TestClient>}
 */
export async function createClient(port = 14200) {
  const baseUrl = `http://127.0.0.1:${port}`;
  const wsUrl = `ws://127.0.0.1:${port}/ws/events`;

  // --- HTTP helpers ---

  async function get(path) {
    const { data } = await httpRequest("GET", `${baseUrl}${path}`);
    return data;
  }

  async function post(path, body = {}) {
    const { data } = await httpRequest("POST", `${baseUrl}${path}`, body);
    return data;
  }

  // --- WebSocket ---

  /** @type {Array<object>} All events received via WebSocket */
  const events = [];
  let wsOpen = false;

  const ws = new WebSocket(wsUrl);

  await new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      reject(new Error("WebSocket connection timed out (5s)"));
    }, 5000);

    ws.on("open", () => {
      wsOpen = true;
      // Send subscribe message to acknowledge connection
      ws.send(JSON.stringify({ type: "subscribe", filters: {} }));
      clearTimeout(timeout);
      resolve();
    });

    ws.on("error", (err) => {
      clearTimeout(timeout);
      reject(err);
    });
  });

  ws.on("message", (data) => {
    try {
      const parsed = JSON.parse(data.toString());
      events.push(parsed);
    } catch {
      // Ignore non-JSON messages
    }
  });

  // --- Event helpers ---

  /**
   * Wait for a specific event type to appear in the events array.
   * Polls at 50ms intervals.
   *
   * @param {string} type - The event_type.type value (e.g. "agent.state_changed")
   * @param {number} timeoutMs - Max wait time in ms (default 5000)
   * @returns {Promise<object>} The matching event
   */
  function waitForEvent(type, timeoutMs = 5000) {
    return new Promise((resolve, reject) => {
      const start = Date.now();
      const check = () => {
        const match = events.find(
          (e) =>
            e.event_type?.type === type ||
            e.type === type
        );
        if (match) {
          resolve(match);
          return;
        }
        if (Date.now() - start > timeoutMs) {
          reject(
            new Error(
              `Timed out waiting for event "${type}" after ${timeoutMs}ms. ` +
              `Received ${events.length} events: ${JSON.stringify(events.map(e => e.event_type?.type || e.type))}`
            )
          );
          return;
        }
        setTimeout(check, 50);
      };
      check();
    });
  }

  /**
   * Collect all events for a given duration.
   *
   * @param {number} durationMs - Collection window in ms
   * @returns {Promise<Array<object>>} Events collected during the window
   */
  function collectEvents(durationMs) {
    const startIdx = events.length;
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve(events.slice(startIdx));
      }, durationMs);
    });
  }

  /**
   * Clear the events buffer.
   */
  function clearEvents() {
    events.length = 0;
  }

  /**
   * Close the WebSocket connection.
   */
  function close() {
    if (ws.readyState === WebSocket.OPEN) {
      ws.close();
    }
    wsOpen = false;
  }

  return {
    get,
    post,
    events,
    waitForEvent,
    collectEvents,
    clearEvents,
    close,
    get isOpen() {
      return wsOpen && ws.readyState === WebSocket.OPEN;
    },
  };
}

/**
 * Check if a server is listening on a port.
 * @param {number} port
 * @param {number} timeoutMs
 * @returns {Promise<boolean>}
 */
export function isPortListening(port = 14200, timeoutMs = 2000) {
  return new Promise((resolve) => {
    const req = http.get(`http://127.0.0.1:${port}/health`, (res) => {
      res.resume(); // consume response
      resolve(true);
    });
    req.on("error", () => resolve(false));
    req.setTimeout(timeoutMs, () => {
      req.destroy();
      resolve(false);
    });
  });
}
