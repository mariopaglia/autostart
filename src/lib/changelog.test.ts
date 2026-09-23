import { describe, expect, it } from "vitest";
import englishChangelog from "../../CHANGELOG.md?raw";
import portugueseChangelog from "../../CHANGELOG.pt-BR.md?raw";
import { releasedHistory } from "./changelog";

describe("releasedHistory", () => {
  it("keeps only the released versions and the link definitions", () => {
    const changelog = [
      "# Changelog",
      "",
      "Intro.",
      "",
      "## [Unreleased]",
      "",
      "- Pending change.",
      "",
      "## [0.2.0] - 2026-09-22",
      "",
      "- Released change.",
      "",
      "[0.2.0]: https://example.com/v0.2.0",
      "",
    ].join("\n");

    expect(releasedHistory(changelog)).toBe(
      "## [0.2.0] - 2026-09-22\n\n- Released change.\n\n[0.2.0]: https://example.com/v0.2.0",
    );
  });

  it("is empty before the first release", () => {
    expect(releasedHistory("# Changelog\n\n## [Unreleased]\n\n- Pending.\n")).toBe("");
  });

  it("reads the bundled changelogs, which start with the latest release", () => {
    for (const changelog of [englishChangelog, portugueseChangelog]) {
      expect(releasedHistory(changelog)).toMatch(/^## \[\d+\.\d+\.\d+\] - /);
    }
  });
});
