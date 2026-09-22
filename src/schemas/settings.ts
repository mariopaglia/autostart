import { z } from "zod";
import type { Language } from "@/bindings/Language";
import type { Settings } from "@/bindings/Settings";
import type { Theme } from "@/bindings/Theme";

export const MIN_GRACEFUL_TIMEOUT_MS = 1_000;
export const MAX_GRACEFUL_TIMEOUT_MS = 60_000;

export const themeSchema = z.enum(["system", "light", "dark"]) satisfies z.ZodType<Theme>;
export const languageSchema = z.enum(["pt-BR", "en"]) satisfies z.ZodType<Language>;

export const settingsSchema = z.object({
  activeProfileId: z.uuid().nullable().default(null),
  startWithWindows: z.boolean().default(false),
  startMinimized: z.boolean().default(true),
  gracefulTimeoutMs: z
    .number()
    .int()
    .min(MIN_GRACEFUL_TIMEOUT_MS)
    .max(MAX_GRACEFUL_TIMEOUT_MS)
    .default(5_000),
  closeOnlyIfLaunchedByApp: z.boolean().default(true),
  theme: themeSchema.default("dark"),
  language: languageSchema.default("pt-BR"),
  onboardingCompleted: z.boolean().default(false),
  checkUpdatesOnStartup: z.boolean().default(true),
}) satisfies z.ZodType<Settings>;
