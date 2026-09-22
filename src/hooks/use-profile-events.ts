import { useEffect } from "react";
import { profileEvents } from "@/lib/tauri";
import { useProfilesStore } from "@/stores/profiles-store";

export function useProfileEvents() {
  useEffect(() => {
    const subscription = profileEvents.onChanged(useProfilesStore.getState().replace);
    return () => {
      void subscription.then((unlisten) => {
        unlisten();
      });
    };
  }, []);
}
