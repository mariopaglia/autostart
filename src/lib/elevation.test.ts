import { describe, expect, it } from "vitest";
import type { AppItem } from "@/bindings/AppItem";
import type { Profile } from "@/bindings/Profile";
import { needsElevationWarning } from "./elevation";
import { createProfile } from "./profile-factory";

function profileWith(overrides: Partial<AppItem>): Profile {
  const item: AppItem = {
    id: crypto.randomUUID(),
    name: "SPAD.neXt",
    exePath: "C:\\SPAD\\Spad.exe",
    processName: "Spad.exe",
    processNameMode: "auto",
    delayMs: 0,
    runAsAdmin: true,
    startMinimized: false,
    waitForSimConnect: false,
    onClose: "graceful",
    enabled: true,
    ...overrides,
  };
  return { ...createProfile("MSFS 2024"), items: [{ type: "app", ...item }] };
}

describe("needsElevationWarning", () => {
  it("warns about an enabled admin app that must be closed", () => {
    expect(needsElevationWarning(profileWith({}), false)).toBe(true);
  });

  it("does not warn when AutoStart is already elevated", () => {
    expect(needsElevationWarning(profileWith({}), true)).toBe(false);
  });

  it("ignores admin apps kept open or disabled", () => {
    expect(needsElevationWarning(profileWith({ onClose: "keep" }), false)).toBe(false);
    expect(needsElevationWarning(profileWith({ enabled: false }), false)).toBe(false);
  });

  it("ignores apps that do not run as administrator", () => {
    expect(needsElevationWarning(profileWith({ runAsAdmin: false }), false)).toBe(false);
  });
});
