// @vitest-environment jsdom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { i18n } from "@/i18n";
import { bilingualReleaseNotes } from "@/lib/release-notes";
import { system } from "@/lib/tauri";
import { ReleaseNotes } from "./ReleaseNotes";

vi.mock("@/lib/tauri", () => ({
  system: { openUrl: vi.fn(() => Promise.resolve()) },
}));

const body = bilingualReleaseNotes(
  "### Fixed\n\n- A **bold** fix, see [the docs](https://example.com/en).",
  "### Corrigido\n\n- Uma correção em **negrito**, veja [a documentação](https://example.com/pt).",
);

afterEach(async () => {
  cleanup();
  await i18n.changeLanguage("pt-BR");
});

describe("ReleaseNotes", () => {
  it("renders the markdown of the notes in the app language", async () => {
    await i18n.changeLanguage("pt-BR");
    render(<ReleaseNotes body={body} />);

    expect(screen.getByRole("heading", { name: "Corrigido" })).toBeDefined();
    expect(screen.getByRole("listitem").textContent).toContain("Uma correção em negrito");
    expect(screen.getByText("negrito").tagName).toBe("STRONG");
    expect(screen.queryByText(/##|\*\*/)).toBeNull();
    expect(screen.queryByText("Fixed")).toBeNull();
  });

  it("shows the English notes when the app is in English", async () => {
    await i18n.changeLanguage("en");
    render(<ReleaseNotes body={body} />);

    expect(screen.getByRole("heading", { name: "Fixed" })).toBeDefined();
    expect(screen.queryByText("Corrigido")).toBeNull();
  });

  it("opens links in the browser instead of the app window", async () => {
    await i18n.changeLanguage("pt-BR");
    render(<ReleaseNotes body={body} />);

    fireEvent.click(screen.getByRole("link", { name: "a documentação" }));

    expect(system.openUrl).toHaveBeenCalledWith("https://example.com/pt");
  });
});
