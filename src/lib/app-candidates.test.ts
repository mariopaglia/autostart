import { describe, expect, it } from "vitest";
import type { AppCandidate } from "@/bindings/AppCandidate";
import { DEFAULT_DELAY_MS } from "@/schemas/profile";
import {
  candidateToAppForm,
  candidateToItem,
  executableChoiceToAppForm,
  groupCandidates,
  isAlreadyInProfile,
} from "./app-candidates";

const littleNavmap: AppCandidate = {
  name: "Little Navmap",
  exePath: "C:\\Tools\\LittleNavmap\\littlenavmap.exe",
  args: "--safe",
  workingDir: "C:\\Tools\\LittleNavmap",
  processName: "littlenavmap.exe",
  iconBase64: "iVBORw0KGgo",
  source: "installed",
  suggested: true,
};

describe("candidateToAppForm", () => {
  it("carries the candidate data over the form defaults", () => {
    const form = candidateToAppForm(littleNavmap);

    expect(form).toMatchObject({
      name: "Little Navmap",
      exePath: littleNavmap.exePath,
      args: "--safe",
      workingDir: "C:\\Tools\\LittleNavmap",
      processName: "littlenavmap.exe",
      processNameMode: "auto",
      iconBase64: "iVBORw0KGgo",
      delayMs: DEFAULT_DELAY_MS,
      onClose: "graceful",
      enabled: true,
    });
  });

  it("uses empty text for missing arguments and working folder", () => {
    const form = candidateToAppForm({ ...littleNavmap, args: undefined, workingDir: undefined });

    expect(form.args).toBe("");
    expect(form.workingDir).toBe("");
  });

  it("shortens names longer than the item limit", () => {
    expect(candidateToAppForm({ ...littleNavmap, name: "x".repeat(120) }).name).toHaveLength(80);
  });
});

describe("executableChoiceToAppForm", () => {
  it("fills the form from the inspected executable", () => {
    const form = executableChoiceToAppForm({
      path: "C:\\vPilot\\vPilot.exe",
      info: { processName: "vPilot.exe", productName: "vPilot" },
    });

    expect(form).toMatchObject({
      name: "vPilot",
      exePath: "C:\\vPilot\\vPilot.exe",
      processName: "vPilot.exe",
      processNameMode: "auto",
    });
  });
});

describe("candidateToItem", () => {
  it("builds enabled items with default options and a new id each time", () => {
    const first = candidateToItem({ type: "app", ...littleNavmap });
    const second = candidateToItem({ type: "app", ...littleNavmap });

    expect(first).toMatchObject({
      type: "app",
      name: "Little Navmap",
      args: "--safe",
      enabled: true,
      runAsAdmin: false,
      delayMs: DEFAULT_DELAY_MS,
    });
    expect(first?.id).not.toBe(second?.id);
  });

  it("builds url items", () => {
    expect(
      candidateToItem({ type: "url", name: "SimBrief", url: "https://www.simbrief.com" }),
    ).toMatchObject({ type: "url", name: "SimBrief", url: "https://www.simbrief.com" });
  });

  it("returns undefined when the item would be invalid", () => {
    expect(candidateToItem({ type: "app", ...littleNavmap, name: "  " })).toBeUndefined();
  });
});

describe("isAlreadyInProfile", () => {
  it("compares paths ignoring case and slash direction", () => {
    const paths = ["C:\\Tools\\LittleNavmap\\littlenavmap.exe"];

    expect(isAlreadyInProfile("c:/tools/littlenavmap/LITTLENAVMAP.exe", paths)).toBe(true);
    expect(isAlreadyInProfile("C:\\Tools\\Volanta\\Volanta.exe", paths)).toBe(false);
    expect(isAlreadyInProfile("", [""])).toBe(false);
  });
});

describe("groupCandidates", () => {
  const notepad: AppCandidate = {
    ...littleNavmap,
    name: "Notepad++",
    exePath: "C:\\Program Files\\Notepad++\\notepad++.exe",
    suggested: false,
  };
  const volanta: AppCandidate = {
    ...littleNavmap,
    name: "Volanta",
    exePath: "C:\\Users\\me\\AppData\\Local\\Volanta\\Volanta.exe",
  };
  const candidates = [littleNavmap, notepad, volanta];

  it("puts suggestions in their own group, keeping the order", () => {
    const groups = groupCandidates(candidates, "", true);

    expect(groups.suggested.map((candidate) => candidate.name)).toEqual([
      "Little Navmap",
      "Volanta",
    ]);
    expect(groups.others.map((candidate) => candidate.name)).toEqual(["Notepad++"]);
  });

  it("filters by name or path ignoring case", () => {
    expect(groupCandidates(candidates, "NAVMAP", true).suggested).toEqual([littleNavmap]);
    expect(groupCandidates(candidates, "appdata", true).suggested).toEqual([volanta]);
    expect(groupCandidates(candidates, "zzz", true)).toEqual({ suggested: [], others: [] });
  });

  it("keeps every match together when suggestions are not wanted", () => {
    expect(groupCandidates(candidates, "", false)).toEqual({ suggested: [], others: candidates });
  });
});
