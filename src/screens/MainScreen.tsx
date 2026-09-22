import { useEffect, useState } from "react";
import { AppWindow, Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { Profile } from "@/bindings/Profile";
import { ElevationBanner } from "@/components/common/ElevationBanner";
import { EmptyItems } from "@/components/items/EmptyItems";
import { ItemFormDialog, type ItemFormTarget } from "@/components/items/ItemFormDialog";
import { ItemList } from "@/components/items/ItemList";
import { TopBar } from "@/components/layout/TopBar";
import { Button } from "@/components/ui/button";
import { commands } from "@/lib/tauri";
import { supportsSimConnect } from "@/lib/trigger-presets";
import { useProfilesStore, useSelectedProfile } from "@/stores/profiles-store";

function useMissingExecutables(profile: Profile | undefined): ReadonlySet<string> {
  const [missing, setMissing] = useState<ReadonlySet<string>>(new Set());
  const paths = profile?.items.flatMap((item) => (item.type === "app" ? [item.exePath] : [])) ?? [];
  const key = paths.join("|");

  useEffect(() => {
    let cancelled = false;
    void commands.findMissingExecutables(key ? key.split("|") : []).then((result) => {
      if (!cancelled) setMissing(new Set(result));
    });
    return () => {
      cancelled = true;
    };
  }, [key]);

  return missing;
}

export function MainScreen() {
  const { t } = useTranslation();
  const profile = useSelectedProfile();
  const save = useProfilesStore((state) => state.save);
  const missingExecutables = useMissingExecutables(profile);
  const [formTarget, setFormTarget] = useState<ItemFormTarget | null>(null);

  if (!profile) return null;

  function saveItems(items: LaunchItem[]) {
    if (profile) void save({ ...profile, items });
  }

  function saveItem(item: LaunchItem) {
    if (!profile) return;
    const exists = profile.items.some((candidate) => candidate.id === item.id);
    saveItems(
      exists
        ? profile.items.map((candidate) => (candidate.id === item.id ? item : candidate))
        : [...profile.items, item],
    );
    setFormTarget(null);
  }

  const openCreate = (type: LaunchItem["type"]) => {
    setFormTarget({ mode: "create", type });
  };

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <TopBar profile={profile} />

      <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
        <ElevationBanner />
        {profile.items.length === 0 ? (
          <EmptyItems
            onAddApp={() => {
              openCreate("app");
            }}
            onAddUrl={() => {
              openCreate("url");
            }}
          />
        ) : (
          <>
            <div className="flex justify-end gap-2">
              <Button
                size="sm"
                onClick={() => {
                  openCreate("app");
                }}
              >
                <AppWindow />
                {t("items.addApp")}
              </Button>
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  openCreate("url");
                }}
              >
                <Globe />
                {t("items.addUrl")}
              </Button>
            </div>
            <ItemList
              profileId={profile.id}
              items={profile.items}
              missingExecutables={missingExecutables}
              onChange={saveItems}
              onEdit={(item) => {
                setFormTarget({ mode: "edit", item });
              }}
            />
          </>
        )}
      </section>

      <ItemFormDialog
        target={formTarget}
        simConnectSupported={supportsSimConnect(profile.trigger)}
        onSave={saveItem}
        onClose={() => {
          setFormTarget(null);
        }}
      />
    </div>
  );
}
