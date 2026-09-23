import type { TFunction } from "i18next";
import type { ErrorKind } from "@/bindings/ErrorKind";
import type { TimelineEntry } from "@/bindings/TimelineEntry";
import { translateError } from "@/lib/error-messages";

export interface SessionNotification {
  title: string;
  body: string;
  /** Entries sharing a key are notified once per session, e.g. one SimConnect timeout per item. */
  key: string;
}

export interface NotificationContext {
  t: TFunction;
  isTestSession: boolean;
  showNotifications: boolean;
}

const LAUNCH_ERROR_KINDS: readonly ErrorKind[] = [
  "executableNotFound",
  "launchFailed",
  "processNotDetected",
  "elevationDenied",
  "unsupported",
];

function forError(entry: TimelineEntry, t: TFunction): SessionNotification | null {
  const { error } = entry;
  if (!error) return null;
  const name = entry.itemName ?? "";

  if (error.kind === "simConnectUnavailable") {
    return {
      title: t("notifications.simConnectTitle"),
      body: t("notifications.simConnectBody"),
      key: "simConnectUnavailable",
    };
  }
  if (error.kind === "keptCrashing") {
    return {
      title: t("notifications.gaveUpTitle", { name }),
      body: t("notifications.gaveUpBody"),
      key: `gaveUp-${entry.itemId ?? ""}`,
    };
  }
  if (!LAUNCH_ERROR_KINDS.includes(error.kind)) return null;
  return {
    title: t("notifications.launchFailedTitle", { name }),
    body: translateError(t, error),
    key: `launchFailed-${entry.itemId ?? ""}-${String(entry.timestampMs)}`,
  };
}

/** Only events that need the pilot's attention while AutoStart sits in the tray. */
export function notificationFor(
  entry: TimelineEntry,
  { t, isTestSession, showNotifications }: NotificationContext,
): SessionNotification | null {
  if (!showNotifications || isTestSession) return null;

  switch (entry.kind) {
    case "error":
      return forError(entry, t);
    case "closeDelayed":
      return {
        title: t("notifications.closeDelayedTitle"),
        body: t("notifications.closeDelayedBody", { seconds: entry.detail ?? "" }),
        key: `closeDelayed-${String(entry.timestampMs)}`,
      };
    case "relaunched":
      return {
        title: t("notifications.relaunchedTitle", { name: entry.itemName ?? "" }),
        body: t("notifications.relaunchedBody"),
        key: `relaunched-${entry.itemId ?? ""}-${String(entry.timestampMs)}`,
      };
    default:
      return null;
  }
}
