import { z } from "zod";
import type { AppCandidate } from "@/bindings/AppCandidate";
import type { CandidateSource } from "@/bindings/CandidateSource";
import type { DroppedCandidate } from "@/bindings/DroppedCandidate";
import type { DropRejection } from "@/bindings/DropRejection";
import type { DropResolution } from "@/bindings/DropResolution";
import type { RejectedDrop } from "@/bindings/RejectedDrop";
import type { UrlCandidate } from "@/bindings/UrlCandidate";

export const candidateSourceSchema = z.enum([
  "installed",
  "open",
  "dropped",
]) satisfies z.ZodType<CandidateSource>;

export const appCandidateSchema = z.object({
  name: z.string(),
  exePath: z.string(),
  args: z.string().optional(),
  workingDir: z.string().optional(),
  processName: z.string(),
  iconBase64: z.string().optional(),
  source: candidateSourceSchema,
  suggested: z.boolean(),
}) satisfies z.ZodType<AppCandidate>;

export const appCandidatesSchema = z.array(appCandidateSchema);

export const urlCandidateSchema = z.object({
  name: z.string(),
  url: z.string(),
}) satisfies z.ZodType<UrlCandidate>;

export const droppedCandidateSchema = z.discriminatedUnion("type", [
  appCandidateSchema.extend({ type: z.literal("app") }),
  urlCandidateSchema.extend({ type: z.literal("url") }),
]) satisfies z.ZodType<DroppedCandidate>;

export const dropRejectionSchema = z.enum([
  "unsupportedFile",
  "executableNotFound",
  "unsupportedUrl",
  "unsupportedShortcut",
]) satisfies z.ZodType<DropRejection>;

export const rejectedDropSchema = z.object({
  path: z.string(),
  reason: dropRejectionSchema,
}) satisfies z.ZodType<RejectedDrop>;

export const dropResolutionSchema = z.object({
  candidates: z.array(droppedCandidateSchema),
  rejected: z.array(rejectedDropSchema),
}) satisfies z.ZodType<DropResolution>;
