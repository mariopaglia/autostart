// Shared by the release script (which builds the notes) and the app (which reads them), so it has no imports.

type NotesLanguage = "en" | "pt-BR";

const END_MARKER = "<!-- release-notes:end -->";

function startMarker(language: NotesLanguage): string {
  return `<!-- release-notes:${language} -->`;
}

function section(language: NotesLanguage, notes: string): string {
  return `${startMarker(language)}\n${notes}\n${END_MARKER}`;
}

/** The visible headings keep the GitHub release page readable; the markers let the app pick one language. */
export function bilingualReleaseNotes(englishNotes: string, portugueseNotes: string): string {
  return [
    "## English",
    section("en", englishNotes),
    "## Português (Brasil)",
    section("pt-BR", portugueseNotes),
  ].join("\n\n");
}

function extractSection(body: string, language: NotesLanguage): string | null {
  const start = body.indexOf(startMarker(language));
  if (start === -1) return null;
  const contentStart = start + startMarker(language).length;
  const end = body.indexOf(END_MARKER, contentStart);
  if (end === -1) return null;
  return body.slice(contentStart, end).trim();
}

/** Releases published before the notes were bilingual have no markers and are shown as they are. */
export function localizedReleaseNotes(body: string, language: string): string {
  const preferred: NotesLanguage = language === "pt-BR" ? "pt-BR" : "en";
  return extractSection(body, preferred) ?? extractSection(body, "en") ?? body.trim();
}
