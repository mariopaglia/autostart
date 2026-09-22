import { describe, expect, it } from "vitest";
import { triggerFromProcessName } from "./trigger-presets";

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
