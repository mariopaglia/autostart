import { LayoutGrid, ScrollText, Settings as SettingsIcon, Rocket } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ProfileList } from "@/components/profiles/ProfileList";
import { Separator } from "@/components/ui/separator";
import { cn } from "@/lib/utils";
import { useUiStore, type Screen } from "@/stores/ui-store";

const NAVIGATION: { screen: Screen; icon: LucideIcon }[] = [
  { screen: "main", icon: LayoutGrid },
  { screen: "logs", icon: ScrollText },
  { screen: "settings", icon: SettingsIcon },
];

export function Sidebar() {
  const { t } = useTranslation();
  const { screen: current, setScreen } = useUiStore();

  return (
    <aside className="flex w-60 shrink-0 flex-col gap-4 border-r bg-sidebar p-3 text-sidebar-foreground">
      <div className="flex items-center gap-2 px-2 pt-1">
        <span className="flex size-8 items-center justify-center rounded-lg bg-primary text-primary-foreground">
          <Rocket className="size-4" />
        </span>
        <span className="text-lg font-semibold tracking-tight">AutoStart</span>
      </div>

      <nav className="flex flex-col gap-1">
        {NAVIGATION.map(({ screen, icon: Icon }) => (
          <button
            key={screen}
            type="button"
            aria-current={screen === current ? "page" : undefined}
            onClick={() => {
              setScreen(screen);
            }}
            className={cn(
              "flex items-center gap-2 rounded-lg px-3 py-2 text-sm outline-none transition-colors",
              "hover:bg-sidebar-accent focus-visible:ring-2 focus-visible:ring-ring",
              screen === current && "bg-sidebar-accent font-medium",
            )}
          >
            <Icon className="size-4" />
            {t(`nav.${screen}`)}
          </button>
        ))}
      </nav>

      <Separator />

      <ProfileList />
    </aside>
  );
}
