import { useTranslation } from "react-i18next";
import type { ItemRuntime } from "@/bindings/ItemRuntime";
import type { ItemStatus } from "@/bindings/ItemStatus";
import { Badge } from "@/components/ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { translateError } from "@/lib/error-messages";
import { cn } from "@/lib/utils";

const STATUS_CLASSES: Record<ItemStatus, string> = {
  pending: "bg-muted text-muted-foreground",
  launching: "bg-sky-500/15 text-sky-700 dark:text-sky-400",
  running: "bg-emerald-500/15 text-emerald-700 dark:text-emerald-400",
  skipped: "bg-muted text-muted-foreground",
  closing: "bg-amber-500/15 text-amber-700 dark:text-amber-400",
  closed: "bg-muted text-muted-foreground",
  error: "bg-destructive/15 text-destructive",
};

export function ItemStatusBadge({ runtime }: { runtime: ItemRuntime }) {
  const { t } = useTranslation();
  const badge = (
    <Badge className={cn("border-transparent", STATUS_CLASSES[runtime.status])}>
      {t(`items.status.${runtime.status}`)}
    </Badge>
  );

  if (!runtime.error) return badge;
  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <span
          tabIndex={0}
          className="rounded-md outline-none focus-visible:ring-2 focus-visible:ring-ring"
        >
          {badge}
        </span>
      </TooltipTrigger>
      <TooltipContent className="flex max-w-xs flex-col gap-1 break-words">
        <span className="font-medium">{translateError(t, runtime.error)}</span>
        <span className="opacity-80">{runtime.error.message}</span>
      </TooltipContent>
    </Tooltip>
  );
}
