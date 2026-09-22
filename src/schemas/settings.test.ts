import { describe, expect, it } from "vitest";
import { settingsSchema } from "./settings";

describe("settingsSchema", () => {
  it("fills every field with its default", () => {
    expect(settingsSchema.parse({})).toEqual({
      activeProfileId: null,
      startWithWindows: false,
      startMinimized: true,
      gracefulTimeoutMs: 5_000,
      closeOnlyIfLaunchedByApp: true,
      theme: "dark",
      language: "pt-BR",
      onboardingCompleted: false,
      checkUpdatesOnStartup: true,
    });
  });

  it("rejects a graceful timeout below one second", () => {
    expect(settingsSchema.safeParse({ gracefulTimeoutMs: 200 }).success).toBe(false);
  });
});
