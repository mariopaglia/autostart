import { FolderOpen, ScrollText } from "lucide-react";
import { useTranslation } from "react-i18next";
import { TimelineRow } from "@/components/logs/TimelineRow";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";

function openLogDir() {
  commands.openLogDir().catch(notifyError);
}

export function LogsScreen() {
  const { t, i18n } = useTranslation();
  const sessionLog = useMonitorStore((state) => state.sessionLog);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center justify-between gap-3 border-b px-6 py-4">
        <h1 className="text-xl font-semibold tracking-tight">{t("logs.title")}</h1>
        <Button variant="outline" size="sm" onClick={openLogDir}>
          <FolderOpen />
          {t("logs.openFolder")}
        </Button>
      </header>

      <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
        {sessionLog ? (
          <>
            <div className="flex flex-wrap items-center gap-2">
              <h2 className="font-medium">{sessionLog.profileName}</h2>
              {sessionLog.isTest && <Badge variant="outline">{t("logs.test")}</Badge>}
              <span className="text-sm text-muted-foreground">
                {t("logs.startedAt", {
                  date: new Date(sessionLog.startedAtMs).toLocaleString(i18n.language),
                })}
              </span>
              {sessionLog.endedAtMs === null && <Badge>{t("logs.inProgress")}</Badge>}
            </div>
            <ol className="flex flex-col rounded-2xl border bg-card p-2" aria-live="polite">
              {sessionLog.entries.map((entry, index) => (
                <TimelineRow
                  key={`${String(entry.timestampMs)}-${String(index)}`}
                  entry={entry}
                  language={i18n.language}
                />
              ))}
            </ol>
          </>
        ) : (
          <Empty>
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <ScrollText />
              </EmptyMedia>
              <EmptyTitle>{t("logs.emptyTitle")}</EmptyTitle>
              <EmptyDescription>{t("logs.emptyDescription")}</EmptyDescription>
            </EmptyHeader>
          </Empty>
        )}
      </section>
    </div>
  );
}
