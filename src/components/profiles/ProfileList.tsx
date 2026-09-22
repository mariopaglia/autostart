import { useState } from "react";
import {
  CircleCheck,
  Copy,
  Download,
  MoreHorizontal,
  Pencil,
  Plus,
  Trash2,
  Upload,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Profile } from "@/bindings/Profile";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { notifySuccess } from "@/lib/notify";
import { createProfile, duplicateProfile } from "@/lib/profile-factory";
import { cn } from "@/lib/utils";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";
import { exportProfileToFile, readProfileFromFile } from "./profile-files";
import { ProfileNameDialog } from "./ProfileNameDialog";

type NameDialogState = { mode: "create" } | { mode: "rename"; profile: Profile } | null;

export function ProfileList() {
  const { t } = useTranslation();
  const { profiles, selectedId, select, save, remove } = useProfilesStore();
  const activeId = useSettingsStore((state) => state.settings?.activeProfileId);
  const setActiveProfile = useSettingsStore((state) => state.setActiveProfile);
  const [nameDialog, setNameDialog] = useState<NameDialogState>(null);
  const [pendingDelete, setPendingDelete] = useState<Profile | null>(null);

  async function addProfile(profile: Profile) {
    if (await save(profile)) select(profile.id);
  }

  async function importProfile() {
    const profile = await readProfileFromFile();
    if (!profile) return;
    await addProfile(profile);
    notifySuccess(t("profiles.imported", { name: profile.name }));
  }

  function submitName(name: string) {
    if (nameDialog?.mode === "rename") {
      void save({ ...nameDialog.profile, name });
    } else {
      void addProfile(createProfile(name));
    }
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-2">
      <div className="flex items-center justify-between px-2">
        <span className="text-xs font-semibold tracking-wide text-muted-foreground uppercase">
          {t("profiles.title")}
        </span>
        <Button
          variant="ghost"
          size="icon-sm"
          aria-label={t("profiles.new")}
          onClick={() => {
            setNameDialog({ mode: "create" });
          }}
        >
          <Plus />
        </Button>
      </div>

      <ul className="flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto">
        {profiles.map((profile) => (
          <li key={profile.id} className="group relative">
            <button
              type="button"
              onClick={() => {
                select(profile.id);
              }}
              className={cn(
                "flex w-full flex-col items-start gap-0.5 rounded-lg px-3 py-2 pr-9 text-left text-sm transition-colors outline-none",
                "hover:bg-muted focus-visible:ring-2 focus-visible:ring-ring",
                profile.id === selectedId && "bg-muted",
              )}
            >
              <span className="flex w-full items-center gap-2">
                <span className="truncate font-medium">{profile.name}</span>
                {profile.id === activeId && (
                  <Badge className="h-4 px-1.5 text-[10px]">{t("profiles.active")}</Badge>
                )}
              </span>
              <span className="truncate text-xs text-muted-foreground">
                {profile.enabled ? profile.trigger.label : t("profiles.disabled")}
              </span>
            </button>

            <DropdownMenu>
              <DropdownMenuTrigger asChild>
                <Button
                  variant="ghost"
                  size="icon-xs"
                  className="absolute top-2 right-2 opacity-0 group-focus-within:opacity-100 group-hover:opacity-100 aria-expanded:opacity-100"
                  aria-label={t("profiles.actions")}
                >
                  <MoreHorizontal />
                </Button>
              </DropdownMenuTrigger>
              <DropdownMenuContent align="start">
                <DropdownMenuItem
                  disabled={profile.id === activeId}
                  onSelect={() => void setActiveProfile(profile.id)}
                >
                  <CircleCheck />
                  {t("profiles.setActive")}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onSelect={() => {
                    setNameDialog({ mode: "rename", profile });
                  }}
                >
                  <Pencil />
                  {t("profiles.rename")}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onSelect={() =>
                    void addProfile(duplicateProfile(profile, t("profiles.copySuffix")))
                  }
                >
                  <Copy />
                  {t("profiles.duplicate")}
                </DropdownMenuItem>
                <DropdownMenuItem onSelect={() => void exportProfileToFile(profile)}>
                  <Download />
                  {t("profiles.export")}
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  variant="destructive"
                  disabled={profiles.length === 1}
                  onSelect={() => {
                    setPendingDelete(profile);
                  }}
                >
                  <Trash2 />
                  {t("profiles.delete")}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          </li>
        ))}
      </ul>

      <Button variant="outline" size="sm" onClick={() => void importProfile()}>
        <Upload />
        {t("profiles.import")}
      </Button>

      <ProfileNameDialog
        open={nameDialog !== null}
        title={
          nameDialog?.mode === "rename" ? t("profiles.renameTitle") : t("profiles.createTitle")
        }
        initialName={nameDialog?.mode === "rename" ? nameDialog.profile.name : ""}
        onSubmit={submitName}
        onOpenChange={(open) => {
          if (!open) setNameDialog(null);
        }}
      />

      <ConfirmDialog
        open={pendingDelete !== null}
        title={t("profiles.deleteConfirm.title", { name: pendingDelete?.name ?? "" })}
        description={t("profiles.deleteConfirm.description")}
        confirmLabel={t("profiles.deleteConfirm.confirm")}
        destructive
        onConfirm={() => {
          if (pendingDelete) void remove(pendingDelete.id);
          setPendingDelete(null);
        }}
        onOpenChange={(open) => {
          if (!open) setPendingDelete(null);
        }}
      />
    </div>
  );
}
