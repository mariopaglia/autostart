import {
  CircleAlert,
  CircleCheck,
  CircleDashed,
  FastForward,
  Flag,
  Pin,
  Play,
  Plug,
  Rocket,
  ScanSearch,
  SkipForward,
  Zap,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { TimelineEntry } from "@/bindings/TimelineEntry";
import type { TimelineKind } from "@/bindings/TimelineKind";
import { translateError } from "@/lib/error-messages";
import { cn } from "@/lib/utils";

const KIND_ICONS: Record<TimelineKind, { icon: LucideIcon; className: string }> = {
  sessionStarted: { icon: Play, className: "text-sky-600 dark:text-sky-400" },
  launched: { icon: Rocket, className: "text-emerald-600 dark:text-emerald-400" },
  skipped: { icon: SkipForward, className: "text-muted-foreground" },
  notRunning: { icon: CircleDashed, className: "text-muted-foreground" },
  closedGracefully: { icon: CircleCheck, className: "text-emerald-600 dark:text-emerald-400" },
  forceClosed: { icon: Zap, className: "text-amber-600 dark:text-amber-400" },
  kept: { icon: Pin, className: "text-muted-foreground" },
  processNameLearned: { icon: ScanSearch, className: "text-sky-600 dark:text-sky-400" },
  simConnectReady: { icon: Plug, className: "text-violet-600 dark:text-violet-400" },
  simConnectWaitSkipped: { icon: FastForward, className: "text-muted-foreground" },
  error: { icon: CircleAlert, className: "text-destructive" },
  sessionEnded: { icon: Flag, className: "text-sky-600 dark:text-sky-400" },
};

export function TimelineRow({ entry, language }: { entry: TimelineEntry; language: string }) {
  const { t } = useTranslation();
  const { icon: Icon, className } = KIND_ICONS[entry.kind];

  return (
    <li className="flex items-start gap-3 rounded-xl px-3 py-2 hover:bg-muted/50">
      <time
        className="w-20 shrink-0 pt-0.5 font-mono text-xs text-muted-foreground tabular-nums"
        dateTime={new Date(entry.timestampMs).toISOString()}
      >
        {new Date(entry.timestampMs).toLocaleTimeString(language)}
      </time>
      <Icon className={cn("mt-0.5 size-4 shrink-0", className)} aria-hidden />
      <div className="flex min-w-0 flex-col gap-0.5 text-sm">
        <span>
          {entry.itemName && <span className="font-medium">{entry.itemName} · </span>}
          {t(`logs.kind.${entry.kind}`)}
          {entry.detail && <span className="font-mono text-xs"> · {entry.detail}</span>}
        </span>
        {entry.error && (
          <span className="text-xs text-destructive">
            {translateError(t, entry.error)}
            <span className="text-muted-foreground"> — {entry.error.message}</span>
          </span>
        )}
      </div>
    </li>
  );
}
