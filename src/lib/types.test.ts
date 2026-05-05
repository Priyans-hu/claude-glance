import { describe, it, expect } from "vitest";
import type { SessionStatus } from "./types";

describe("SessionStatus", () => {
  it("includes the four statuses claude-glance groups by", () => {
    const statuses: SessionStatus[] = ["working", "waiting", "plan", "idle"];
    expect(statuses).toHaveLength(4);
  });
});
