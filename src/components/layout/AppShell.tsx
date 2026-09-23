import { LogsScreen } from "@/screens/LogsScreen";
import { MainScreen } from "@/screens/MainScreen";
import { OnboardingWizard } from "@/screens/OnboardingWizard";
import { SettingsScreen } from "@/screens/SettingsScreen";
import { useMonitorStore } from "@/stores/monitor-store";
import { useUiStore, type Screen } from "@/stores/ui-store";
import { CloseCountdown } from "./CloseCountdown";
import { Sidebar } from "./Sidebar";

const SCREENS: Record<Screen, () => React.JSX.Element | null> = {
  main: MainScreen,
  logs: LogsScreen,
  settings: SettingsScreen,
};

export function AppShell() {
  const screen = useUiStore((state) => state.screen);
  const CurrentScreen = SCREENS[screen];
  const closesAtMs = useMonitorStore((state) =>
    state.snapshot.state === "closePending" ? state.snapshot.closesAtMs : null,
  );

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <main className="flex min-w-0 flex-1 flex-col">
        {closesAtMs !== null && <CloseCountdown closesAtMs={closesAtMs} />}
        <CurrentScreen />
      </main>
      <OnboardingWizard />
    </div>
  );
}
