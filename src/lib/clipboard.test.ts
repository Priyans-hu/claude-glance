import { describe, it, expect } from "vitest";
import { resumeCommand } from "./clipboard";

describe("resumeCommand", () => {
  it("formats a resume command for a given session uuid", () => {
    expect(resumeCommand("111e1111-e29b-41d4-a716-446655440000")).toBe(
      "claude --resume 111e1111-e29b-41d4-a716-446655440000",
    );
  });

  it("preserves session ids that aren't uuids", () => {
    expect(resumeCommand("abc")).toBe("claude --resume abc");
  });
});
