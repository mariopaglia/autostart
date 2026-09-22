import { useEffect } from "react";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { commands, monitorEvents } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";

async function refreshSessionLog() {
  useMonitorStore.getState().setSessionLog(await commands.getSessionLog());
}

export function useMonitorEvents() {
  useEffect(() => {
    const { setSnapshot } = useMonitorStore.getState();
    const subscriptions: Promise<UnlistenFn>[] = [
      monitorEvents.onState(setSnapshot),
      monitorEvents.onLog(() => void refreshSessionLog()),
    ];

    void commands.getMonitorState().then(setSnapshot);
    void refreshSessionLog();

    return () => {
      for (const subscription of subscriptions) {
        void subscription.then((unlisten) => {
          unlisten();
        });
      }
    };
  }, []);
}
