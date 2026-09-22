import { describe, expect, it } from "vitest";
import { launchItemSchema, profileSchema } from "./profile";

const PROFILE_ID = "6f1c1d52-8f8e-4c4b-9a53-2f0a5f3a1d10";
const ITEM_ID = "0b6f0c2e-3f4d-4a8e-9b1a-7c2d5e6f7a8b";

function buildProfile(overrides: Record<string, unknown> = {}) {
  return {
    id: PROFILE_ID,
    name: "Live MSFS 2024",
    trigger: { processName: "FlightSimulator2024.exe", label: "MSFS 2024" },
    ...overrides,
  };
}

describe("profileSchema", () => {
  it("applies defaults to a minimal profile", () => {
    const profile = profileSchema.parse(buildProfile());

    expect(profile.items).toEqual([]);
    expect(profile.enabled).toBe(true);
  });

  it("rejects a blank name", () => {
    const result = profileSchema.safeParse(buildProfile({ name: "   " }));

    expect(result.success).toBe(false);
    expect(result.error?.issues[0]?.path).toEqual(["name"]);
  });

  it("rejects a trigger that is not an executable", () => {
    const result = profileSchema.safeParse(
      buildProfile({ trigger: { processName: "FlightSimulator", label: "MSFS" } }),
    );

    expect(result.success).toBe(false);
    expect(result.error?.issues[0]?.path).toEqual(["trigger", "processName"]);
  });
});

describe("launchItemSchema", () => {
  it("parses an app item with defaults", () => {
    const item = launchItemSchema.parse({
      type: "app",
      id: ITEM_ID,
      name: "Volanta",
      exePath: "C:\\Apps\\Volanta\\Launcher.exe",
      processName: "Volanta.exe",
      args: "",
    });

    expect(item).toMatchObject({
      type: "app",
      delayMs: 800,
      runAsAdmin: false,
      onClose: "graceful",
      enabled: true,
    });
    expect(item.type === "app" && item.args).toBeUndefined();
  });

  it("accepts an app item saved by v0.1.0 with the new launch options defaulted", () => {
    const itemFromV010 = {
      type: "app",
      id: ITEM_ID,
      name: "Volanta",
      exePath: "C:\\Apps\\Volanta\\Launcher.exe",
      processName: "Volanta.exe",
      delayMs: 0,
      runAsAdmin: false,
      onClose: "graceful",
      enabled: true,
    };

    expect(profileSchema.parse(buildProfile({ items: [itemFromV010] })).items[0]).toMatchObject({
      processNameMode: "auto",
      startMinimized: false,
      waitForSimConnect: false,
    });
  });

  it("accepts only http and https urls", () => {
    const base = { type: "url", id: ITEM_ID, name: "SimBrief" };

    expect(launchItemSchema.safeParse({ ...base, url: "https://simbrief.com" }).success).toBe(true);
    expect(launchItemSchema.safeParse({ ...base, url: "ftp://example.com" }).success).toBe(false);
    expect(launchItemSchema.safeParse({ ...base, url: "not a url" }).success).toBe(false);
  });

  it("rejects a delay outside the allowed range", () => {
    const result = launchItemSchema.safeParse({
      type: "url",
      id: ITEM_ID,
      name: "Charts",
      url: "https://charts.navigraph.com",
      delayMs: 60_001,
    });

    expect(result.success).toBe(false);
  });

  it("rejects an unknown item type", () => {
    const result = launchItemSchema.safeParse({ type: "script", id: ITEM_ID, name: "x" });

    expect(result.success).toBe(false);
  });
});
