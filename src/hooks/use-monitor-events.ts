import { useEffect } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { commands, monitorEvents } from "@/lib/tauri";
import { useLogsStore } from "@/stores/logs-store";
import { useMonitorStore } from "@/stores/monitor-store";

async function refreshSessionLog() {
  useMonitorStore.getState().setSessionLog(await commands.getSessionLog());
}

export function useMonitorEvents() {
  useEffect(() => {
    const { setSnapshot } = useMonitorStore.getState();
    const subscriptions: Promise<UnlistenFn>[] = [
      monitorEvents.onState(setSnapshot),
      monitorEvents.onLog((entry) => {
        void refreshSessionLog();
        void useLogsStore.getState().load();
        if (entry.kind === "sessionEnded") void useLogsStore.getState().load();
      }),
    ];

    void commands.getMonitorState().then(setSnapshot);
    void refreshSessionLog();
    void useLogsStore.getState().load();

    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => {
          unlisten();
        });
      }
    };
  }, []);
}
