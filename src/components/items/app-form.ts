import type { z } from "zod";
import { appItemSchema, DEFAULT_DELAY_MS } from "@/schemas/profile";

export const appFormSchema = appItemSchema.omit({ id: true });
export type AppFormInput = z.input<typeof appFormSchema>;
export type AppFormOutput = z.output<typeof appFormSchema>;

export const EMPTY_APP_FORM: AppFormInput = {
  name: "",
  exePath: "",
  args: "",
  workingDir: "",
  processName: "",
  processNameMode: "auto",
  delayMs: DEFAULT_DELAY_MS,
  runAsAdmin: false,
  startMinimized: false,
  waitForSimConnect: false,
  restartOnCrash: false,
  launchBeforeSimulator: false,
  onlyForTriggers: [],
  onClose: "graceful",
  enabled: true,
};

export function executableName(exePath: string): string {
  return exePath.split(/[\\/]/).pop()?.trim() ?? "";
}
