import { describe, it, expect } from "vitest";
import type { SessionStatus } from "./types";

describe("SessionStatus", () => {
  it("includes the six statuses claude-glance groups by", () => {
    const statuses: SessionStatus[] = ["running", "waiting", "plan", "idle", "done", "error"];
    expect(statuses).toHaveLength(6);
  });
});
