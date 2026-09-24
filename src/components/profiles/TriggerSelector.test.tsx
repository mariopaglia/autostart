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
const XPLANE = { processName: "X-Plane.exe", label: "X-Plane 12" };

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

  it("sets a ready-made start target for MSFS", () => {
    const onChange = vi.fn();
    render(<TriggerSelector triggers={[MSFS_2024, MSFS_2020]} onChange={onChange} />);

    fireEvent.click(
      screen.getByRole("button", {
        name: i18n.t("trigger.launchTarget.edit", { label: "MSFS 2024" }),
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: i18n.t("trigger.launchTarget.steam") }));

    expect(onChange).toHaveBeenCalledWith([
      { ...MSFS_2024, launchTarget: "steam://rungameid/2537590" },
      MSFS_2020,
    ]);
  });

  it("rejects a typed target that is not an executable, Steam link or Store app", () => {
    const onChange = vi.fn();
    render(<TriggerSelector triggers={[XPLANE]} onChange={onChange} />);

    fireEvent.click(
      screen.getByRole("button", {
        name: i18n.t("trigger.launchTarget.edit", { label: "X-Plane 12" }),
      }),
    );
    expect(screen.queryByRole("button", { name: i18n.t("trigger.launchTarget.steam") })).toBeNull();
    fireEvent.change(screen.getByLabelText(i18n.t("trigger.launchTarget.field")), {
      target: { value: "X-Plane.exe" },
    });
    fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));

    expect(screen.getByText(i18n.t("validation.launchTarget"))).toBeDefined();
    expect(onChange).not.toHaveBeenCalled();
  });

  it("removes the start target", () => {
    const onChange = vi.fn();
    const withTarget = { ...MSFS_2024, launchTarget: "steam://rungameid/2537590" };
    render(<TriggerSelector triggers={[withTarget]} onChange={onChange} />);

    expect(screen.getByLabelText(i18n.t("trigger.launchTarget.set"))).toBeDefined();
    fireEvent.click(
      screen.getByRole("button", {
        name: i18n.t("trigger.launchTarget.edit", { label: "MSFS 2024" }),
      }),
    );
    fireEvent.click(screen.getByRole("button", { name: i18n.t("trigger.launchTarget.remove") }));

    expect(onChange).toHaveBeenCalledWith([MSFS_2024]);
  });
});
