/**
 * WebSocket mock for Playwright E2E tests.
 *
 * Overrides `window.WebSocket` so the app's useEventStream hook never
 * attempts a real connection. Tests can emit events programmatically.
 */
import type { Page } from "@playwright/test";

/**
 * Inject WebSocket mock. Must be called BEFORE page.goto().
 */
export async function injectWsMock(page: Page) {
  await page.addInitScript(() => {
    // Store captured sockets for programmatic event emission
    const _sockets: Array<{
      url: string;
      onopen: ((ev: Event) => void) | null;
      onmessage: ((ev: MessageEvent) => void) | null;
      onclose: ((ev: CloseEvent) => void) | null;
      onerror: ((ev: Event) => void) | null;
      readyState: number;
      send: (data: string) => void;
      close: () => void;
    }> = [];

    (window as any).__WS_MOCK_SOCKETS__ = _sockets;

    // Replace global WebSocket
    (window as any)._OriginalWebSocket = window.WebSocket;

    class MockWebSocket {
      static CONNECTING = 0;
      static OPEN = 1;
      static CLOSING = 2;
      static CLOSED = 3;

      CONNECTING = 0;
      OPEN = 1;
      CLOSING = 2;
      CLOSED = 3;

      url: string;
      readyState = 0; // CONNECTING
      onopen: ((ev: Event) => void) | null = null;
      onmessage: ((ev: MessageEvent) => void) | null = null;
      onclose: ((ev: CloseEvent) => void) | null = null;
      onerror: ((ev: Event) => void) | null = null;

      constructor(url: string, _protocols?: string | string[]) {
        this.url = url;
        const self = this;

        // Register so tests can access it
        const socketRecord = {
          url,
          get onopen() { return self.onopen; },
          get onmessage() { return self.onmessage; },
          get onclose() { return self.onclose; },
          get onerror() { return self.onerror; },
          get readyState() { return self.readyState; },
          send: (data: string) => self.send(data),
          close: () => self.close(),
        };
        _sockets.push(socketRecord);

        // Auto-open after microtask (simulates async connect)
        queueMicrotask(() => {
          self.readyState = 1; // OPEN
          if (self.onopen) {
            self.onopen(new Event("open"));
          }
        });
      }

      send(_data: string) {
        // No-op in mock — tests don't need to verify outgoing messages
      }

      close() {
        this.readyState = 3; // CLOSED
        if (this.onclose) {
          this.onclose(new CloseEvent("close", { wasClean: true, code: 1000, reason: "" }));
        }
      }

      addEventListener(_type: string, _listener: EventListener) {
        // Minimal stub
      }

      removeEventListener(_type: string, _listener: EventListener) {
        // Minimal stub
      }
    }

    (window as any).WebSocket = MockWebSocket;
  });
}

/**
 * Emit a WebSocket event from test code into the app.
 *
 * The event payload matches the `EventMessage` interface from useEventStream.
 */
export async function emitWsEvent(
  page: Page,
  event: {
    id?: string;
    agent_id: string;
    timestamp?: string;
    type: string;
    payload: Record<string, unknown>;
  },
) {
  await page.evaluate((evt) => {
    const sockets = (window as any).__WS_MOCK_SOCKETS__ as Array<{
      onmessage: ((ev: MessageEvent) => void) | null;
    }>;

    const data = {
      id: evt.id ?? `evt-${Date.now()}`,
      agent_id: evt.agent_id,
      timestamp: evt.timestamp ?? new Date().toISOString(),
      type: evt.type,
      payload: evt.payload,
    };

    // Send to all open sockets
    for (const sock of sockets) {
      if (sock.onmessage) {
        sock.onmessage(new MessageEvent("message", { data: JSON.stringify(data) }));
      }
    }
  }, event);
}
