import { useEffect, useState } from "react";
import { UpdateDialog } from "@/components/common/UpdateDialog";
import { AppShell } from "@/components/layout/AppShell";
import { Toaster } from "@/components/ui/sonner";
import { TooltipProvider } from "@/components/ui/tooltip";
import { useAppearance } from "@/hooks/use-appearance";
import { useMonitorEvents } from "@/hooks/use-monitor-events";
import { useSettingsEvents } from "@/hooks/use-settings-events";
import { notifyError } from "@/lib/notify";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";
import { useUpdatesStore } from "@/stores/updates-store";

async function loadInitialData() {
  await useSettingsStore.getState().load();
  await useProfilesStore.getState().load();
}

function checkUpdatesOnStartup() {
  if (useSettingsStore.getState().settings?.checkUpdatesOnStartup) {
    void useUpdatesStore.getState().check({ silent: true });
  }
}

export function App() {
  const [ready, setReady] = useState(false);
  const theme = useAppearance();
  useMonitorEvents();
  useSettingsEvents();

  useEffect(() => {
    loadInitialData()
      .then(() => {
        setReady(true);
        checkUpdatesOnStartup();
      })
      .catch(notifyError);
  }, []);

  return (
    <TooltipProvider>
      {ready && <AppShell />}
      <UpdateDialog />
      <Toaster position="bottom-right" theme={theme} />
    </TooltipProvider>
  );
}
