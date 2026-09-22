import { Download } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Spinner } from "@/components/ui/spinner";
import { useUpdatesStore } from "@/stores/updates-store";

export function UpdateDialog() {
  const { t } = useTranslation();
  const { status, update, downloadedBytes, totalBytes, install, dismiss } = useUpdatesStore();
  const installing = status === "installing";
  const open = update !== null && (status === "available" || installing);
  const progress = totalBytes
    ? Math.min(100, Math.round((downloadedBytes / totalBytes) * 100))
    : null;

  return (
    <Dialog
      open={open}
      onOpenChange={(nextOpen) => {
        if (!nextOpen && !installing) dismiss();
      }}
    >
      <DialogContent showCloseButton={!installing}>
        <DialogHeader>
          <DialogTitle>{t("updates.available", { version: update?.version ?? "" })}</DialogTitle>
          <DialogDescription>
            {t("updates.currentVersion", { version: update?.currentVersion ?? "" })}
          </DialogDescription>
        </DialogHeader>

        {update?.body && (
          <div className="max-h-60 overflow-y-auto rounded-xl border bg-muted/40 p-3 text-sm whitespace-pre-wrap">
            {update.body}
          </div>
        )}

        {installing && (
          <div className="flex flex-col gap-1.5" role="status">
            <div className="h-2 overflow-hidden rounded-full bg-muted">
              <div
                className="h-full bg-primary transition-[width]"
                style={{ width: `${String(progress ?? 100)}%` }}
              />
            </div>
            <span className="text-xs text-muted-foreground">
              {progress === null ? t("updates.installing") : t("updates.progress", { progress })}
            </span>
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" disabled={installing} onClick={dismiss}>
            {t("updates.later")}
          </Button>
          <Button disabled={installing} onClick={() => void install()}>
            {installing ? <Spinner /> : <Download />}
            {t("updates.install")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
