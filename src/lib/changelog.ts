const FIRST_RELEASE_HEADING = /^## \[(?!Unreleased\])/m;

/** Drops the file intro and the Unreleased section, which describe nothing the user has installed. */
export function releasedHistory(changelog: string): string {
  const match = FIRST_RELEASE_HEADING.exec(changelog);
  return match ? changelog.slice(match.index).trim() : "";
}
