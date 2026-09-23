import { useSortable } from "@dnd-kit/sortable";
import { CSS } from "@dnd-kit/utilities";
import {
  GripVertical,
  Minimize2,
  MoreVertical,
  Pencil,
  Plug,
  RotateCcw,
  ShieldAlert,
  Trash2,
  TriangleAlert,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import type { LaunchItem } from "@/bindings/LaunchItem";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Switch } from "@/components/ui/switch";
import { shortenPath } from "@/lib/paths";
import { cn } from "@/lib/utils";
import { useItemRuntime } from "@/stores/monitor-store";
import { ItemIcon } from "./ItemIcon";
import { ItemStatusBadge } from "./ItemStatusBadge";

interface ItemCardProps {
  profileId: string;
  item: LaunchItem;
  executableMissing: boolean;
  onToggle: (enabled: boolean) => void;
  onEdit: () => void;
  onRemove: () => void;
}

export function ItemCard({
  profileId,
  item,
  executableMissing,
  onToggle,
  onEdit,
  onRemove,
}: ItemCardProps) {
  const { t } = useTranslation();
  const runtime = useItemRuntime(profileId, item.id);
  const {
    attributes,
    listeners,
    setNodeRef,
    setActivatorNodeRef,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: item.id });

  const subtitle = item.type === "app" ? shortenPath(item.exePath) : item.url;

  return (
    <li
      ref={setNodeRef}
      style={{ transform: CSS.Transform.toString(transform), transition }}
      className={cn(
        "group flex items-center gap-3 rounded-2xl border bg-card p-3 shadow-xs transition-[opacity,box-shadow,border-color] animate-in fade-in duration-300",
        "hover:border-primary/40 hover:shadow-md motion-reduce:transition-none",
        !item.enabled && "opacity-50",
        isDragging && "z-10 shadow-lg ring-2 ring-primary/40",
      )}
    >
      <button
        ref={setActivatorNodeRef}
        type="button"
        className="cursor-grab rounded-md p-1 text-muted-foreground outline-none hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring active:cursor-grabbing"
        aria-label={t("items.dragHandle")}
        {...attributes}
        {...listeners}
      >
        <GripVertical className="size-4" />
      </button>

      <button
        type="button"
        className="flex min-w-0 flex-1 items-center gap-3 rounded-lg text-left outline-none focus-visible:ring-2 focus-visible:ring-ring"
        onClick={onEdit}
      >
        <ItemIcon type={item.type} iconBase64={item.type === "app" ? item.iconBase64 : undefined} />
        <span className="flex min-w-0 flex-col gap-1">
          <span className="truncate font-medium">{item.name}</span>
          <span className="truncate text-xs text-muted-foreground" title={subtitle}>
            {subtitle}
          </span>
          <span className="flex flex-wrap items-center gap-1">
            {item.type === "app" && item.runAsAdmin && (
              <Badge variant="outline" className="gap-1">
                <ShieldAlert className="size-3" />
                {t("items.admin")}
              </Badge>
            )}
            {item.type === "app" && item.startMinimized && (
              <Badge variant="outline" className="gap-1">
                <Minimize2 className="size-3" />
                {t("items.minimized")}
              </Badge>
            )}
            {item.type === "app" && item.waitForSimConnect && (
              <Badge variant="outline" className="gap-1">
                <Plug className="size-3" />
                {t("items.simConnect")}
              </Badge>
            )}
            {item.type === "app" && item.restartOnCrash && (
              <Badge variant="outline" className="gap-1">
                <RotateCcw className="size-3" />
                {t("items.restartOnCrash")}
              </Badge>
            )}
            {item.delayMs > 0 && (
              <Badge variant="outline">
                {t("items.delay", { seconds: (item.delayMs / 1000).toLocaleString() })}
              </Badge>
            )}
            {item.type === "app" && (
              <Badge variant="outline">{t(`items.onClose.${item.onClose}`)}</Badge>
            )}
            {executableMissing && (
              <Badge variant="destructive" className="gap-1">
                <TriangleAlert className="size-3" />
                {t("items.missingExe")}
              </Badge>
            )}
          </span>
        </span>
      </button>

      {runtime && <ItemStatusBadge runtime={runtime} />}

      <Switch checked={item.enabled} onCheckedChange={onToggle} aria-label={t("items.toggle")} />

      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button variant="ghost" size="icon-sm" aria-label={t("items.actions")}>
            <MoreVertical />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem onSelect={onEdit}>
            <Pencil />
            {t("items.edit")}
          </DropdownMenuItem>
          <DropdownMenuItem variant="destructive" onSelect={onRemove}>
            <Trash2 />
            {t("items.remove")}
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </li>
  );
}
