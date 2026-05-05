// Svelte 5 runes-based reactive store for the session list. Components
// just read `sessionStore.sessions` and trust the store to keep them in
// sync with the Tauri side.

import { getSessions, subscribeToSessions } from "./api";
import { countByStatus, groupByStatus } from "./grouping";
import type { Session, SessionStatus } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";

class SessionStore {
  sessions = $state<Session[]>([]);
  loading = $state(true);
  error = $state<string | null>(null);
  initialized = false;

  private unlisten: UnlistenFn | null = null;

  /** Idempotent — safe to call from multiple components / on every mount. */
  async init(): Promise<void> {
    if (this.initialized) return;
    this.initialized = true;

    try {
      this.sessions = await getSessions();
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    } finally {
      this.loading = false;
    }

    try {
      this.unlisten = await subscribeToSessions((sessions) => {
        this.sessions = sessions;
      });
    } catch (err) {
      this.error = err instanceof Error ? err.message : String(err);
    }
  }

  destroy(): void {
    this.unlisten?.();
    this.unlisten = null;
    this.initialized = false;
  }

  groupedByStatus = $derived.by(
    (): Record<SessionStatus, Session[]> => groupByStatus(this.sessions),
  );

  counts = $derived.by((): Record<SessionStatus, number> => countByStatus(this.sessions));
}

export const sessionStore = new SessionStore();
