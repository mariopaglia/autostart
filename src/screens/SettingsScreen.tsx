import { useEffect, useState } from "react";
import { FolderOpen, RefreshCw } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Language } from "@/bindings/Language";
import type { Settings } from "@/bindings/Settings";
import type { Theme } from "@/bindings/Theme";
import { AboutSection } from "@/components/settings/AboutSection";
import { CloseDelayField } from "@/components/settings/CloseDelayField";
import { GracefulTimeoutField } from "@/components/settings/GracefulTimeoutField";
import { SettingRow } from "@/components/settings/SettingRow";
import { SettingsSection } from "@/components/settings/SettingsSection";
import { Button } from "@/components/ui/button";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Separator } from "@/components/ui/separator";
import { Spinner } from "@/components/ui/spinner";
import { Switch } from "@/components/ui/switch";
import { notifyError } from "@/lib/notify";
import { commands, system } from "@/lib/tauri";
import { languageSchema, themeSchema } from "@/schemas/settings";
import { useSettingsStore } from "@/stores/settings-store";
import { useUpdatesStore } from "@/stores/updates-store";

const THEMES: readonly Theme[] = themeSchema.options;
const LANGUAGES: readonly Language[] = languageSchema.options;
const LANGUAGE_NAMES: Record<Language, string> = { "pt-BR": "Português (Brasil)", en: "English" };

function useAppVersion(): string {
  const [version, setVersion] = useState("");
  useEffect(() => {
    void system.getAppVersion().then(setVersion);
  }, []);
  return version;
}

export function SettingsScreen() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const saveSettings = useSettingsStore((state) => state.save);
  const version = useAppVersion();
  const updateStatus = useUpdatesStore((state) => state.status);
  const checkUpdates = useUpdatesStore((state) => state.check);

  if (!settings) return null;

  const save = (patch: Partial<Settings>) => void saveSettings(patch);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="border-b px-6 py-4">
        <h1 className="text-xl font-semibold tracking-tight">{t("settings.title")}</h1>
      </header>

      <div className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
        <SettingsSection title={t("settings.sections.startup")}>
          <SettingRow
            id="start-with-windows"
            label={t("settings.startWithWindows")}
            description={t("settings.startWithWindowsHint")}
          >
            <Switch
              id="start-with-windows"
              checked={settings.startWithWindows}
              onCheckedChange={(startWithWindows) => {
                save({ startWithWindows });
              }}
            />
          </SettingRow>
          <Separator />
          <SettingRow
            id="start-minimized"
            label={t("settings.startMinimized")}
            description={t("settings.startMinimizedHint")}
          >
            <Switch
              id="start-minimized"
              checked={settings.startMinimized}
              onCheckedChange={(startMinimized) => {
                save({ startMinimized });
              }}
            />
          </SettingRow>
        </SettingsSection>

        <SettingsSection title={t("settings.sections.closing")}>
          <SettingRow
            id="close-delay"
            label={t("settings.closeDelay")}
            description={t("settings.closeDelayHint")}
          >
            <CloseDelayField
              id="close-delay"
              valueMs={settings.closeDelayMs}
              onSave={(closeDelayMs) => {
                save({ closeDelayMs });
              }}
            />
          </SettingRow>
          <Separator />
          <SettingRow
            id="graceful-timeout"
            label={t("settings.gracefulTimeout")}
            description={t("settings.gracefulTimeoutHint")}
          >
            <GracefulTimeoutField
              id="graceful-timeout"
              value={settings.gracefulTimeoutMs}
              onSave={(gracefulTimeoutMs) => {
                save({ gracefulTimeoutMs });
              }}
            />
          </SettingRow>
          <Separator />
          <SettingRow
            id="close-only-launched"
            label={t("settings.closeOnlyIfLaunchedByApp")}
            description={t("settings.closeOnlyIfLaunchedByAppHint")}
          >
            <Switch
              id="close-only-launched"
              checked={settings.closeOnlyIfLaunchedByApp}
              onCheckedChange={(closeOnlyIfLaunchedByApp) => {
                save({ closeOnlyIfLaunchedByApp });
              }}
            />
          </SettingRow>
        </SettingsSection>

        <SettingsSection title={t("settings.sections.notifications")}>
          <SettingRow
            id="show-notifications"
            label={t("settings.showNotifications")}
            description={t("settings.showNotificationsHint")}
          >
            <Switch
              id="show-notifications"
              checked={settings.showNotifications}
              onCheckedChange={(showNotifications) => {
                save({ showNotifications });
              }}
            />
          </SettingRow>
        </SettingsSection>

        <SettingsSection title={t("settings.sections.appearance")}>
          <SettingRow id="theme" label={t("settings.theme")}>
            <Select
              value={settings.theme}
              onValueChange={(value) => {
                save({ theme: themeSchema.parse(value) });
              }}
            >
              <SelectTrigger id="theme" className="w-44">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {THEMES.map((theme) => (
                  <SelectItem key={theme} value={theme}>
                    {t(`settings.themes.${theme}`)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingRow>
          <Separator />
          <SettingRow id="language" label={t("settings.language")}>
            <Select
              value={settings.language}
              onValueChange={(value) => {
                save({ language: languageSchema.parse(value) });
              }}
            >
              <SelectTrigger id="language" className="w-44">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {LANGUAGES.map((language) => (
                  <SelectItem key={language} value={language}>
                    {LANGUAGE_NAMES[language]}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </SettingRow>
        </SettingsSection>

        <SettingsSection title={t("settings.sections.maintenance")}>
          <SettingRow
            id="check-updates"
            label={t("settings.checkUpdatesOnStartup")}
            description={t("settings.checkUpdatesOnStartupHint")}
          >
            <Switch
              id="check-updates"
              checked={settings.checkUpdatesOnStartup}
              onCheckedChange={(checkUpdatesOnStartup) => {
                save({ checkUpdatesOnStartup });
              }}
            />
          </SettingRow>
          <Separator />
          <SettingRow id="check-updates-now" label={t("updates.checkNow")}>
            <Button
              id="check-updates-now"
              variant="outline"
              size="sm"
              disabled={updateStatus !== "idle"}
              onClick={() => void checkUpdates({ silent: false })}
            >
              {updateStatus === "checking" ? <Spinner /> : <RefreshCw />}
              {t("updates.checkNow")}
            </Button>
          </SettingRow>
          <Separator />
          <SettingRow
            id="open-logs"
            label={t("settings.logs")}
            description={t("settings.logsHint")}
          >
            <Button
              id="open-logs"
              variant="outline"
              size="sm"
              onClick={() => void commands.openLogDir().catch(notifyError)}
            >
              <FolderOpen />
              {t("logs.openFolder")}
            </Button>
          </SettingRow>
        </SettingsSection>

        <AboutSection version={version} />
      </div>
    </div>
  );
}
