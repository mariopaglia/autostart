import { FilePlus2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface DropOverlayProps {
  visible: boolean;
  profileName: string;
}

export function DropOverlay({ visible, profileName }: DropOverlayProps) {
  const { t } = useTranslation();

  return (
    <div
      aria-hidden={!visible}
      className={cn(
        "pointer-events-none absolute inset-3 z-40 flex flex-col items-center justify-center gap-3 rounded-2xl border-2 border-dashed border-primary bg-background/85 text-center backdrop-blur-sm transition-opacity duration-150 motion-reduce:transition-none",
        visible ? "opacity-100" : "opacity-0",
      )}
    >
      <FilePlus2 className="size-10 text-primary" />
      <p className="text-lg font-semibold">{t("fileDrop.overlayTitle")}</p>
      <p className="max-w-sm text-sm text-muted-foreground">
        {t("fileDrop.overlayDescription", { profile: profileName })}
      </p>
    </div>
  );
}
