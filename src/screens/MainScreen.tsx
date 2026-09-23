import { useEffect, useState } from "react";
import { AppWindow, Globe } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { Profile } from "@/bindings/Profile";
import type { RejectedDrop } from "@/bindings/RejectedDrop";
import { ElevationBanner } from "@/components/common/ElevationBanner";
import { AppPickerDialog } from "@/components/items/AppPickerDialog";
import { DropOverlay } from "@/components/items/DropOverlay";
import { EmptyItems } from "@/components/items/EmptyItems";
import { ItemFormDialog, type ItemFormTarget } from "@/components/items/ItemFormDialog";
import { ItemList } from "@/components/items/ItemList";
import { TopBar } from "@/components/layout/TopBar";
import { Button } from "@/components/ui/button";
import { useFileDrop } from "@/hooks/use-file-drop";
import { useIsElevated } from "@/hooks/use-is-elevated";
import { planDrop } from "@/lib/drop-plan";
import { notifyError, notifySuccess, notifyWarning } from "@/lib/notify";
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

function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function MainScreen() {
  const { t } = useTranslation();
  const profile = useSelectedProfile();
  const save = useProfilesStore((state) => state.save);
  const missingExecutables = useMissingExecutables(profile);
  const isElevated = useIsElevated();
  const [formTarget, setFormTarget] = useState<ItemFormTarget | null>(null);
  const [pickerOpen, setPickerOpen] = useState(false);

  function saveItems(items: LaunchItem[]) {
    if (profile) void save({ ...profile, items });
  }

  function reportRejected(rejected: RejectedDrop[]) {
    if (rejected.length === 0) return;
    const lines = rejected.map(
      ({ path, reason }) => `${fileName(path)}: ${t(`fileDrop.reasons.${reason}`)}`,
    );
    notifyWarning(t("fileDrop.rejectedTitle"), lines.join("\n"));
  }

  async function addDroppedFiles(paths: string[]) {
    if (!profile) return;
    try {
      const { action, rejected } = planDrop(await commands.resolveDroppedPaths(paths));
      if (action.kind === "openForm") setFormTarget(action.target);
      if (action.kind === "append") {
        saveItems([...profile.items, ...action.items]);
        notifySuccess(t("fileDrop.added", { count: action.items.length }));
      }
      reportRejected(rejected);
    } catch (error) {
      notifyError(error);
    }
  }

  const isDragging = useFileDrop({
    enabled: profile !== undefined && !pickerOpen && formTarget === null,
    onDrop: (paths) => void addDroppedFiles(paths),
  });

  if (!profile) return null;

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

  const openPicker = () => {
    setPickerOpen(true);
  };
  const openUrlForm = () => {
    setFormTarget({ mode: "create", type: "url" });
  };
  const profileExePaths = profile.items.flatMap((item) =>
    item.type === "app" ? [item.exePath] : [],
  );

  return (
    <div className="relative flex min-h-0 flex-1 flex-col">
      <TopBar profile={profile} />

      <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
        <ElevationBanner />
        {profile.items.length === 0 ? (
          <EmptyItems isElevated={isElevated} onAddApp={openPicker} onAddUrl={openUrlForm} />
        ) : (
          <>
            <div className="flex justify-end gap-2">
              <Button size="sm" onClick={openPicker}>
                <AppWindow />
                {t("items.addApp")}
              </Button>
              <Button size="sm" variant="outline" onClick={openUrlForm}>
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

      <DropOverlay visible={isDragging} profileName={profile.name} />

      <AppPickerDialog
        open={pickerOpen}
        profileExePaths={profileExePaths}
        onPick={(initial) => {
          setFormTarget({ mode: "create", type: "app", initial });
        }}
        onClose={() => {
          setPickerOpen(false);
        }}
      />

      <ItemFormDialog
        target={formTarget}
        profileItems={profile.items}
        simConnectSupported={supportsSimConnect(profile.trigger)}
        onSave={saveItem}
        onClose={() => {
          setFormTarget(null);
        }}
      />
    </div>
  );
}
