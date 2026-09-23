import { readFileSync, writeFileSync } from "node:fs";

import { bilingualReleaseNotes } from "../../src/lib/release-notes.ts";
import {
  bumpTypes,
  bumpVersion,
  type ChangelogRelease,
  isBumpType,
  readJsonVersion,
  releaseChangelog,
  setCargoLockVersion,
  setCargoPackageVersion,
  setJsonVersion,
} from "./release-version.ts";

const [bump = "", notesPath, mode] = process.argv.slice(2);
const isDryRun = mode === "--dry-run";
if (!isBumpType(bump) || !notesPath) {
  console.error(
    `Usage: node scripts/release/prepare-release.ts <${bumpTypes.join("|")}> <notes-file> [--dry-run]`,
  );
  process.exit(1);
}

function updateFile(path: string, update: (content: string) => string): void {
  writeFileSync(path, update(readFileSync(path, "utf8")));
}

const version = bumpVersion(readJsonVersion(readFileSync("package.json", "utf8")), bump);
const date = new Date().toISOString().slice(0, 10);

function releaseChangelogFile(path: string): ChangelogRelease {
  const release = releaseChangelog(readFileSync(path, "utf8"), version, date);
  if (!release.notes && !isDryRun) {
    console.error(
      `The "## [Unreleased]" section of ${path} is empty: add the changes to release in both changelogs`,
    );
    process.exit(1);
  }
  return release;
}

const english = releaseChangelogFile("CHANGELOG.md");
const portuguese = releaseChangelogFile("CHANGELOG.pt-BR.md");

updateFile("package.json", (content) => setJsonVersion(content, version));
updateFile("src-tauri/tauri.conf.json", (content) => setJsonVersion(content, version));
updateFile("src-tauri/Cargo.toml", (content) => setCargoPackageVersion(content, version));
updateFile("src-tauri/Cargo.lock", (content) => setCargoLockVersion(content, "autostart", version));
writeFileSync("CHANGELOG.md", english.changelog);
writeFileSync("CHANGELOG.pt-BR.md", portuguese.changelog);
writeFileSync(notesPath, `${bilingualReleaseNotes(english.notes, portuguese.notes)}\n`);

console.log(version);
