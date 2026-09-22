import { Controller, type Control } from "react-hook-form";
import { Field, FieldDescription, FieldLabel } from "@/components/ui/field";
import { Switch } from "@/components/ui/switch";
import type { AppFormInput, AppFormOutput } from "./app-form";

type BooleanField = "runAsAdmin" | "startMinimized" | "waitForSimConnect";

interface SwitchFieldProps {
  control: Control<AppFormInput, unknown, AppFormOutput>;
  name: BooleanField;
  label: string;
  description?: string;
  disabled?: boolean;
}

export function SwitchField({ control, name, label, description, disabled }: SwitchFieldProps) {
  return (
    <Controller
      control={control}
      name={name}
      render={({ field }) => (
        <Field orientation="horizontal" data-disabled={disabled}>
          <Switch
            id={name}
            checked={field.value ?? false}
            disabled={disabled}
            onCheckedChange={field.onChange}
            aria-describedby={description ? `${name}-description` : undefined}
          />
          <div className="flex flex-col gap-0.5">
            <FieldLabel htmlFor={name}>{label}</FieldLabel>
            {description && (
              <FieldDescription id={`${name}-description`}>{description}</FieldDescription>
            )}
          </div>
        </Field>
      )}
    />
  );
}
