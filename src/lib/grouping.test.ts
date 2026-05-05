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
  it("renders waiting first so it's the most attention-grabbing group", () => {
    expect(STATUS_ORDER[0]).toBe("waiting");
  });

  it("includes every status", () => {
    expect(new Set(STATUS_ORDER)).toEqual(
      new Set(["running", "waiting", "plan", "idle", "done", "error"]),
    );
  });
});

describe("groupByStatus", () => {
  it("buckets sessions by status and sorts each group most-recent first", () => {
    const groups = groupByStatus([
      s("a", "running", "2026-01-01T00:00:00Z"),
      s("b", "running", "2026-01-02T00:00:00Z"),
      s("c", "idle", "2026-01-03T00:00:00Z"),
    ]);
    expect(groups.running.map((x) => x.id)).toEqual(["b", "a"]);
    expect(groups.idle.map((x) => x.id)).toEqual(["c"]);
    expect(groups.done).toEqual([]);
  });
});

describe("countByStatus", () => {
  it("returns counts for each status", () => {
    const counts = countByStatus([
      s("a", "running", "2026-01-01T00:00:00Z"),
      s("b", "running", "2026-01-02T00:00:00Z"),
      s("c", "waiting", "2026-01-03T00:00:00Z"),
    ]);
    expect(counts.running).toBe(2);
    expect(counts.waiting).toBe(1);
    expect(counts.idle).toBe(0);
  });
});
