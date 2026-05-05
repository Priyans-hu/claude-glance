import { describe, it, expect } from "vitest";
import { STATUS_ORDER, groupByStatus, countByStatus } from "./grouping";
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
