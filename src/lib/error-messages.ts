import type { TFunction } from "i18next";
import type { ErrorPayload } from "@/bindings/ErrorPayload";

export function translateError(t: TFunction, error: ErrorPayload): string {
  return t(`errors.${error.kind}`);
}
