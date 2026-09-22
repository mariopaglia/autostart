import { useTranslation } from "react-i18next";
import type { ItemRuntime } from "@/bindings/ItemRuntime";
import type { ItemStatus } from "@/bindings/ItemStatus";
import { Badge } from "@/components/ui/badge";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { cn } from "@/lib/utils";

const STATUS_CLASSES: Record<ItemStatus, string> = {
  pending: "bg-muted text-muted-foreground",
  launching: "bg-sky-500/15 text-sky-500",
  running: "bg-emerald-500/15 text-emerald-500",
  skipped: "bg-muted text-muted-foreground",
  closing: "bg-amber-500/15 text-amber-500",
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

  if (!runtime.message) return badge;
  return (
    <Tooltip>
      <TooltipTrigger asChild>{badge}</TooltipTrigger>
      <TooltipContent className="max-w-xs break-words">{runtime.message}</TooltipContent>
    </Tooltip>
  );
}
