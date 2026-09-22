import { AppWindow, Globe } from "lucide-react";
import type { LaunchItem } from "@/bindings/LaunchItem";
import { cn } from "@/lib/utils";

interface ItemIconProps {
  type: LaunchItem["type"];
  iconBase64?: string;
  className?: string;
}

export function ItemIcon({ type, iconBase64, className }: ItemIconProps) {
  return (
    <div
      className={cn(
        "flex size-12 shrink-0 items-center justify-center rounded-xl bg-muted text-muted-foreground",
        className,
      )}
    >
      {type === "url" && <Globe className="size-6" />}
      {type === "app" && !iconBase64 && <AppWindow className="size-6" />}
      {type === "app" && iconBase64 && (
        <img src={`data:image/png;base64,${iconBase64}`} alt="" className="size-8" />
      )}
    </div>
  );
}
