import Markdown, { type Components } from "react-markdown";
import { useTranslation } from "react-i18next";
import { notifyError } from "@/lib/notify";
import { localizedReleaseNotes } from "@/lib/release-notes";
import { system } from "@/lib/tauri";

const MARKDOWN_COMPONENTS: Components = {
  h2: ({ children }) => <h3 className="mt-3 font-semibold first:mt-0">{children}</h3>,
  h3: ({ children }) => (
    <h4 className="mt-3 text-xs font-semibold tracking-wide text-muted-foreground uppercase first:mt-0">
      {children}
    </h4>
  ),
  p: ({ children }) => <p className="mt-2 first:mt-0">{children}</p>,
  ul: ({ children }) => <ul className="mt-1.5 list-disc space-y-1 pl-5">{children}</ul>,
  ol: ({ children }) => <ol className="mt-1.5 list-decimal space-y-1 pl-5">{children}</ol>,
  strong: ({ children }) => <strong className="font-semibold">{children}</strong>,
  code: ({ children }) => (
    <code className="rounded bg-muted px-1 py-0.5 font-mono text-xs">{children}</code>
  ),
  // Following a link inside the webview would replace the app, so it opens in the browser.
  a: ({ href, children }) => (
    <a
      href={href}
      className="text-primary underline underline-offset-2"
      onClick={(event) => {
        event.preventDefault();
        if (href) void system.openUrl(href).catch(notifyError);
      }}
    >
      {children}
    </a>
  ),
};

export function ReleaseNotes({ body }: { body: string }) {
  const { i18n } = useTranslation();

  return (
    <div className="max-h-60 overflow-y-auto rounded-xl border bg-muted/40 p-3 text-sm">
      <Markdown components={MARKDOWN_COMPONENTS}>
        {localizedReleaseNotes(body, i18n.language)}
      </Markdown>
    </div>
  );
}
