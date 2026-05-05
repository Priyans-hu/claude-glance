// Pure (no runes) helpers for the session list. Kept testable under
// `bun test` which doesn't transpile `.svelte.ts` runes.

import type { Session, SessionStatus } from "./types";

export const STATUS_ORDER: SessionStatus[] = ["working", "waiting", "plan", "idle"];

export const STATUS_LABEL: Record<SessionStatus, string> = {
  working: "WORKING",
  waiting: "WAITING ON YOU",
  plan: "PLAN MODE",
  idle: "IDLE",
};

export function groupByStatus(sessions: readonly Session[]): Record<SessionStatus, Session[]> {
  const groups: Record<SessionStatus, Session[]> = {
    working: [],
    waiting: [],
    plan: [],
    idle: [],
  };
  for (const s of sessions) groups[s.status].push(s);
  for (const k of Object.keys(groups) as SessionStatus[]) {
    groups[k].sort((a, b) => b.lastActivity.localeCompare(a.lastActivity));
  }
  return groups;
}

export function countByStatus(sessions: readonly Session[]): Record<SessionStatus, number> {
  const c: Record<SessionStatus, number> = {
    working: 0,
    waiting: 0,
    plan: 0,
    idle: 0,
  };
  for (const s of sessions) c[s.status]++;
  return c;
}
