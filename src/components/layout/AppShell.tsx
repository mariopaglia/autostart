import { LogsScreen } from "@/screens/LogsScreen";
import { MainScreen } from "@/screens/MainScreen";
import { OnboardingWizard } from "@/screens/OnboardingWizard";
import { SettingsScreen } from "@/screens/SettingsScreen";
import { useUiStore, type Screen } from "@/stores/ui-store";
import { Sidebar } from "./Sidebar";

const SCREENS: Record<Screen, () => React.JSX.Element | null> = {
  main: MainScreen,
  logs: LogsScreen,
  settings: SettingsScreen,
};

export function AppShell() {
  const screen = useUiStore((state) => state.screen);
  const CurrentScreen = SCREENS[screen];

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <main className="flex min-w-0 flex-1 flex-col">
        <CurrentScreen />
      </main>
      <OnboardingWizard />
    </div>
  );
}
