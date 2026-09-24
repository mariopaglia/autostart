import { describe, expect, it } from "vitest";
import {
  addTrigger,
  launchTargetPresets,
  withLaunchTarget,
  MAX_TRIGGERS,
  removeTrigger,
  restrictionLabels,
  supportsSimConnect,
  triggerFromProcessName,
  withTriggers,
} from "./trigger-presets";
import { launchTargetSchema } from "@/schemas/profile";
import { createProfile } from "./profile-factory";

const MSFS_2020 = { processName: "FlightSimulator.exe", label: "MSFS 2020" };
const MSFS_2024 = { processName: "FlightSimulator2024.exe", label: "MSFS 2024" };

describe("addTrigger", () => {
  it("adds a new simulator at the end", () => {
    expect(addTrigger([MSFS_2024], MSFS_2020)).toEqual([MSFS_2024, MSFS_2020]);
  });

  it("ignores a simulator already in the list, regardless of case", () => {
    const repeated = { processName: "flightsimulator2024.EXE", label: "Copy" };

    expect(addTrigger([MSFS_2024], repeated)).toEqual([MSFS_2024]);
  });

  it("stops at the maximum", () => {
    const full = Array.from({ length: MAX_TRIGGERS }, (_, index) =>
      triggerFromProcessName(`Sim${String(index)}`),
    );

    expect(addTrigger(full, MSFS_2024)).toEqual(full);
  });
});

describe("removeTrigger", () => {
  it("removes a simulator", () => {
    expect(removeTrigger([MSFS_2024, MSFS_2020], MSFS_2024)).toEqual([MSFS_2020]);
  });

  it("keeps the last simulator", () => {
    expect(removeTrigger([MSFS_2024], MSFS_2024)).toEqual([MSFS_2024]);
  });
});

describe("triggerFromProcessName", () => {
  it("reuses the preset label for known simulators", () => {
    expect(triggerFromProcessName("x-plane.exe")).toEqual({
      processName: "X-Plane.exe",
      label: "X-Plane 12",
    });
  });

  it("adds the .exe extension and derives a label for custom processes", () => {
    expect(triggerFromProcessName("Prepar3D")).toEqual({
      processName: "Prepar3D.exe",
      label: "Prepar3D",
    });
  });
});

describe("supportsSimConnect", () => {
  const xPlane = { processName: "X-Plane.exe", label: "X-Plane 12" };

  it("accepts MSFS 2020 and 2024 regardless of case", () => {
    expect(supportsSimConnect([{ processName: "FlightSimulator.exe", label: "MSFS 2020" }])).toBe(
      true,
    );
    expect(
      supportsSimConnect([{ processName: "flightsimulator2024.EXE", label: "MSFS 2024" }]),
    ).toBe(true);
  });

  it("accepts a profile where any simulator is MSFS", () => {
    expect(
      supportsSimConnect([xPlane, { processName: "FlightSimulator2024.exe", label: "MSFS 2024" }]),
    ).toBe(true);
  });

  it("rejects profiles without MSFS", () => {
    expect(supportsSimConnect([xPlane])).toBe(false);
    expect(supportsSimConnect([])).toBe(false);
  });
});

describe("withTriggers", () => {
  it("drops a removed simulator from the items restricted to it", () => {
    const profile = {
      ...createProfile("MSFS"),
      triggers: [MSFS_2020, MSFS_2024],
      items: [
        {
          type: "url" as const,
          id: crypto.randomUUID(),
          name: "Only 2020",
          url: "https://a.com",
          delayMs: 0,
          onlyForTriggers: ["FlightSimulator.exe"],
          enabled: true,
        },
        {
          type: "url" as const,
          id: crypto.randomUUID(),
          name: "Both",
          url: "https://b.com",
          delayMs: 0,
          onlyForTriggers: ["flightsimulator.exe", "FlightSimulator2024.exe"],
          enabled: true,
        },
      ],
    };

    const updated = withTriggers(profile, [MSFS_2024]);

    expect(updated.triggers).toEqual([MSFS_2024]);
    expect(updated.items.map((item) => item.onlyForTriggers)).toEqual([
      [],
      ["FlightSimulator2024.exe"],
    ]);
  });
});

describe("restrictionLabels", () => {
  it("names the simulators an item is restricted to, in profile order", () => {
    expect(
      restrictionLabels([MSFS_2020, MSFS_2024], ["flightsimulator2024.exe", "FlightSimulator.exe"]),
    ).toEqual(["MSFS 2020", "MSFS 2024"]);
    expect(restrictionLabels([MSFS_2020, MSFS_2024], [])).toEqual([]);
  });
});

describe("launch targets", () => {
  it("offers Steam and Microsoft Store only for MSFS", () => {
    expect(launchTargetPresets(MSFS_2024).map((preset) => preset.source)).toEqual([
      "steam",
      "microsoftStore",
    ]);
    expect(launchTargetPresets({ processName: "flightsimulator.EXE", label: "x" })).toHaveLength(2);
    expect(launchTargetPresets({ processName: "X-Plane.exe", label: "X-Plane 12" })).toEqual([]);
  });

  it("offers presets that pass the profile validation", () => {
    for (const trigger of [MSFS_2020, MSFS_2024]) {
      for (const { target } of launchTargetPresets(trigger)) {
        expect(launchTargetSchema.safeParse(target).success, target).toBe(true);
      }
    }
  });

  it("sets and removes the start target of one simulator", () => {
    const withTarget = withLaunchTarget(
      [MSFS_2020, MSFS_2024],
      "flightsimulator2024.exe",
      "steam://rungameid/2537590",
    );

    expect(withTarget).toEqual([
      MSFS_2020,
      { ...MSFS_2024, launchTarget: "steam://rungameid/2537590" },
    ]);
    expect(withLaunchTarget(withTarget, "FlightSimulator2024.exe", undefined)).toEqual([
      MSFS_2020,
      MSFS_2024,
    ]);
  });
});
