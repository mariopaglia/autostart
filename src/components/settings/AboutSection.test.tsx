// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
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
});
