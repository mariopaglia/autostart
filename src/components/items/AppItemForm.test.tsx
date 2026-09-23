// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
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

afterEach(cleanup);

function renderForm(otherExePaths: string[], onSubmit = vi.fn()) {
  render(
    <AppItemForm
      initial={volanta}
      otherExePaths={otherExePaths}
      simConnectSupported
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
});
