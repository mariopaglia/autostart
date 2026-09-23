import { describe, expect, it } from "vitest";

import {
  bumpVersion,
  readJsonVersion,
  releaseChangelog,
  setCargoLockVersion,
  setCargoPackageVersion,
  setJsonVersion,
} from "./release-version.ts";

describe("bumpVersion", () => {
  it.each([
    ["patch", "0.2.1"],
    ["minor", "0.3.0"],
    ["major", "1.0.0"],
  ] as const)("applies a %s bump", (bump, expected) => {
    expect(bumpVersion("0.2.0", bump)).toBe(expected);
  });

  it("rejects versions that are not plain semver", () => {
    expect(() => bumpVersion("0.2.0-beta.1", "patch")).toThrow();
  });
});

describe("JSON version", () => {
  const json = '{\n  "name": "autostart",\n  "version": "0.2.0",\n  "targets": ["nsis"]\n}\n';

  it("reads the version", () => {
    expect(readJsonVersion(json)).toBe("0.2.0");
  });

  it("replaces only the version and keeps the formatting", () => {
    expect(setJsonVersion(json, "0.3.0")).toBe(json.replace("0.2.0", "0.3.0"));
  });
});

describe("Cargo versions", () => {
  it("replaces the package version, not dependency versions", () => {
    const toml =
      '[package]\nname = "autostart"\nversion = "0.2.0"\n\n[dependencies]\nlog = { version = "0.4" }\n';
    expect(setCargoPackageVersion(toml, "0.2.1")).toBe(toml.replace('"0.2.0"', '"0.2.1"'));
  });

  it("replaces the lock entry of the given package only", () => {
    const lock =
      '[[package]]\nname = "autocfg"\nversion = "1.4.0"\n\n[[package]]\nname = "autostart"\nversion = "0.2.0"\n';
    expect(setCargoLockVersion(lock, "autostart", "0.2.1")).toBe(lock.replace("0.2.0", "0.2.1"));
  });

  it("fails when the package is missing", () => {
    expect(() => setCargoLockVersion("", "autostart", "0.2.1")).toThrow();
  });
});

describe("releaseChangelog", () => {
  const url = "https://github.com/mariopaglia/autostart/compare/";
  const changelog = [
    "# Changelog",
    "",
    "## [Unreleased]",
    "",
    "### Added",
    "",
    "- New feature.",
    "",
    "## [0.2.0] - 2026-09-22",
    "",
    "- Old entry.",
    "",
    `[Unreleased]: ${url}v0.2.0...HEAD`,
    `[0.2.0]: ${url}v0.1.0...v0.2.0`,
    "",
  ].join("\n");

  it("moves the unreleased entries under the new version and returns them as notes", () => {
    const result = releaseChangelog(changelog, "0.3.0", "2026-10-01");

    expect(result.notes).toBe("### Added\n\n- New feature.");
    expect(result.changelog).toBe(
      [
        "# Changelog",
        "",
        "## [Unreleased]",
        "",
        "## [0.3.0] - 2026-10-01",
        "",
        "### Added",
        "",
        "- New feature.",
        "",
        "## [0.2.0] - 2026-09-22",
        "",
        "- Old entry.",
        "",
        `[Unreleased]: ${url}v0.3.0...HEAD`,
        `[0.3.0]: ${url}v0.2.0...v0.3.0`,
        `[0.2.0]: ${url}v0.1.0...v0.2.0`,
        "",
      ].join("\n"),
    );
  });

  it("returns empty notes when there is nothing to release", () => {
    const empty = changelog.replace("### Added\n\n- New feature.\n\n", "");
    expect(releaseChangelog(empty, "0.3.0", "2026-10-01").notes).toBe("");
  });
});
