import { useState } from "react";
import { ExternalLink } from "lucide-react";
import { useTranslation } from "react-i18next";
import { ChangelogDialog } from "@/components/settings/ChangelogDialog";
import { SettingsSection } from "@/components/settings/SettingsSection";
import { Button } from "@/components/ui/button";
import { notifyError } from "@/lib/notify";
import { system } from "@/lib/tauri";

const REPOSITORY_URL = "https://github.com/mariopaglia/autostart";
const LICENSE_URL = `${REPOSITORY_URL}/blob/main/LICENSE`;

function openExternal(url: string) {
  return () => {
    void system.openUrl(url).catch(notifyError);
  };
}

export function AboutSection({ version }: { version: string }) {
  const { t } = useTranslation();
  const [changelogOpen, setChangelogOpen] = useState(false);

  return (
    <SettingsSection title={t("settings.sections.about")}>
      <dl className="grid grid-cols-[auto_1fr] gap-x-6 gap-y-2 py-3 text-sm">
        <dt className="text-muted-foreground">{t("settings.version")}</dt>
        <dd className="flex items-baseline gap-2">
          <span className="font-mono tabular-nums">{version}</span>
          <Button
            variant="link"
            className="h-auto p-0"
            onClick={() => {
              setChangelogOpen(true);
            }}
          >
            {t("settings.viewChangelog")}
          </Button>
        </dd>
        <dt className="text-muted-foreground">{t("settings.license")}</dt>
        <dd>
          <Button variant="link" className="h-auto p-0" onClick={openExternal(LICENSE_URL)}>
            {t("settings.licenseName")}
            <ExternalLink />
          </Button>
        </dd>
        <dt className="text-muted-foreground">{t("settings.repository")}</dt>
        <dd>
          <Button variant="link" className="h-auto p-0" onClick={openExternal(REPOSITORY_URL)}>
            github.com/mariopaglia/autostart
            <ExternalLink />
          </Button>
        </dd>
      </dl>
      <p className="pb-3 text-xs text-muted-foreground">
        {t("settings.copyright")} {t("settings.licenseNotice")}
      </p>
      <ChangelogDialog open={changelogOpen} onOpenChange={setChangelogOpen} />
    </SettingsSection>
  );
}
