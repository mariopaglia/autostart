import { useTranslation } from "react-i18next";
import type { MonitorState } from "@/bindings/MonitorState";
import { cn } from "@/lib/utils";

const DOT_CLASSES: Record<MonitorState, string> = {
  idle: "bg-muted-foreground",
  simRunning: "bg-emerald-500 animate-pulse motion-reduce:animate-none",
  closing: "bg-amber-500 animate-pulse motion-reduce:animate-none",
  paused: "bg-sky-500",
};

export function StatusBadge({ state }: { state: MonitorState }) {
  const { t } = useTranslation();

  return (
    <span
      role="status"
      className="inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1 text-xs font-medium"
    >
      <span className={cn("size-2 rounded-full", DOT_CLASSES[state])} aria-hidden />
      {t(`monitor.state.${state}`)}
    </span>
  );
}
