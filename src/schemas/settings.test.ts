import { describe, expect, it } from "vitest";
import { settingsSchema } from "./settings";

describe("settingsSchema", () => {
  it("fills every field with its default", () => {
    expect(settingsSchema.parse({})).toEqual({
      startWithWindows: false,
      startMinimized: true,
      gracefulTimeoutMs: 5_000,
      closeDelayMs: 60_000,
      closeOnlyIfLaunchedByApp: true,
      showNotifications: true,
      theme: "dark",
      language: "pt-BR",
      onboardingCompleted: false,
      checkUpdatesOnStartup: true,
    });
  });

  it("rejects a graceful timeout below one second", () => {
    expect(settingsSchema.safeParse({ gracefulTimeoutMs: 200 }).success).toBe(false);
  });

  it("accepts a close delay from zero up to ten minutes", () => {
    expect(settingsSchema.safeParse({ closeDelayMs: 0 }).success).toBe(true);
    expect(settingsSchema.safeParse({ closeDelayMs: 600_000 }).success).toBe(true);
    expect(settingsSchema.safeParse({ closeDelayMs: 900_000 }).success).toBe(false);
  });

  it("ignores the active profile saved by v0.2.0", () => {
    expect(settingsSchema.parse({ activeProfileId: null })).not.toHaveProperty("activeProfileId");
  });
});
