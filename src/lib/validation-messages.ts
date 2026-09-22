import type { TFunction } from "i18next";
import type { FieldError } from "react-hook-form";

const VALIDATION_KEYS = ["required", "name", "exe", "url", "delay", "invalid"] as const;
type ValidationKey = (typeof VALIDATION_KEYS)[number];

function isValidationKey(value: string): value is ValidationKey {
  return (VALIDATION_KEYS as readonly string[]).includes(value);
}

/** Schemas use `validation.<key>` as their error message; anything else falls back to "invalid". */
export function translateValidationMessage(t: TFunction, message: string | undefined): string {
  const key = message?.replace(/^validation\./, "") ?? "";
  return t(`validation.${isValidationKey(key) ? key : "invalid"}`);
}

export function translateFieldError(t: TFunction, error: FieldError | undefined) {
  return error ? [{ message: translateValidationMessage(t, error.message) }] : [];
}
