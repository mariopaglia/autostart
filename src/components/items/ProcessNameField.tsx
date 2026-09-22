import { ScanSearch, Sparkles } from "lucide-react";
import { Controller, useController, useWatch, type Control } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { translateFieldError } from "@/lib/validation-messages";
import { executableName, type AppFormInput, type AppFormOutput } from "./app-form";

interface ProcessNameFieldProps {
  control: Control<AppFormInput, unknown, AppFormOutput>;
}

export function ProcessNameField({ control }: ProcessNameFieldProps) {
  const { t } = useTranslation();
  const {
    field: { value: mode, onChange: setMode },
  } = useController({ control, name: "processNameMode" });
  const exePath = useWatch({ control, name: "exePath" });
  const isAuto = mode !== "manual";

  return (
    <Controller
      control={control}
      name="processName"
      render={({ field, fieldState }) => (
        <Field data-invalid={fieldState.invalid}>
          <div className="flex items-center justify-between gap-2">
            <FieldLabel htmlFor="processName">{t("itemForm.processName")}</FieldLabel>
            {isAuto ? (
              <Badge variant="secondary" className="gap-1">
                <Sparkles className="size-3" />
                {t("itemForm.processNameAuto")}
              </Badge>
            ) : (
              <Button
                type="button"
                variant="ghost"
                size="xs"
                onClick={() => {
                  setMode("auto");
                  const fromExecutable = executableName(exePath);
                  if (fromExecutable) field.onChange(fromExecutable);
                }}
              >
                <ScanSearch />
                {t("itemForm.detectAutomatically")}
              </Button>
            )}
          </div>
          <Input
            id="processName"
            name={field.name}
            ref={field.ref}
            value={field.value}
            onBlur={field.onBlur}
            onChange={(event) => {
              field.onChange(event.target.value);
              setMode("manual");
            }}
            aria-invalid={fieldState.invalid}
          />
          <FieldDescription>
            {isAuto ? t("itemForm.processNameAutoHint") : t("itemForm.processNameManualHint")}
          </FieldDescription>
          <FieldError errors={translateFieldError(t, fieldState.error)} />
        </Field>
      )}
    />
  );
}
