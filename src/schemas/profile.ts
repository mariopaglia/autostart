import { z } from "zod";
import type { AppItem } from "@/bindings/AppItem";
import type { LaunchItem } from "@/bindings/LaunchItem";
import type { OnClose } from "@/bindings/OnClose";
import type { Profile } from "@/bindings/Profile";
import type { Trigger } from "@/bindings/Trigger";
import type { UrlItem } from "@/bindings/UrlItem";

export const DEFAULT_DELAY_MS = 800;
export const MAX_DELAY_MS = 60_000;

const nameSchema = z.string().trim().min(1).max(80);
const processNameSchema = z
  .string()
  .trim()
  .regex(/^[^\\/:*?"<>|]+\.exe$/i);
const delaySchema = z.number().int().min(0).max(MAX_DELAY_MS).default(DEFAULT_DELAY_MS);
const optionalTextSchema = z
  .string()
  .trim()
  .optional()
  .transform((value) => (value === "" ? undefined : value));

export const onCloseSchema = z.enum(["graceful", "force", "keep"]) satisfies z.ZodType<OnClose>;

export const triggerSchema = z.object({
  processName: processNameSchema,
  label: nameSchema,
}) satisfies z.ZodType<Trigger>;

export const appItemSchema = z.object({
  id: z.uuid(),
  name: nameSchema,
  exePath: z.string().trim().min(1),
  args: optionalTextSchema,
  workingDir: optionalTextSchema,
  processName: processNameSchema,
  iconBase64: z.string().optional(),
  delayMs: delaySchema,
  runAsAdmin: z.boolean().default(false),
  onClose: onCloseSchema.default("graceful"),
  enabled: z.boolean().default(true),
}) satisfies z.ZodType<AppItem>;

export const urlItemSchema = z.object({
  id: z.uuid(),
  name: nameSchema,
  url: z.url({ protocol: /^https?$/ }),
  delayMs: delaySchema,
  enabled: z.boolean().default(true),
}) satisfies z.ZodType<UrlItem>;

export const launchItemSchema = z.discriminatedUnion("type", [
  appItemSchema.extend({ type: z.literal("app") }),
  urlItemSchema.extend({ type: z.literal("url") }),
]) satisfies z.ZodType<LaunchItem>;

export const profileSchema = z.object({
  id: z.uuid(),
  name: nameSchema,
  trigger: triggerSchema,
  items: z.array(launchItemSchema).default([]),
  enabled: z.boolean().default(true),
}) satisfies z.ZodType<Profile>;
