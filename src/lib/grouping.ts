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

/** Display bucket — same four statuses plus a "recent" age-based bucket. */
export type SessionBucket = SessionStatus | "recent";

export const BUCKET_ORDER: SessionBucket[] = ["working", "waiting", "plan", "idle", "recent"];

export const BUCKET_LABEL: Record<SessionBucket, string> = {
  working: "WORKING",
  waiting: "WAITING ON YOU",
  plan: "PLAN MODE",
  idle: "IDLE",
  recent: "RECENT — last 7 days",
};

const VISIBLE_AGE_LIMIT_MS = 2 * 24 * 60 * 60 * 1000; // 2 days
const RECENT_AGE_LIMIT_MS = 7 * 24 * 60 * 60 * 1000; // 7 days

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

/**
 * Bucket sessions for display. Sessions ≤2 days old land in their status
 * bucket; sessions 2–7 days old land in "recent" (the collapsed group);
 * sessions >7 days old should already have been filtered by the scanner —
 * any that slip through are dropped here defensively.
 */
export function bucketSessions(
  sessions: readonly Session[],
  now: Date = new Date(),
): Record<SessionBucket, Session[]> {
  const buckets: Record<SessionBucket, Session[]> = {
    working: [],
    waiting: [],
    plan: [],
    idle: [],
    recent: [],
  };

  const nowMs = now.getTime();
  for (const s of sessions) {
    const ts = s.lastActivity ? new Date(s.lastActivity).getTime() : NaN;
    if (Number.isNaN(ts)) {
      buckets[s.status].push(s);
      continue;
    }
    const ageMs = nowMs - ts;
    if (ageMs > RECENT_AGE_LIMIT_MS) continue; // belt-and-braces: scanner filters these
    if (ageMs > VISIBLE_AGE_LIMIT_MS) {
      buckets.recent.push(s);
    } else {
      buckets[s.status].push(s);
    }
  }

  for (const k of BUCKET_ORDER) {
    buckets[k].sort((a, b) => b.lastActivity.localeCompare(a.lastActivity));
  }
  return buckets;
}
