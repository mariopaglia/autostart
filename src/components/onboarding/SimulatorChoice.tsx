import { useState } from "react";
import { Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Trigger } from "@/bindings/Trigger";
import { Input } from "@/components/ui/input";
import { TRIGGER_PRESETS, triggerFromProcessName } from "@/lib/trigger-presets";
import { cn } from "@/lib/utils";
import { processNameSchema } from "@/schemas/profile";

interface SimulatorChoiceProps {
  value: Trigger | null;
  onChange: (trigger: Trigger | null) => void;
}

function customTrigger(processName: string): Trigger | null {
  if (processName.trim() === "") return null;
  const trigger = triggerFromProcessName(processName);
  return processNameSchema.safeParse(trigger.processName).success ? trigger : null;
}

function isPreset(trigger: Trigger | null): boolean {
  return TRIGGER_PRESETS.some((preset) => preset.processName === trigger?.processName);
}

export function SimulatorChoice({ value, onChange }: SimulatorChoiceProps) {
  const { t } = useTranslation();
  const [customSelected, setCustomSelected] = useState(value !== null && !isPreset(value));
  const [customName, setCustomName] = useState(customSelected ? (value?.processName ?? "") : "");
  const showCustomError = customName.trim() !== "" && customTrigger(customName) === null;

  function chooseCustom(processName: string) {
    setCustomSelected(true);
    setCustomName(processName);
    onChange(customTrigger(processName));
  }

  const optionClass = (selected: boolean) =>
    cn(
      "flex items-center justify-between gap-3 rounded-xl border px-4 py-3 text-left text-sm outline-none transition-colors motion-reduce:transition-none",
      "hover:border-primary/50 focus-visible:ring-2 focus-visible:ring-ring",
      selected && "border-primary bg-primary/5",
    );

  return (
    <div role="radiogroup" aria-label={t("onboarding.simulator.title")} className="grid gap-2">
      {TRIGGER_PRESETS.map((preset) => {
        const selected = !customSelected && value?.processName === preset.processName;
        return (
          <button
            key={preset.processName}
            type="button"
            role="radio"
            aria-checked={selected}
            className={optionClass(selected)}
            onClick={() => {
              setCustomSelected(false);
              onChange(preset);
            }}
          >
            <span className="flex flex-col">
              <span className="font-medium">{preset.label}</span>
              <span className="text-xs text-muted-foreground">{preset.processName}</span>
            </span>
            {selected && <Check className="size-4 text-primary" />}
          </button>
        );
      })}

      <div className={cn(optionClass(customSelected), "flex-col items-stretch")}>
        <button
          type="button"
          role="radio"
          aria-checked={customSelected}
          className="text-left font-medium outline-none focus-visible:underline"
          onClick={() => {
            chooseCustom(customName);
          }}
        >
          {t("onboarding.simulator.custom")}
        </button>
        {customSelected && (
          <div className="flex flex-col gap-1">
            <Input
              autoFocus
              value={customName}
              placeholder="Simulator.exe"
              aria-label={t("trigger.label")}
              aria-invalid={showCustomError}
              onChange={(event) => {
                chooseCustom(event.target.value);
              }}
            />
            {showCustomError && (
              <p role="alert" className="text-xs text-destructive">
                {t("validation.exe")}
              </p>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
