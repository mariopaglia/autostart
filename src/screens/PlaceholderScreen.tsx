import { useTranslation } from "react-i18next";
import type { Screen } from "@/stores/ui-store";

// Logs and Settings are built in phase 3; this keeps navigation working meanwhile.
export function PlaceholderScreen({ screen }: { screen: Exclude<Screen, "main"> }) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-1 flex-col gap-2 p-6">
      <h1 className="text-xl font-semibold tracking-tight">{t(`nav.${screen}`)}</h1>
      <p className="text-sm text-muted-foreground">{t("common.comingSoon")}</p>
    </div>
  );
}
