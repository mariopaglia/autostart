import { describe, expect, it } from "vitest";
import type { SessionLog } from "@/bindings/SessionLog";
import { errorCount, selectedSession, sessionKey, sessionList } from "./session-history";

function session(startedAtMs: number, overrides: Partial<SessionLog> = {}): SessionLog {
  return {
    profileId: "profile",
    profileName: "MSFS",
    isTest: false,
    startedAtMs,
    endedAtMs: startedAtMs + 1,
    entries: [],
    ...overrides,
  };
}

describe("session history", () => {
  it("puts a live session in front of the stored ones", () => {
    const live = session(3, { endedAtMs: null });

    expect(sessionList(live, [session(2), session(1)]).map((s) => s.startedAtMs)).toEqual([
      3, 2, 1,
    ]);
  });

  it("shows a finished session only from the history", () => {
    expect(sessionList(session(2), [session(1)]).map((s) => s.startedAtMs)).toEqual([1]);
    expect(sessionList(session(2), [])).toEqual([]);
  });

  it("counts the entries with an error", () => {
    const failed = session(1, {
      entries: [
        { timestampMs: 1, kind: "launched" },
        { timestampMs: 2, kind: "error", error: { kind: "launchFailed", message: "x" } },
        { timestampMs: 3, kind: "error", error: { kind: "executableNotFound", message: "y" } },
      ],
    });

    expect(errorCount(failed)).toBe(2);
  });

  it("keeps the picked session and falls back to the newest one", () => {
    const sessions = [session(2), session(1)];

    expect(selectedSession(sessions, sessionKey(session(1)))?.startedAtMs).toBe(1);
    expect(selectedSession(sessions, null)?.startedAtMs).toBe(2);
    expect(selectedSession(sessions, "gone")?.startedAtMs).toBe(2);
  });
});
