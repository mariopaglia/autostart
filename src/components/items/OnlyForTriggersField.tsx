import { Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Trigger } from "@/bindings/Trigger";
import { Button } from "@/components/ui/button";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";

interface OnlyForTriggersFieldProps {
  triggers: readonly Trigger[];
  value: readonly string[];
  onChange: (value: string[]) => void;
}

function includes(value: readonly string[], processName: string): boolean {
  return value.some((selected) => selected.toLowerCase() === processName.toLowerCase());
}

/** Only meaningful when the profile has more than one simulator. */
export function OnlyForTriggersField({ triggers, value, onChange }: OnlyForTriggersFieldProps) {
  const { t } = useTranslation();

  function toggle(processName: string) {
    onChange(
      includes(value, processName)
        ? value.filter((selected) => selected.toLowerCase() !== processName.toLowerCase())
        : [...value, processName],
    );
  }

  return (
    <Field role="group" aria-labelledby="onlyForTriggers-label">
      <FieldLabel id="onlyForTriggers-label">{t("itemForm.onlyForTriggers")}</FieldLabel>
      <div className="flex flex-wrap gap-2">
        {triggers.map((trigger) => {
          const selected = includes(value, trigger.processName);
          return (
            <Button
              key={trigger.processName}
              type="button"
              size="sm"
              variant={selected ? "secondary" : "outline"}
              aria-pressed={selected}
              onClick={() => {
                toggle(trigger.processName);
              }}
            >
              {selected && <Check />}
              {trigger.label}
            </Button>
          );
        })}
      </div>
      <FieldDescription>{t("itemForm.onlyForTriggersHint")}</FieldDescription>
    </Field>
  );
}
