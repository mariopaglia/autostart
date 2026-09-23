// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { TriggerSelector } from "./TriggerSelector";

vi.mock("@/lib/tauri", () => ({
  commands: { listRunningProcesses: vi.fn().mockResolvedValue([]) },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

const MSFS_2020 = { processName: "FlightSimulator.exe", label: "MSFS 2020" };
const MSFS_2024 = { processName: "FlightSimulator2024.exe", label: "MSFS 2024" };

afterEach(cleanup);

function removeButton(label: string) {
  return screen.getByRole("button", { name: i18n.t("trigger.remove", { label }) });
}

describe("TriggerSelector", () => {
  it("removes a simulator while others remain", () => {
    const onChange = vi.fn();
    render(<TriggerSelector triggers={[MSFS_2024, MSFS_2020]} onChange={onChange} />);

    fireEvent.click(removeButton("MSFS 2024"));

    expect(onChange).toHaveBeenCalledWith([MSFS_2020]);
  });

  it("keeps the last simulator", () => {
    render(<TriggerSelector triggers={[MSFS_2024]} onChange={vi.fn()} />);

    expect(removeButton("MSFS 2024")).toHaveProperty("disabled", true);
  });

  it("adds a preset and marks the ones already in the profile", async () => {
    const onChange = vi.fn();
    render(<TriggerSelector triggers={[MSFS_2024]} onChange={onChange} />);

    fireEvent.click(screen.getByRole("button", { name: i18n.t("trigger.add") }));
    const presets = await screen.findAllByRole("option");
    const added = presets.find((option) => option.textContent.includes("MSFS 2024"));
    const available = presets.find((option) => option.textContent.includes("MSFS 2020"));

    expect(added?.getAttribute("aria-disabled")).toBe("true");
    if (available) fireEvent.click(available);
    expect(onChange).toHaveBeenCalledWith([MSFS_2024, MSFS_2020]);
  });
});
