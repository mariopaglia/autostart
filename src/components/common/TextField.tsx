import type { ComponentProps } from "react";
import { Controller, type Control, type FieldPath, type FieldValues } from "react-hook-form";
import { useTranslation } from "react-i18next";
import { Field, FieldDescription, FieldError, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { translateFieldError } from "@/lib/validation-messages";

interface TextFieldProps<TValues extends FieldValues> {
  control: Control<TValues>;
  name: FieldPath<TValues>;
  label: string;
  description?: string;
  inputProps?: Omit<ComponentProps<typeof Input>, "id" | "name" | "value" | "onChange">;
  numeric?: boolean;
}

export function TextField<TValues extends FieldValues>({
  control,
  name,
  label,
  description,
  inputProps,
  numeric = false,
}: TextFieldProps<TValues>) {
  const { t } = useTranslation();

  return (
    <Controller
      control={control}
      name={name}
      render={({ field, fieldState }) => (
        <Field data-invalid={fieldState.invalid}>
          <FieldLabel htmlFor={name}>{label}</FieldLabel>
          <Input
            {...inputProps}
            id={name}
            name={field.name}
            ref={field.ref}
            onBlur={field.onBlur}
            type={numeric ? "number" : (inputProps?.type ?? "text")}
            value={field.value ?? ""}
            onChange={(event) => {
              field.onChange(numeric ? event.target.valueAsNumber : event.target.value);
            }}
            aria-invalid={fieldState.invalid}
          />
          {description && <FieldDescription>{description}</FieldDescription>}
          <FieldError errors={translateFieldError(t, fieldState.error)} />
        </Field>
      )}
    />
  );
}
