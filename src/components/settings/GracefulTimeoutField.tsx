import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { translateValidationMessage } from "@/lib/validation-messages";
import { gracefulTimeoutSchema } from "@/schemas/settings";

interface GracefulTimeoutFieldProps {
  id: string;
  value: number;
  onSave: (value: number) => void;
}

export function GracefulTimeoutField({ id, value, onSave }: GracefulTimeoutFieldProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(String(value));
  const parsed = gracefulTimeoutSchema.safeParse(Number(draft === "" ? Number.NaN : draft));
  const errorMessage = parsed.success ? undefined : parsed.error.issues[0]?.message;

  function commit() {
    if (parsed.success && parsed.data !== value) onSave(parsed.data);
  }

  return (
    <div className="flex flex-col items-end gap-1">
      <Input
        id={id}
        type="number"
        min={1000}
        max={60000}
        step={500}
        className="w-32 text-right"
        value={draft}
        aria-invalid={!parsed.success}
        aria-describedby={parsed.success ? undefined : `${id}-error`}
        onChange={(event) => {
          setDraft(event.target.value);
        }}
        onBlur={commit}
        onKeyDown={(event) => {
          if (event.key === "Enter") commit();
        }}
      />
      {!parsed.success && (
        <p id={`${id}-error`} role="alert" className="text-xs text-destructive">
          {translateValidationMessage(t, errorMessage)}
        </p>
      )}
    </div>
  );
}
