export const bumpTypes = ["patch", "minor", "major"] as const;
export type BumpType = (typeof bumpTypes)[number];

export function isBumpType(value: string): value is BumpType {
  return (bumpTypes as readonly string[]).includes(value);
}

export function bumpVersion(version: string, bump: BumpType): string {
  const match = /^(\d+)\.(\d+)\.(\d+)$/.exec(version);
  if (!match) {
    throw new Error(`Unsupported version "${version}"`);
  }
  const [major, minor, patch] = match.slice(1).map(Number) as [number, number, number];
  switch (bump) {
    case "major":
      return `${major + 1}.0.0`;
    case "minor":
      return `${major}.${minor + 1}.0`;
    case "patch":
      return `${major}.${minor}.${patch + 1}`;
  }
}

export function readJsonVersion(json: string): string {
  const match = /"version":\s*"([^"]+)"/.exec(json);
  if (!match?.[1]) {
    throw new Error('No "version" field found');
  }
  return match[1];
}

// Regex replacements instead of JSON/TOML round-trips keep the files byte-identical to what Prettier and cargo produce.
export function setJsonVersion(json: string, version: string): string {
  return replaceOnce(json, /("version":\s*")[^"]+(")/, version);
}

export function setCargoPackageVersion(cargoToml: string, version: string): string {
  return replaceOnce(cargoToml, /(\[package\][^[]*?\nversion = ")[^"]+(")/, version);
}

export function setCargoLockVersion(
  cargoLock: string,
  packageName: string,
  version: string,
): string {
  const pattern = new RegExp(`(\\nname = "${packageName}"\\nversion = ")[^"]+(")`);
  return replaceOnce(cargoLock, pattern, version);
}

function replaceOnce(text: string, pattern: RegExp, version: string): string {
  if (!pattern.test(text)) {
    throw new Error(`Version pattern ${pattern} not found`);
  }
  return text.replace(pattern, `$1${version}$2`);
}

export interface ChangelogRelease {
  changelog: string;
  notes: string;
}

const unreleasedHeading = "## [Unreleased]";

export function releaseChangelog(
  changelog: string,
  version: string,
  date: string,
): ChangelogRelease {
  const headingStart = changelog.indexOf(unreleasedHeading);
  if (headingStart === -1) {
    throw new Error(`CHANGELOG.md has no "${unreleasedHeading}" section`);
  }
  const bodyStart = headingStart + unreleasedHeading.length;
  const nextSection = /\n## \[|\n\[Unreleased\]:/.exec(changelog.slice(bodyStart));
  const bodyEnd = nextSection ? bodyStart + nextSection.index : changelog.length;
  const notes = changelog.slice(bodyStart, bodyEnd).trim();
  if (!notes) {
    throw new Error(`The "${unreleasedHeading}" section of CHANGELOG.md is empty`);
  }

  const released =
    changelog.slice(0, bodyStart) +
    `\n\n## [${version}] - ${date}\n\n${notes}\n` +
    changelog.slice(bodyEnd);
  return { changelog: updateCompareLinks(released, version), notes };
}

function updateCompareLinks(changelog: string, version: string): string {
  const unreleasedLink = /^\[Unreleased\]: (.+\/compare\/)(v[^.\s]+\.[^.\s]+\.[^.\s]+)\.\.\.HEAD$/m;
  const match = unreleasedLink.exec(changelog);
  if (!match) {
    return changelog;
  }
  const [, compareUrl, previousTag] = match;
  return changelog.replace(
    unreleasedLink,
    `[Unreleased]: ${compareUrl}v${version}...HEAD\n[${version}]: ${compareUrl}${previousTag}...v${version}`,
  );
}
