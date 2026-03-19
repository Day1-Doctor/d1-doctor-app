import { useEffect } from "react";
import { useAuthStore } from "../stores/authStore";

/**
 * Listen for deep link events from the Tauri deep-link plugin.
 *
 * When the OS opens `day1copilot://auth/callback?token=...`, the plugin
 * emits a `deep-link://new-url` event. This hook picks it up and processes
 * the OAuth callback through the auth store.
 */
export function useDeepLink() {
  const handleDeepLinkCallback = useAuthStore((s) => s.handleDeepLinkCallback);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    async function setup() {
      try {
        // Tauri v2 deep-link plugin: listen via the event system
        const { listen } = await import("@tauri-apps/api/event");
        const unlistenFn = await listen<string[]>("deep-link://new-url", (event) => {
          const urls = event.payload;
          if (!urls || urls.length === 0) return;

          for (const url of urls) {
            // Only handle auth callback URLs
            if (url.startsWith("day1copilot://auth/callback")) {
              handleDeepLinkCallback(url);
              break;
            }
          }
        });
        unlisten = unlistenFn;
      } catch {
        // Deep link plugin not available (e.g. dev mode without plugin)
        // This is non-fatal — users can still authenticate via manual flow
      }
    }

    setup();

    return () => {
      if (unlisten) unlisten();
    };
  }, [handleDeepLinkCallback]);
}
