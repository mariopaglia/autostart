import { useEffect, useState } from "react";
import { AppShell } from "@/components/layout/AppShell";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useMonitorEvents } from "@/hooks/use-monitor-events";
import { notifyError } from "@/lib/notify";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";

async function loadInitialData() {
  await useSettingsStore.getState().load();
  await useProfilesStore.getState().load();
}

export function App() {
  const [ready, setReady] = useState(false);
  useMonitorEvents();

  useEffect(() => {
    loadInitialData()
      .then(() => {
        setReady(true);
      })
      .catch(notifyError);
  }, []);

  return (
    <TooltipProvider>
      {ready && <AppShell />}
      <Toaster position="bottom-right" />
    </TooltipProvider>
  );
}
