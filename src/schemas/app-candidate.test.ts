import { describe, expect, it } from "vitest";
import { appCandidatesSchema, dropResolutionSchema } from "./app-candidate";

describe("app candidate schemas", () => {
  it("accepts the lists returned by the backend", () => {
    const candidates = appCandidatesSchema.parse([
      {
        name: "Little Navmap",
        exePath: "C:\\Tools\\LittleNavmap\\littlenavmap.exe",
        workingDir: "C:\\Tools\\LittleNavmap",
        processName: "littlenavmap.exe",
        iconBase64: "iVBORw0KGgo",
        source: "installed",
        suggested: true,
      },
    ]);

    expect(candidates[0]?.suggested).toBe(true);
    expect(candidates[0]?.args).toBeUndefined();
  });

  it("accepts a drop resolution with app, url and rejected entries", () => {
    const resolution = dropResolutionSchema.parse({
      candidates: [
        {
          type: "app",
          name: "vPilot",
          exePath: "C:\\vPilot\\vPilot.exe",
          processName: "vPilot.exe",
          source: "dropped",
          suggested: true,
        },
        { type: "url", name: "SimBrief", url: "https://www.simbrief.com" },
      ],
      rejected: [{ path: "C:\\manual.pdf", reason: "unsupportedFile" }],
    });

    expect(resolution.candidates.map((candidate) => candidate.type)).toEqual(["app", "url"]);
    expect(resolution.rejected[0]?.reason).toBe("unsupportedFile");
  });

  it("rejects an unknown rejection reason", () => {
    const result = dropResolutionSchema.safeParse({
      candidates: [],
      rejected: [{ path: "C:\\a", reason: "tooBig" }],
    });

    expect(result.success).toBe(false);
  });
});
