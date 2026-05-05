// Thin wrapper around Tauri's `invoke`/`event` so the rest of the app
// doesn't have to know about the IPC plumbing.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { Session } from "./types";

export const SESSIONS_CHANGED_EVENT = "sessions_changed";

export async function getSessions(): Promise<Session[]> {
  return invoke<Session[]>("list_sessions");
}

/**
 * Subscribe to live session updates. Returns an unlisten function.
 * The callback receives the full snapshot every time something changes.
 */
export async function subscribeToSessions(cb: (sessions: Session[]) => void): Promise<UnlistenFn> {
  return listen<Session[]>(SESSIONS_CHANGED_EVENT, (event) => {
    cb(event.payload);
  });
}
