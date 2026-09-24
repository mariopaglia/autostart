import type { SessionLog } from "@/bindings/SessionLog";

export function sessionKey(session: SessionLog): string {
  return `${session.profileId}-${String(session.startedAtMs)}`;
}

/** A finished session is shown from the history, so a cleared history stays empty. */
export function sessionList(live: SessionLog | null, history: readonly SessionLog[]): SessionLog[] {
  const inProgress = live?.endedAtMs === null ? live : null;
  if (!inProgress || history.some((stored) => sessionKey(stored) === sessionKey(inProgress))) {
    return [...history];
  }
  return [inProgress, ...history];
}

export function errorCount(session: SessionLog): number {
  return session.entries.filter((entry) => entry.error !== undefined).length;
}

/** A pick that no longer exists (e.g. after clearing) falls back to the newest session. */
export function selectedSession(
  sessions: readonly SessionLog[],
  pickedKey: string | null,
): SessionLog | undefined {
  return sessions.find((session) => sessionKey(session) === pickedKey) ?? sessions[0];
}
