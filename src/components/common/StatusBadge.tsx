import { useTranslation } from "react-i18next";
import type { MonitorState } from "@/bindings/MonitorState";
import { cn } from "@/lib/utils";

const DOT_CLASSES: Record<MonitorState, string> = {
  idle: "bg-muted-foreground",
  simStarting: "bg-sky-500 animate-pulse motion-reduce:animate-none",
  simRunning: "bg-emerald-500 animate-pulse motion-reduce:animate-none",
  closePending: "bg-amber-500",
  closing: "bg-amber-500 animate-pulse motion-reduce:animate-none",
  paused: "bg-sky-500",
};

interface StatusBadgeProps {
  state: MonitorState;
  startingSimulator?: string | null;
}

export function StatusBadge({ state, startingSimulator }: StatusBadgeProps) {
  const { t } = useTranslation();

  return (
    <span
      role="status"
      className="inline-flex items-center gap-2 rounded-full border bg-card px-3 py-1 text-xs font-medium"
    >
      <span className={cn("size-2 rounded-full", DOT_CLASSES[state])} aria-hidden />
      {state === "simStarting"
        ? t("monitor.state.simStarting", { simulator: startingSimulator ?? "" })
        : t(`monitor.state.${state}`)}
    </span>
  );
}
