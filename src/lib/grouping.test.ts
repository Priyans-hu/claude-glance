import { describe, it, expect } from "vitest";
import {
  STATUS_ORDER,
  BUCKET_ORDER,
  groupByStatus,
  countByStatus,
  bucketSessions,
} from "./grouping";
import type { Session } from "./types";

function s(id: string, status: Session["status"], lastActivity: string): Session {
  return {
    id,
    cwd: "/x",
    project: "x",
    branch: null,
    title: id,
    status,
    currentTool: null,
    permissionMode: null,
    subagentCount: 0,
    lastActivity,
    tokens: 0,
  };
}

function isoMinusHours(now: Date, hours: number): string {
  return new Date(now.getTime() - hours * 60 * 60 * 1000).toISOString();
}

describe("STATUS_ORDER", () => {
  it("renders working first so active sessions surface to the top", () => {
    expect(STATUS_ORDER[0]).toBe("working");
  });

  it("places waiting second so attention-needed sessions appear above plan/idle", () => {
    expect(STATUS_ORDER[1]).toBe("waiting");
  });

  it("includes every status", () => {
    expect(new Set(STATUS_ORDER)).toEqual(new Set(["working", "waiting", "plan", "idle"]));
  });
});

describe("groupByStatus", () => {
  it("buckets sessions by status and sorts each group most-recent first", () => {
    const groups = groupByStatus([
      s("a", "working", "2026-01-01T00:00:00Z"),
      s("b", "working", "2026-01-02T00:00:00Z"),
      s("c", "idle", "2026-01-03T00:00:00Z"),
    ]);
    expect(groups.working.map((x) => x.id)).toEqual(["b", "a"]);
    expect(groups.idle.map((x) => x.id)).toEqual(["c"]);
    expect(groups.plan).toEqual([]);
  });
});

describe("countByStatus", () => {
  it("returns counts for each status", () => {
    const counts = countByStatus([
      s("a", "working", "2026-01-01T00:00:00Z"),
      s("b", "working", "2026-01-02T00:00:00Z"),
      s("c", "waiting", "2026-01-03T00:00:00Z"),
    ]);
    expect(counts.working).toBe(2);
    expect(counts.waiting).toBe(1);
    expect(counts.idle).toBe(0);
  });
});

describe("BUCKET_ORDER", () => {
  it("ends with the collapsed recent group", () => {
    expect(BUCKET_ORDER[BUCKET_ORDER.length - 1]).toBe("recent");
  });

  it("places working/waiting/plan/idle before recent in that order", () => {
    expect(BUCKET_ORDER).toEqual(["working", "waiting", "plan", "idle", "recent"]);
  });
});

describe("bucketSessions", () => {
  const now = new Date("2026-05-10T12:00:00Z");

  it("puts sessions <=2d old in their status bucket", () => {
    const buckets = bucketSessions(
      [
        s("a", "waiting", isoMinusHours(now, 1)),
        s("b", "idle", isoMinusHours(now, 24)),
        s("c", "working", isoMinusHours(now, 47)),
      ],
      now,
    );
    expect(buckets.waiting.map((x) => x.id)).toEqual(["a"]);
    expect(buckets.idle.map((x) => x.id)).toEqual(["b"]);
    expect(buckets.working.map((x) => x.id)).toEqual(["c"]);
    expect(buckets.recent).toEqual([]);
  });

  it("puts sessions in the 2-7d window in the recent bucket regardless of status", () => {
    const buckets = bucketSessions(
      [s("a", "waiting", isoMinusHours(now, 49)), s("b", "idle", isoMinusHours(now, 24 * 6))],
      now,
    );
    expect(buckets.waiting).toEqual([]);
    expect(buckets.idle).toEqual([]);
    expect(buckets.recent.map((x) => x.id)).toEqual(["a", "b"]);
  });

  it("drops sessions >7d old defensively", () => {
    const buckets = bucketSessions([s("a", "idle", isoMinusHours(now, 24 * 8))], now);
    expect(buckets.recent).toEqual([]);
    expect(buckets.idle).toEqual([]);
  });

  it("sorts each bucket most-recent first", () => {
    const buckets = bucketSessions(
      [
        s("old", "waiting", isoMinusHours(now, 5)),
        s("new", "waiting", isoMinusHours(now, 1)),
        s("mid", "waiting", isoMinusHours(now, 3)),
      ],
      now,
    );
    expect(buckets.waiting.map((x) => x.id)).toEqual(["new", "mid", "old"]);
  });
});
