import { useState } from "react";
import { ShieldAlert } from "lucide-react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Button } from "@/components/ui/button";
import { Spinner } from "@/components/ui/spinner";
import { useIsElevated } from "@/hooks/use-is-elevated";
import { needsElevationWarning } from "@/lib/elevation";
import { notifyError } from "@/lib/notify";
import { commands, errorKindOf } from "@/lib/tauri";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";

export function ElevationBanner() {
  const { t } = useTranslation();
  const isElevated = useIsElevated();
  const activeId = useSettingsStore((state) => state.settings?.activeProfileId);
  const activeProfile = useProfilesStore((state) =>
    state.profiles.find((profile) => profile.id === activeId),
  );
  const [relaunching, setRelaunching] = useState(false);

  if (isElevated === null || !needsElevationWarning(activeProfile, isElevated)) return null;

  async function relaunch() {
    setRelaunching(true);
    try {
      await commands.relaunchAsAdmin();
    } catch (error) {
      if (errorKindOf(error) === "elevationDenied") {
        toast.error(t("elevation.relaunchCancelled"));
      } else {
        notifyError(error);
      }
    } finally {
      setRelaunching(false);
    }
  }

  return (
    <Alert className="border-amber-500/40 bg-amber-500/10">
      <ShieldAlert className="text-amber-600 dark:text-amber-400" />
      <AlertTitle>{t("elevation.title")}</AlertTitle>
      <AlertDescription className="flex flex-wrap items-center justify-between gap-3">
        <span>{t("elevation.description", { profile: activeProfile?.name ?? "" })}</span>
        <Button size="sm" variant="outline" disabled={relaunching} onClick={() => void relaunch()}>
          {relaunching && <Spinner />}
          {t("elevation.relaunch")}
        </Button>
      </AlertDescription>
    </Alert>
  );
}
