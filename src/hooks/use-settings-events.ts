import { useEffect } from "react";
import { settingsEvents } from "@/lib/tauri";
import { useSettingsStore } from "@/stores/settings-store";

export function useSettingsEvents() {
  useEffect(() => {
    const subscription = settingsEvents.onChanged(useSettingsStore.getState().replace);
    return () => {
      void subscription.then((unlisten) => {
        unlisten();
      });
    };
  }, []);
}
