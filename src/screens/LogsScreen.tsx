import { useState } from "react";
import { FolderOpen, ScrollText, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { SessionLog } from "@/bindings/SessionLog";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
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
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { notifyError } from "@/lib/notify";
import { errorCount, selectedSession, sessionKey, sessionList } from "@/lib/session-history";
import { commands } from "@/lib/tauri";
import { useLogsStore } from "@/stores/logs-store";
import { useMonitorStore } from "@/stores/monitor-store";

function openLogDir() {
  commands.openLogDir().catch(notifyError);
}

function SessionSummary({ session }: { session: SessionLog }) {
  const { t, i18n } = useTranslation();
  const errors = errorCount(session);

  return (
    <span className="flex items-center gap-2">
      <span className="font-medium">{session.profileName}</span>
      <span className="text-muted-foreground">
        {new Date(session.startedAtMs).toLocaleString(i18n.language)}
      </span>
      {session.isTest && <Badge variant="outline">{t("logs.test")}</Badge>}
      {session.endedAtMs === null && <Badge>{t("logs.inProgress")}</Badge>}
      {errors > 0 && <Badge variant="destructive">{t("logs.errors", { count: errors })}</Badge>}
    </span>
  );
}

export function LogsScreen() {
  const { t, i18n } = useTranslation();
  const liveSession = useMonitorStore((state) => state.sessionLog);
  const history = useLogsStore((state) => state.history);
  const clearHistory = useLogsStore((state) => state.clear);
  const [pickedKey, setPickedKey] = useState<string | null>(null);
  const [confirmingClear, setConfirmingClear] = useState(false);

  const sessions = sessionList(liveSession, history);
  const shown = selectedSession(sessions, pickedKey);

  function confirmClear() {
    setPickedKey(null);
    void clearHistory();
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <header className="flex items-center justify-between gap-3 border-b px-6 py-4">
        <h1 className="text-xl font-semibold tracking-tight">{t("logs.title")}</h1>
        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={history.length === 0}
            onClick={() => {
              setConfirmingClear(true);
            }}
          >
            <Trash2 />
            {t("logs.clearHistory")}
          </Button>
          <Button variant="outline" size="sm" onClick={openLogDir}>
            <FolderOpen />
            {t("logs.openFolder")}
          </Button>
        </div>
      </header>

      <section className="flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-6">
        {shown ? (
          <>
            <Select value={sessionKey(shown)} onValueChange={setPickedKey}>
              <SelectTrigger className="w-full" aria-label={t("logs.session")}>
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {sessions.map((session) => (
                  <SelectItem key={sessionKey(session)} value={sessionKey(session)}>
                    <SessionSummary session={session} />
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <ol className="flex flex-col rounded-2xl border bg-card p-2" aria-live="polite">
              {shown.entries.map((entry, index) => (
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

      <ConfirmDialog
        open={confirmingClear}
        title={t("logs.clearTitle")}
        description={t("logs.clearDescription")}
        confirmLabel={t("logs.clearHistory")}
        destructive
        onConfirm={confirmClear}
        onOpenChange={setConfirmingClear}
      />
    </div>
  );
}
