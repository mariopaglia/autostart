import { z } from "zod";
import type { AppItem } from "@/bindings/AppItem";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { OnClose } from "@/bindings/OnClose";
import type { ProcessNameMode } from "@/bindings/ProcessNameMode";
import type { Profile } from "@/bindings/Profile";
import type { Trigger } from "@/bindings/Trigger";
import type { UrlItem } from "@/bindings/UrlItem";

export const DEFAULT_DELAY_MS = 800;
export const MAX_DELAY_MS = 60_000;
export const MAX_TRIGGERS = 5;

export const nameSchema = z
  .string()
  .trim()
  .min(1, { error: "validation.name" })
  .max(80, { error: "validation.name" });
export const processNameSchema = z
  .string()
  .trim()
  .regex(/^[^\\/:*?"<>|]+\.exe$/i, { error: "validation.exe" });
const delaySchema = z
  .number({ error: "validation.delay" })
  .int({ error: "validation.delay" })
  .min(0, { error: "validation.delay" })
  .max(MAX_DELAY_MS, { error: "validation.delay" })
  .default(DEFAULT_DELAY_MS);
const optionalTextSchema = z
  .string()
  .trim()
  .optional()
  .transform((value) => (value === "" ? undefined : value));

const STEAM_LAUNCH_LINK = /^steam:\/\/(rungameid|run)\/\d+$/;

export function isSteamLaunchLink(value: string): boolean {
  return STEAM_LAUNCH_LINK.test(value);
}

function isWebUrl(value: string): boolean {
  return z.url({ protocol: /^https?$/ }).safeParse(value).success;
}

/** Profiles are shared between pilots, so only schemes that cannot run arbitrary handlers pass. */
export const itemUrlSchema = z
  .string()
  .trim()
  .refine((value) => isWebUrl(value) || isSteamLaunchLink(value), { error: "validation.url" });

const EXECUTABLE_PATH = /^([a-z]:\\|\\\\|\/).*\.exe$/i;
const STORE_APP = /^shell:AppsFolder\\[^\s\\/:*?"<>|]*![^\s\\/:*?"<>|]*$/;

/** How AutoStart starts a simulator: an absolute `.exe`, a Steam link or a Store app. */
export const launchTargetSchema = z
  .string()
  .trim()
  .refine(
    (value) => EXECUTABLE_PATH.test(value) || isSteamLaunchLink(value) || STORE_APP.test(value),
    { error: "validation.launchTarget" },
  );

export const processNameModeSchema = z.enum([
  "auto",
  "manual",
]) satisfies z.ZodType<ProcessNameMode>;

export const onCloseSchema = z.enum(["graceful", "force", "keep"]) satisfies z.ZodType<OnClose>;

export const triggerSchema = z.object({
  processName: processNameSchema,
  label: nameSchema,
  launchTarget: launchTargetSchema.optional(),
}) satisfies z.ZodType<Trigger>;

export const triggersSchema = z
  .array(triggerSchema)
  .min(1, { error: "validation.triggers" })
  .max(MAX_TRIGGERS, { error: "validation.triggers" })
  .refine(
    (triggers) =>
      new Set(triggers.map((trigger) => trigger.processName.toLowerCase())).size ===
      triggers.length,
    { error: "validation.triggerDuplicate" },
  );

export const appItemSchema = z.object({
  id: z.uuid(),
  name: nameSchema,
  exePath: z.string().trim().min(1, { error: "validation.required" }),
  args: optionalTextSchema,
  workingDir: optionalTextSchema,
  processName: processNameSchema,
  processNameMode: processNameModeSchema.default("auto"),
  iconBase64: z.string().optional(),
  delayMs: delaySchema,
  runAsAdmin: z.boolean().default(false),
  startMinimized: z.boolean().default(false),
  waitForSimConnect: z.boolean().default(false),
  restartOnCrash: z.boolean().default(false),
  launchBeforeSimulator: z.boolean().default(false),
  onlyForTriggers: z.array(processNameSchema).default([]),
  onClose: onCloseSchema.default("graceful"),
  enabled: z.boolean().default(true),
}) satisfies z.ZodType<AppItem>;

export const urlItemSchema = z.object({
  id: z.uuid(),
  name: nameSchema,
  url: itemUrlSchema,
  delayMs: delaySchema,
  onlyForTriggers: z.array(processNameSchema).default([]),
  enabled: z.boolean().default(true),
}) satisfies z.ZodType<UrlItem>;

export const launchItemSchema = z.discriminatedUnion("type", [
  appItemSchema.extend({ type: z.literal("app") }),
  urlItemSchema.extend({ type: z.literal("url") }),
]) satisfies z.ZodType<LaunchItem>;

/** Profiles exported before v0.3.0 have a single `trigger` instead of a `triggers` list. */
function withTriggersList(raw: unknown): unknown {
  if (typeof raw !== "object" || raw === null || !("trigger" in raw) || "triggers" in raw) {
    return raw;
  }
  const { trigger, ...rest } = raw;
  return { ...rest, triggers: [trigger] };
}

function checkItems(
  profile: { triggers: Trigger[]; items: LaunchItem[] },
  context: z.RefinementCtx,
) {
  const triggerNames = new Set(
    profile.triggers.map((trigger) => trigger.processName.toLowerCase()),
  );
  profile.items.forEach((item, index) => {
    const onlyFor = item.onlyForTriggers.map((processName) => processName.toLowerCase());
    const unknownOrRepeated =
      onlyFor.some((processName) => !triggerNames.has(processName)) ||
      new Set(onlyFor).size !== onlyFor.length;
    if (unknownOrRepeated) {
      context.addIssue({
        code: "custom",
        path: ["items", index, "onlyForTriggers"],
        message: "validation.onlyForTriggers",
      });
    }
    if (item.type === "app" && item.launchBeforeSimulator && item.waitForSimConnect) {
      context.addIssue({
        code: "custom",
        path: ["items", index, "launchBeforeSimulator"],
        message: "validation.launchBeforeSimulator",
      });
    }
  });
}

export const profileSchema = z.preprocess(
  withTriggersList,
  z
    .object({
      id: z.uuid(),
      name: nameSchema,
      triggers: triggersSchema,
      items: z.array(launchItemSchema).default([]),
      enabled: z.boolean().default(true),
    })
    .superRefine(checkItems),
) satisfies z.ZodType<Profile>;
