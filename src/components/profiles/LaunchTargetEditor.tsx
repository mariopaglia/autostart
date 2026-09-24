import { useState } from "react";
import { FolderOpen, Rocket, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Trigger } from "@/bindings/Trigger";
import { pickExecutablePath } from "@/components/items/pick-executable";
import { Button } from "@/components/ui/button";
import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { notifyError } from "@/lib/notify";
import { launchTargetPresets } from "@/lib/trigger-presets";
import { translateValidationMessage } from "@/lib/validation-messages";
import { launchTargetSchema } from "@/schemas/profile";

interface LaunchTargetEditorProps {
  trigger: Trigger;
  onChange: (launchTarget: string | undefined) => void;
}

export function LaunchTargetEditor({ trigger, onChange }: LaunchTargetEditorProps) {
  const { t } = useTranslation();
  const [typed, setTyped] = useState(trigger.launchTarget ?? "");
  const [error, setError] = useState<string | undefined>();
  const presets = launchTargetPresets(trigger);

  function save(target: string) {
    const result = launchTargetSchema.safeParse(target);
    if (!result.success) {
      setError(translateValidationMessage(t, result.error.issues[0]?.message));
      return;
    }
    setError(undefined);
    setTyped(result.data);
    onChange(result.data);
  }

  async function chooseExecutable() {
    try {
      const path = await pickExecutablePath(t("itemForm.exeFilter"));
      if (path) save(path);
    } catch (pickError) {
      notifyError(pickError);
    }
  }

  return (
    <div className="flex flex-col gap-3">
      <div className="flex flex-col gap-1">
        <h3 className="flex items-center gap-2 text-sm font-medium">
          <Rocket className="size-4" />
          {t("trigger.launchTarget.title", { label: trigger.label })}
        </h3>
        <p className="text-xs text-muted-foreground">{t("trigger.launchTarget.description")}</p>
      </div>

      <div className="flex flex-wrap gap-2">
        {presets.map((preset) => (
          <Button
            key={preset.source}
            type="button"
            size="sm"
            variant={trigger.launchTarget === preset.target ? "secondary" : "outline"}
            aria-pressed={trigger.launchTarget === preset.target}
            onClick={() => {
              save(preset.target);
            }}
          >
            {t(`trigger.launchTarget.${preset.source}`)}
          </Button>
        ))}
        <Button type="button" size="sm" variant="outline" onClick={() => void chooseExecutable()}>
          <FolderOpen />
          {t("trigger.launchTarget.chooseExe")}
        </Button>
      </div>

      <form
        className="flex flex-col gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          save(typed);
        }}
      >
        <Field data-invalid={error !== undefined}>
          <FieldLabel htmlFor="launchTarget">{t("trigger.launchTarget.field")}</FieldLabel>
          <div className="flex gap-2">
            <Input
              id="launchTarget"
              value={typed}
              placeholder="steam://rungameid/…"
              aria-invalid={error !== undefined}
              onChange={(event) => {
                setTyped(event.target.value);
              }}
            />
            <Button type="submit" size="sm">
              {t("common.save")}
            </Button>
          </div>
          {error ? (
            <FieldError>{error}</FieldError>
          ) : (
            <FieldDescription>{t("trigger.launchTarget.fieldHint")}</FieldDescription>
          )}
        </Field>
      </form>

      {trigger.launchTarget && (
        <Button
          type="button"
          size="sm"
          variant="ghost"
          className="w-fit text-destructive"
          onClick={() => {
            setTyped("");
            onChange(undefined);
          }}
        >
          <Trash2 />
          {t("trigger.launchTarget.remove")}
        </Button>
      )}
    </div>
  );
}
