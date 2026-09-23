import { useTranslation } from "react-i18next";
import { localizedReleaseNotes } from "@/lib/release-notes";
import { MarkdownContent } from "./MarkdownContent";

export function ReleaseNotes({ body }: { body: string }) {
  const { i18n } = useTranslation();

  return (
    <div className="max-h-60 overflow-y-auto rounded-xl border bg-muted/40 p-3 text-sm">
      <MarkdownContent>{localizedReleaseNotes(body, i18n.language)}</MarkdownContent>
    </div>
  );
}
