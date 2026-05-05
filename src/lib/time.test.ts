import { describe, it, expect } from "vitest";
import { formatRelativeTime } from "./time";

describe("formatRelativeTime", () => {
  const now = new Date("2026-01-01T12:00:00Z");

  it("returns 'now' for very recent timestamps", () => {
    expect(formatRelativeTime("2026-01-01T11:59:58Z", now)).toBe("now");
  });

  it("formats seconds", () => {
    expect(formatRelativeTime("2026-01-01T11:59:30Z", now)).toBe("30s ago");
  });

  it("formats minutes", () => {
    expect(formatRelativeTime("2026-01-01T11:55:00Z", now)).toBe("5m ago");
  });

  it("formats hours", () => {
    expect(formatRelativeTime("2026-01-01T09:00:00Z", now)).toBe("3h ago");
  });

  it("formats days", () => {
    expect(formatRelativeTime("2025-12-30T12:00:00Z", now)).toBe("2d ago");
  });

  it("handles empty/invalid", () => {
    expect(formatRelativeTime("", now)).toBe("—");
    expect(formatRelativeTime("not a date", now)).toBe("—");
  });
});
