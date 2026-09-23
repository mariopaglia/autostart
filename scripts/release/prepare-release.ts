import { readFileSync, writeFileSync } from "node:fs";

import {
  bumpTypes,
  bumpVersion,
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
const { changelog, notes } = releaseChangelog(readFileSync("CHANGELOG.md", "utf8"), version, date);
if (!notes && !isDryRun) {
  console.error(
    'The "## [Unreleased]" section of CHANGELOG.md is empty: add the changes to release',
  );
  process.exit(1);
}

updateFile("package.json", (content) => setJsonVersion(content, version));
updateFile("src-tauri/tauri.conf.json", (content) => setJsonVersion(content, version));
updateFile("src-tauri/Cargo.toml", (content) => setCargoPackageVersion(content, version));
updateFile("src-tauri/Cargo.lock", (content) => setCargoLockVersion(content, "autostart", version));
writeFileSync("CHANGELOG.md", changelog);
writeFileSync(notesPath, `${notes}\n`);

console.log(version);
