import { describe, expect, it } from "vitest";
import type { Profile } from "@/bindings/Profile";
import { profileSchema } from "@/schemas/profile";
import { createProfile, duplicateProfile, withFreshIds } from "./profile-factory";

function profileWithItem(): Profile {
  return {
    ...createProfile("Live"),
    items: [
      {
        type: "url",
        id: crypto.randomUUID(),
        name: "SimBrief",
        url: "https://simbrief.com",
        delayMs: 0,
        enabled: true,
      },
    ],
  };
}

describe("profile factory", () => {
  it("creates a valid profile with the MSFS 2024 trigger", () => {
    const profile = createProfile("  Live  ");

    expect(profileSchema.safeParse(profile).success).toBe(true);
    expect(profile.name).toBe("Live");
    expect(profile.triggers.map((trigger) => trigger.processName)).toEqual([
      "FlightSimulator2024.exe",
    ]);
  });

  it("regenerates the profile and item ids", () => {
    const original = profileWithItem();
    const copy = withFreshIds(original);

    expect(copy.id).not.toBe(original.id);
    expect(copy.items[0]?.id).not.toBe(original.items[0]?.id);
    expect(copy.items[0]?.name).toBe("SimBrief");
  });

  it("appends the localized suffix when duplicating", () => {
    const copy = duplicateProfile(profileWithItem(), " (cópia)");

    expect(copy.name).toBe("Live (cópia)");
  });

  it("creates the copy disabled so it does not take over the original's simulator", () => {
    expect(duplicateProfile(profileWithItem(), " (copy)").enabled).toBe(false);
  });
});
