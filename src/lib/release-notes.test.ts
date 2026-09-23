import { describe, expect, it } from "vitest";
import { bilingualReleaseNotes, localizedReleaseNotes } from "./release-notes";

const english = "### Fixed\n\n- A bug.";
const portuguese = "### Corrigido\n\n- Um bug.";
const body = bilingualReleaseNotes(english, portuguese);

describe("release notes", () => {
  it("shows both languages under visible headings on the release page", () => {
    expect(body).toContain("## English");
    expect(body).toContain("## Português (Brasil)");
    expect(body.indexOf(english)).toBeLessThan(body.indexOf(portuguese));
  });

  it("picks the notes in the app language", () => {
    expect(localizedReleaseNotes(body, "pt-BR")).toBe(portuguese);
    expect(localizedReleaseNotes(body, "en")).toBe(english);
  });

  it("falls back to English for other languages or a missing translation", () => {
    expect(localizedReleaseNotes(body, "de")).toBe(english);
    expect(localizedReleaseNotes(body.replace("<!-- release-notes:pt-BR -->", ""), "pt-BR")).toBe(
      english,
    );
  });

  it("shows notes without language markers as they are", () => {
    expect(localizedReleaseNotes("  ### Added\n\n- Old release.\n", "pt-BR")).toBe(
      "### Added\n\n- Old release.",
    );
  });
});
