// Pure (no runes) helpers for the session list. Kept testable under
// `bun test` which doesn't transpile `.svelte.ts` runes.

import type { Session, SessionStatus } from "./types";

export const STATUS_ORDER: SessionStatus[] = [
  "waiting",
  "running",
  "plan",
  "idle",
  "done",
  "error",
];

export const STATUS_LABEL: Record<SessionStatus, string> = {
  running: "Running",
  waiting: "Waiting on you",
  plan: "Plan mode",
  idle: "Idle",
  done: "Done",
  error: "Error",
};

export function groupByStatus(sessions: readonly Session[]): Record<SessionStatus, Session[]> {
  const groups: Record<SessionStatus, Session[]> = {
    running: [],
    waiting: [],
    plan: [],
    idle: [],
    done: [],
    error: [],
  };
  for (const s of sessions) groups[s.status].push(s);
  for (const k of Object.keys(groups) as SessionStatus[]) {
    groups[k].sort((a, b) => b.lastActivity.localeCompare(a.lastActivity));
  }
  return groups;
}

export function countByStatus(sessions: readonly Session[]): Record<SessionStatus, number> {
  const c: Record<SessionStatus, number> = {
    running: 0,
    waiting: 0,
    plan: 0,
    idle: 0,
    done: 0,
    error: 0,
  };
  for (const s of sessions) c[s.status]++;
  return c;
}
