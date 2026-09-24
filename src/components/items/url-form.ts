import type { z } from "zod";
import { DEFAULT_DELAY_MS, urlItemSchema } from "@/schemas/profile";

export const urlFormSchema = urlItemSchema.omit({ id: true });
export type UrlFormInput = z.input<typeof urlFormSchema>;
export type UrlFormOutput = z.output<typeof urlFormSchema>;

export const EMPTY_URL_FORM: UrlFormInput = {
  name: "",
  url: "https://",
  delayMs: DEFAULT_DELAY_MS,
  onlyForTriggers: [],
  enabled: true,
};
