import type { z } from "zod";
import { appItemSchema } from "@/schemas/profile";

export const appFormSchema = appItemSchema.omit({ id: true });
export type AppFormInput = z.input<typeof appFormSchema>;
export type AppFormOutput = z.output<typeof appFormSchema>;

export function executableName(exePath: string): string {
  return exePath.split(/[\\/]/).pop()?.trim() ?? "";
}
