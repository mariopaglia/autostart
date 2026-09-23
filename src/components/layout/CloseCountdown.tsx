import { useEffect, useState } from "react";
import { Hand, Timer, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { secondsUntil } from "@/lib/countdown";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";

async function run(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (error) {
    notifyError(error);
  }
}

export function CloseCountdown({ closesAtMs }: { closesAtMs: number }) {
  const { t } = useTranslation();
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    const timer = window.setInterval(() => {
      setNow(Date.now());
    }, 1000);
    return () => {
      window.clearInterval(timer);
    };
  }, []);

  return (
    <div
      role="status"
      className="flex flex-wrap items-center gap-3 border-b border-amber-500/40 bg-amber-500/10 px-6 py-2 text-sm"
    >
      <Timer className="size-4 text-amber-600 dark:text-amber-400" aria-hidden />
      <span className="font-medium tabular-nums">
        {t("monitor.closingIn", { seconds: secondsUntil(closesAtMs, now) })}
      </span>
      <div className="ml-auto flex items-center gap-2">
        <Button size="sm" variant="outline" onClick={() => void run(commands.keepAppsOpen)}>
          <Hand />
          {t("monitor.keepApps")}
        </Button>
        <Button size="sm" variant="secondary" onClick={() => void run(commands.closeAppsNow)}>
          <X />
          {t("monitor.closeNow")}
        </Button>
      </div>
    </div>
  );
}
