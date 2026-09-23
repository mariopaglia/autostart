import { useEffect } from "react";
import { i18n } from "@/i18n";
import { notificationFor } from "@/lib/session-notifications";
import { monitorEvents, systemNotifications } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";
import { useSettingsStore } from "@/stores/settings-store";

/** Mirrors the timeline into Windows notifications, since the window is usually hidden. */
export function useSessionNotifications() {
  useEffect(() => {
    const sentKeys = new Set<string>();
    const subscription = monitorEvents.onLog((entry) => {
      if (entry.kind === "sessionStarted") sentKeys.clear();

      const notification = notificationFor(entry, {
        t: i18n.t,
        isTestSession: useMonitorStore.getState().snapshot.isTestSession,
        showNotifications: useSettingsStore.getState().settings?.showNotifications ?? false,
      });
      if (!notification || sentKeys.has(notification.key)) return;
      sentKeys.add(notification.key);
      void systemNotifications.send(notification.title, notification.body);
    });

    return () => {
      void subscription.then((unlisten) => {
        unlisten();
      });
    };
  }, []);
}
