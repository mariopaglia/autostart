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

export function ElevationBanner() {
  const { t } = useTranslation();
  const isElevated = useIsElevated();
  const profiles = useProfilesStore((state) => state.profiles);
  const [relaunching, setRelaunching] = useState(false);

  if (isElevated === null) return null;
  const warnedProfile = profiles.find(
    (profile) => profile.enabled && needsElevationWarning(profile, isElevated),
  );
  if (!warnedProfile) return null;

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
        <span>{t("elevation.description", { profile: warnedProfile.name })}</span>
        <Button size="sm" variant="outline" disabled={relaunching} onClick={() => void relaunch()}>
          {relaunching && <Spinner />}
          {t("elevation.relaunch")}
        </Button>
      </AlertDescription>
    </Alert>
  );
}
