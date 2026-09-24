// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { UrlItemForm } from "./UrlItemForm";

afterEach(cleanup);

function renderForm(url: string) {
  const onSubmit = vi.fn();
  render(
    <UrlItemForm
      initial={{ name: "SimHub", url, delayMs: 0, onlyForTriggers: [], enabled: true }}
      triggers={[{ processName: "FlightSimulator2024.exe", label: "MSFS 2024" }]}
      onSubmit={onSubmit}
      onCancel={vi.fn()}
    />,
  );
  fireEvent.click(screen.getByRole("button", { name: i18n.t("common.save") }));
  return onSubmit;
}

describe("UrlItemForm", () => {
  it("saves a Steam launch link and explains it is never closed", async () => {
    const onSubmit = renderForm("steam://rungameid/1234560");

    expect(screen.getByText(i18n.t("itemForm.urlHint"))).toBeDefined();
    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledWith(
        expect.objectContaining({ url: "steam://rungameid/1234560" }),
      );
    });
  });

  it("rejects other schemes", async () => {
    const onSubmit = renderForm("steam://uninstall/123");

    expect(await screen.findByText(i18n.t("validation.url"))).toBeDefined();
    expect(onSubmit).not.toHaveBeenCalled();
  });
});
