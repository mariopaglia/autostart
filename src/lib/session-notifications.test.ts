import { beforeEach, describe, expect, it } from "vitest";
import type { TimelineEntry } from "@/bindings/TimelineEntry";
import { i18n } from "@/i18n";
import { notificationFor, type NotificationContext } from "./session-notifications";

function entry(overrides: Partial<TimelineEntry>): TimelineEntry {
  return { timestampMs: 1, kind: "launched", itemId: "volanta", itemName: "Volanta", ...overrides };
}

const REAL_SESSION: NotificationContext = {
  t: i18n.t,
  isTestSession: false,
  showNotifications: true,
};

const launchFailure = entry({
  kind: "error",
  error: { kind: "executableNotFound", message: "C:\\Volanta.exe" },
});

describe.each(["pt-BR", "en"])("notificationFor (%s)", (language) => {
  beforeEach(async () => {
    await i18n.changeLanguage(language);
  });

  it("names the app whose launch failed and why", () => {
    const notification = notificationFor(launchFailure, REAL_SESSION);

    expect(notification?.title).toBe(
      i18n.t("notifications.launchFailedTitle", { name: "Volanta" }),
    );
    expect(notification?.body).toBe(i18n.t("errors.executableNotFound"));
  });

  it("announces the close delay with its seconds", () => {
    const notification = notificationFor(
      entry({ kind: "closeDelayed", itemId: undefined, itemName: undefined, detail: "60" }),
      REAL_SESSION,
    );

    expect(notification?.body).toBe(i18n.t("notifications.closeDelayedBody", { seconds: "60" }));
  });

  it("reports reopened apps and apps that kept crashing", () => {
    const relaunched = notificationFor(entry({ kind: "relaunched" }), REAL_SESSION);
    const gaveUp = notificationFor(
      entry({ kind: "error", error: { kind: "keptCrashing", message: "Volanta" } }),
      REAL_SESSION,
    );

    expect(relaunched?.title).toBe(i18n.t("notifications.relaunchedTitle", { name: "Volanta" }));
    expect(gaveUp?.title).toBe(i18n.t("notifications.gaveUpTitle", { name: "Volanta" }));
  });

  it("uses one key for every SimConnect timeout of a session", () => {
    const simConnect = (itemId: string) =>
      notificationFor(
        entry({
          kind: "error",
          itemId,
          error: { kind: "simConnectUnavailable", message: "" },
        }),
        REAL_SESSION,
      );

    expect(simConnect("a")?.key).toBe(simConnect("b")?.key);
  });
});

describe("notificationFor filters", () => {
  it("ignores successes and closing errors", () => {
    expect(notificationFor(entry({ kind: "launched" }), REAL_SESSION)).toBeNull();
    expect(notificationFor(entry({ kind: "closedGracefully" }), REAL_SESSION)).toBeNull();
    expect(
      notificationFor(
        entry({ kind: "error", error: { kind: "closeTimedOut", message: "" } }),
        REAL_SESSION,
      ),
    ).toBeNull();
  });

  it("stays silent in test sessions and when turned off", () => {
    expect(notificationFor(launchFailure, { ...REAL_SESSION, isTestSession: true })).toBeNull();
    expect(
      notificationFor(launchFailure, { ...REAL_SESSION, showNotifications: false }),
    ).toBeNull();
  });
});
