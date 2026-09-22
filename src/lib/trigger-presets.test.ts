import { describe, expect, it } from "vitest";
import { supportsSimConnect, triggerFromProcessName } from "./trigger-presets";

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
  it("accepts MSFS 2020 and 2024 regardless of case", () => {
    expect(supportsSimConnect({ processName: "FlightSimulator.exe", label: "MSFS 2020" })).toBe(
      true,
    );
    expect(supportsSimConnect({ processName: "flightsimulator2024.EXE", label: "MSFS 2024" })).toBe(
      true,
    );
  });

  it("rejects other simulators", () => {
    expect(supportsSimConnect({ processName: "X-Plane.exe", label: "X-Plane 12" })).toBe(false);
  });
});
