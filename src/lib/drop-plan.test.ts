import { describe, expect, it } from "vitest";
import type { DroppedCandidate } from "@/bindings/DroppedCandidate";
import { planDrop } from "./drop-plan";

function app(name: string): DroppedCandidate {
  return {
    type: "app",
    name,
    exePath: `C:\\Tools\\${name}\\${name}.exe`,
    processName: `${name}.exe`,
    source: "dropped",
    suggested: true,
  };
}

const pdf = { path: "C:\\manual.pdf", reason: "unsupportedFile" } as const;

describe("planDrop", () => {
  it("opens the pre-filled form for a single app", () => {
    const plan = planDrop({ candidates: [app("Volanta")], rejected: [] });

    expect(plan.action).toMatchObject({
      kind: "openForm",
      target: {
        mode: "create",
        type: "app",
        initial: { name: "Volanta", processName: "Volanta.exe" },
      },
    });
    expect(plan.rejected).toEqual([]);
  });

  it("opens the pre-filled url form for a single web shortcut", () => {
    const plan = planDrop({
      candidates: [{ type: "url", name: "SimBrief", url: "https://www.simbrief.com" }],
      rejected: [],
    });

    expect(plan.action).toMatchObject({
      kind: "openForm",
      target: { type: "url", initial: { name: "SimBrief", url: "https://www.simbrief.com" } },
    });
  });

  it("appends several candidates in drop order", () => {
    const plan = planDrop({
      candidates: [app("LittleNavmap"), app("Volanta"), app("vPilot")],
      rejected: [],
    });

    expect(plan.action.kind).toBe("append");
    const items = plan.action.kind === "append" ? plan.action.items : [];
    expect(items.map((item) => item.name)).toEqual(["LittleNavmap", "Volanta", "vPilot"]);
    expect(items.every((item) => item.enabled)).toBe(true);
  });

  it("handles the accepted file and reports the rejected one in a mixed drop", () => {
    const plan = planDrop({ candidates: [app("Volanta")], rejected: [pdf] });

    expect(plan.action.kind).toBe("openForm");
    expect(plan.rejected).toEqual([pdf]);
  });

  it("does nothing but report when every file is rejected", () => {
    expect(planDrop({ candidates: [], rejected: [pdf] })).toEqual({
      action: { kind: "none" },
      rejected: [pdf],
    });
  });

  it("reports candidates that would be invalid items instead of appending them", () => {
    const plan = planDrop({
      candidates: [app("Volanta"), { ...app("x"), name: " " }],
      rejected: [],
    });

    expect(plan.action.kind).toBe("append");
    expect(plan.rejected).toEqual([{ path: "C:\\Tools\\x\\x.exe", reason: "unsupportedFile" }]);
  });
});
