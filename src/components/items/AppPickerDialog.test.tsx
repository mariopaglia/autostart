// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { AppCandidate } from "@/bindings/AppCandidate";
import { i18n } from "@/i18n";
import { commands } from "@/lib/tauri";
import { AppPickerDialog } from "./AppPickerDialog";

vi.mock("@/lib/tauri", () => ({
  commands: { listInstalledApps: vi.fn(), listOpenApps: vi.fn(), inspectExe: vi.fn() },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

function candidate(name: string, suggested: boolean): AppCandidate {
  return {
    name,
    exePath: `C:\\Tools\\${name}\\${name}.exe`,
    processName: `${name}.exe`,
    source: "installed",
    suggested,
  };
}

const installed = [candidate("Notepad++", false), candidate("Little Navmap", true)];

beforeEach(() => {
  vi.mocked(commands.listInstalledApps).mockResolvedValue(installed);
  vi.mocked(commands.listOpenApps).mockResolvedValue([]);
});

afterEach(cleanup);

function renderPicker(onPick = vi.fn(), profileExePaths: string[] = []) {
  render(
    <AppPickerDialog open profileExePaths={profileExePaths} onPick={onPick} onClose={vi.fn()} />,
  );
  return onPick;
}

function rowNames(): string[] {
  return screen
    .getAllByRole("option")
    .map((option) => option.querySelector(".font-medium")?.textContent ?? "");
}

describe("AppPickerDialog", () => {
  it("lists suggestions above the other installed apps", async () => {
    renderPicker();

    expect(await screen.findByText("Little Navmap")).toBeDefined();
    expect(screen.getByText(i18n.t("appPicker.suggested"))).toBeDefined();
    expect(rowNames()).toEqual(["Little Navmap", "Notepad++"]);
  });

  it("filters by the search text", async () => {
    renderPicker();
    await screen.findByText("Little Navmap");

    fireEvent.change(screen.getByPlaceholderText(i18n.t("appPicker.search")), {
      target: { value: "notepad" },
    });

    expect(rowNames()).toEqual(["Notepad++"]);
  });

  it("marks apps that are already in the profile", async () => {
    renderPicker(vi.fn(), ["c:\\tools\\little navmap\\LITTLE NAVMAP.exe"]);

    const row = (await screen.findByText("Little Navmap")).closest("[role=option]");

    expect(row).not.toBeNull();
    expect(within(row as HTMLElement).getByText(i18n.t("appPicker.alreadyAdded"))).toBeDefined();
  });

  it("hands the chosen app over as pre-filled form values", async () => {
    const onPick = renderPicker();

    fireEvent.click(await screen.findByText("Little Navmap"));

    expect(onPick).toHaveBeenCalledWith(
      expect.objectContaining({
        name: "Little Navmap",
        exePath: "C:\\Tools\\Little Navmap\\Little Navmap.exe",
        processNameMode: "auto",
      }),
    );
  });
});
