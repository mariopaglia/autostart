import { describe, expect, it } from "vitest";
import {
  addTrigger,
  MAX_TRIGGERS,
  removeTrigger,
  supportsSimConnect,
  triggerFromProcessName,
} from "./trigger-presets";

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
