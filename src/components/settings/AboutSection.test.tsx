// @vitest-environment jsdom
import "@/test/jsdom-polyfills";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { system } from "@/lib/tauri";
import { AboutSection } from "./AboutSection";

vi.mock("@/lib/tauri", () => ({
  system: { openUrl: vi.fn(() => Promise.resolve()) },
}));

afterEach(cleanup);

describe("AboutSection", () => {
  it("shows the copyright, the GPL notice and the no-warranty disclaimer", () => {
    render(<AboutSection version="1.2.3" />);

    expect(screen.getByText("1.2.3")).toBeDefined();
    expect(screen.getByText(/© 2026 Mario Paglia/)).toBeDefined();
    expect(screen.getByText(/GNU General Public License/)).toBeDefined();
    expect(i18n.t("settings.licenseNotice")).toMatch(/SEM NENHUMA GARANTIA|WITHOUT ANY WARRANTY/);
  });

  it("opens the full license text", () => {
    render(<AboutSection version="1.2.3" />);

    fireEvent.click(screen.getByRole("button", { name: i18n.t("settings.licenseName") }));

    expect(system.openUrl).toHaveBeenCalledWith(
      "https://github.com/mariopaglia/autostart/blob/main/LICENSE",
    );
  });

  it("opens the version history in the app language from the version", () => {
    render(<AboutSection version="1.2.3" />);

    fireEvent.click(screen.getByRole("button", { name: i18n.t("settings.viewChangelog") }));

    const dialog = screen.getByRole("dialog", { name: i18n.t("changelog.title") });
    expect(
      within(dialog).getAllByRole("heading", { name: /^\d+\.\d+\.\d+ - \d{4}-\d{2}-\d{2}$/ })
        .length,
    ).toBeGreaterThan(0);
    expect(within(dialog).queryByText(/Unreleased/)).toBeNull();
    expect(within(dialog).getAllByText("Adicionado").length).toBeGreaterThan(0);
  });
});
