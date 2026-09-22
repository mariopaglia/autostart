import { MainScreen } from "@/screens/MainScreen";
import { PlaceholderScreen } from "@/screens/PlaceholderScreen";
import { useUiStore } from "@/stores/ui-store";
import { Sidebar } from "./Sidebar";

export function AppShell() {
  const screen = useUiStore((state) => state.screen);

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar />
      <main className="flex min-w-0 flex-1 flex-col">
        {screen === "main" ? <MainScreen /> : <PlaceholderScreen screen={screen} />}
      </main>
    </div>
  );
}
