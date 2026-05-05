// Svelte 5 runes-based reactive store for the session list. Components
// just read `sessionStore.sessions` and trust the store to keep them in
// sync with the Tauri side.

import { getSessions, subscribeToSessions } from "./api";
import { bucketSessions, countByStatus, groupByStatus } from "./grouping";
import type { SessionBucket } from "./grouping";
import type { Session, SessionStatus } from "./types";
import type { UnlistenFn } from "@tauri-apps/api/event";

const SHOW_RECENT_STORAGE_KEY = "cg.showRecent";

function readShowRecent(): boolean {
  if (typeof localStorage === "undefined") return false;
  try {
    return localStorage.getItem(SHOW_RECENT_STORAGE_KEY) === "true";
  } catch {
    return false;
  }
}

function writeShowRecent(value: boolean): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(SHOW_RECENT_STORAGE_KEY, String(value));
  } catch {
    /* localStorage disabled — fall back to in-memory only */
  }
}

class SessionStore {
  sessions = $state<Session[]>([]);
  loading = $state(true);
  error = $state<string | null>(null);
  /** Whether the Recent (2-7d) group is expanded. Persisted in localStorage. */
  showRecent = $state(readShowRecent());
  /** Volatile — the row whose detail strip is open. Not persisted. */
  expandedSessionId = $state<string | null>(null);
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

  toggleShowRecent(): void {
    this.showRecent = !this.showRecent;
    writeShowRecent(this.showRecent);
  }

  toggleExpanded(sessionId: string): void {
    this.expandedSessionId = this.expandedSessionId === sessionId ? null : sessionId;
  }

  groupedByStatus = $derived.by(
    (): Record<SessionStatus, Session[]> => groupByStatus(this.sessions),
  );

  bucketed = $derived.by((): Record<SessionBucket, Session[]> => bucketSessions(this.sessions));

  counts = $derived.by((): Record<SessionStatus, number> => countByStatus(this.sessions));
}

export const sessionStore = new SessionStore();
