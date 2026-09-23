import { useState } from "react";
import type { z } from "zod";
import { useTranslation } from "react-i18next";
import { Input } from "@/components/ui/input";
import { translateValidationMessage } from "@/lib/validation-messages";

interface SecondsFieldProps {
  id: string;
  valueMs: number;
  schemaMs: z.ZodType<number>;
  minMs: number;
  maxMs: number;
  stepSeconds: number;
  onSave: (valueMs: number) => void;
}

/** Edited in seconds, stored in milliseconds like the other durations. */
export function SecondsField({
  id,
  valueMs,
  schemaMs,
  minMs,
  maxMs,
  stepSeconds,
  onSave,
}: SecondsFieldProps) {
  const { t } = useTranslation();
  const [draft, setDraft] = useState(String(valueMs / 1000));
  const parsed = schemaMs.safeParse(draft === "" ? Number.NaN : Number(draft) * 1000);
  const errorMessage = parsed.success ? undefined : parsed.error.issues[0]?.message;

  function commit() {
    if (parsed.success && parsed.data !== valueMs) onSave(parsed.data);
  }

  return (
    <div className="flex flex-col items-end gap-1">
      <Input
        id={id}
        type="number"
        min={minMs / 1000}
        max={maxMs / 1000}
        step={stepSeconds}
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
