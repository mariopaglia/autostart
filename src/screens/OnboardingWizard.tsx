import { useState } from "react";
import { AppWindow, Plane, Power, Radar } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Trigger } from "@/bindings/Trigger";
import { SimulatorChoice } from "@/components/onboarding/SimulatorChoice";
import { SettingRow } from "@/components/settings/SettingRow";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { cn } from "@/lib/utils";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";

type Step = "trigger" | "simulator" | "items" | "startup";

const STEPS: readonly { step: Step; icon: LucideIcon }[] = [
  { step: "trigger", icon: Radar },
  { step: "simulator", icon: Plane },
  { step: "items", icon: AppWindow },
  { step: "startup", icon: Power },
];

export function OnboardingWizard() {
  const { t } = useTranslation();
  const settings = useSettingsStore((state) => state.settings);
  const saveSettings = useSettingsStore((state) => state.save);
  const profiles = useProfilesStore((state) => state.profiles);
  const saveProfile = useProfilesStore((state) => state.save);
  const [stepIndex, setStepIndex] = useState(0);
  const [trigger, setTrigger] = useState<Trigger | null>(null);

  if (!settings || settings.onboardingCompleted) return null;

  const exampleProfile = profiles.find((profile) => profile.id === settings.activeProfileId);
  const current = STEPS[stepIndex] ?? STEPS[0];
  if (!current) return null;
  const isLast = stepIndex === STEPS.length - 1;
  const Icon = current.icon;

  function finish() {
    void saveSettings({ onboardingCompleted: true });
  }

  async function next() {
    if (current?.step === "simulator" && trigger && exampleProfile) {
      await saveProfile({ ...exampleProfile, name: trigger.label, trigger });
    }
    if (isLast) {
      finish();
    } else {
      setStepIndex(stepIndex + 1);
    }
  }

  return (
    <Dialog
      open
      onOpenChange={(open) => {
        if (!open) finish();
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <span className="mb-2 flex size-10 items-center justify-center rounded-xl bg-primary/10 text-primary">
            <Icon className="size-5" />
          </span>
          <DialogTitle>{t(`onboarding.${current.step}.title`)}</DialogTitle>
          <DialogDescription>{t(`onboarding.${current.step}.description`)}</DialogDescription>
        </DialogHeader>

        <div
          key={current.step}
          className="animate-in fade-in slide-in-from-right-2 duration-200 motion-reduce:animate-none"
        >
          {current.step === "simulator" && (
            <SimulatorChoice
              value={trigger ?? exampleProfile?.trigger ?? null}
              onChange={setTrigger}
            />
          )}
          {current.step === "startup" && (
            <div className="flex flex-col">
              <SettingRow id="onboarding-start-with-windows" label={t("settings.startWithWindows")}>
                <Switch
                  id="onboarding-start-with-windows"
                  checked={settings.startWithWindows}
                  onCheckedChange={(startWithWindows) => void saveSettings({ startWithWindows })}
                />
              </SettingRow>
              <Separator />
              <SettingRow id="onboarding-start-minimized" label={t("settings.startMinimized")}>
                <Switch
                  id="onboarding-start-minimized"
                  checked={settings.startMinimized}
                  onCheckedChange={(startMinimized) => void saveSettings({ startMinimized })}
                />
              </SettingRow>
            </div>
          )}
        </div>

        <DialogFooter className="items-center sm:justify-between">
          <div className="flex gap-1.5">
            <span className="sr-only">
              {t("onboarding.progress", { current: stepIndex + 1, total: STEPS.length })}
            </span>
            {STEPS.map(({ step }, index) => (
              <span
                key={step}
                aria-hidden
                className={cn(
                  "size-2 rounded-full bg-muted transition-colors motion-reduce:transition-none",
                  index <= stepIndex && "bg-primary",
                )}
              />
            ))}
          </div>
          <div className="flex gap-2">
            {!isLast && (
              <Button variant="ghost" onClick={finish}>
                {t("onboarding.skip")}
              </Button>
            )}
            {stepIndex > 0 && (
              <Button
                variant="outline"
                onClick={() => {
                  setStepIndex(stepIndex - 1);
                }}
              >
                {t("onboarding.back")}
              </Button>
            )}
            <Button onClick={() => void next()}>
              {isLast ? t("onboarding.finish") : t("onboarding.next")}
            </Button>
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
