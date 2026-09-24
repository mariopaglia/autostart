// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import type { Trigger } from "@/bindings/Trigger";
import { i18n } from "@/i18n";
import { candidateToAppForm } from "@/lib/app-candidates";
import { AppItemForm } from "./AppItemForm";

const volanta = candidateToAppForm({
  name: "Volanta",
  exePath: "C:\\Users\\me\\AppData\\Local\\Volanta\\Volanta.exe",
  args: "--tray",
  processName: "Volanta.exe",
  source: "installed",
  suggested: true,
});

const MSFS_2020: Trigger = { processName: "FlightSimulator.exe", label: "MSFS 2020" };
const MSFS_2024: Trigger = { processName: "FlightSimulator2024.exe", label: "MSFS 2024" };

afterEach(cleanup);

function renderForm(otherExePaths: string[], onSubmit = vi.fn(), triggers = [MSFS_2024]) {
  render(
    <AppItemForm
      initial={volanta}
      otherExePaths={otherExePaths}
      triggers={triggers}
      onSubmit={onSubmit}
      onCancel={vi.fn()}
    />,
  );
  return onSubmit;
}

describe("AppItemForm", () => {
  it("opens pre-filled from a candidate", () => {
    renderForm([]);

    expect(screen.getByLabelText(i18n.t("itemForm.name"))).toHaveProperty("value", "Volanta");
    expect(screen.getByLabelText(i18n.t("itemForm.exePath"))).toHaveProperty(
      "value",
      volanta.exePath,
    );
    expect(screen.getByLabelText(i18n.t("itemForm.args"))).toHaveProperty("value", "--tray");
    expect(screen.queryByText(i18n.t("itemForm.duplicate"))).toBeNull();
  });

  it("warns about a duplicate executable but still saves it", async () => {
    const onSubmit = renderForm(["c:\\users\\me\\appdata\\local\\volanta\\VOLANTA.exe"]);

    expect(screen.getByText(i18n.t("itemForm.duplicate"))).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledOnce();
    });
    expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({
      name: "Volanta",
      exePath: volanta.exePath,
      processNameMode: "auto",
    });
  });

  it("saves the reopen-on-crash option", async () => {
    const onSubmit = renderForm([]);

    fireEvent.click(screen.getByRole("switch", { name: i18n.t("itemForm.restartOnCrash") }));
    fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledOnce();
    });
    expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({ restartOnCrash: true });
  });

  it("hides the simulator restriction when the profile has a single simulator", () => {
    renderForm([]);

    expect(screen.queryByText(i18n.t("itemForm.onlyForTriggers"))).toBeNull();
  });

  it("restricts the item to the picked simulators", async () => {
    const onSubmit = renderForm([], vi.fn(), [MSFS_2020, MSFS_2024]);

    fireEvent.click(screen.getByRole("button", { name: "MSFS 2024" }));
    fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledOnce();
    });
    expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({
      onlyForTriggers: ["FlightSimulator2024.exe"],
    });
  });

  it("keeps opening before the simulator and waiting for SimConnect mutually exclusive", async () => {
    const onSubmit = renderForm([]);
    const waitSwitch = screen.getByRole("switch", { name: i18n.t("itemForm.waitForSimConnect") });
    const beforeSwitch = screen.getByRole("switch", {
      name: i18n.t("itemForm.launchBeforeSimulator"),
    });

    fireEvent.click(waitSwitch);
    fireEvent.click(beforeSwitch);

    expect(waitSwitch.getAttribute("aria-checked")).toBe("false");
    expect(screen.getByText(i18n.t("itemForm.launchBeforeSimulatorHint"))).toBeDefined();
    fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledOnce();
    });
    expect(onSubmit.mock.calls[0]?.[0]).toMatchObject({
      launchBeforeSimulator: true,
      waitForSimConnect: false,
    });

    fireEvent.click(waitSwitch);
    expect(beforeSwitch.getAttribute("aria-checked")).toBe("false");
  });
});
