import { useState } from "react";
import { ChevronDown, Pause, Plane, Play, PlayCircle, StopCircle, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Profile } from "@/bindings/Profile";
import { ConfirmDialog } from "@/components/common/ConfirmDialog";
import { StatusBadge } from "@/components/common/StatusBadge";
import { TriggerSelector } from "@/components/profiles/TriggerSelector";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Switch } from "@/components/ui/switch";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import { withTriggers } from "@/lib/trigger-presets";
import { useHasSession, useMonitorStore } from "@/stores/monitor-store";
import { useProfilesStore } from "@/stores/profiles-store";

async function run(action: () => Promise<unknown>) {
  try {
    await action();
  } catch (error) {
    notifyError(error);
  }
}

export function TopBar({ profile }: { profile: Profile }) {
  const { t } = useTranslation();
  const save = useProfilesStore((state) => state.save);
  const setEnabled = useProfilesStore((state) => state.setEnabled);
  const snapshot = useMonitorStore((state) => state.snapshot);
  const hasSession = useHasSession();
  const [confirmingClose, setConfirmingClose] = useState(false);

  const paused = snapshot.state === "paused";
  const hasPreviousTestLaunch =
    snapshot.isTestSession &&
    snapshot.sessionProfileId === profile.id &&
    snapshot.items.some((runtime) => runtime.launchedByApp);

  function testClose() {
    if (hasPreviousTestLaunch) {
      void run(() => commands.testClose(profile.id));
    } else {
      setConfirmingClose(true);
    }
  }

  function startFlight(triggerProcessName: string) {
    void run(() => commands.startFlight(profile.id, triggerProcessName));
  }

  const startTargets = profile.triggers.filter((trigger) => trigger.launchTarget);
  const canStartFlight = snapshot.state === "idle";
  const [onlyTarget] = startTargets;

  const flightButton =
    snapshot.state === "simStarting" ? (
      <Button variant="outline" onClick={() => void run(commands.cancelFlightStart)}>
        <X />
        {t("monitor.cancelStart")}
      </Button>
    ) : startTargets.length === 1 && onlyTarget ? (
      <Button
        disabled={!canStartFlight}
        onClick={() => {
          startFlight(onlyTarget.processName);
        }}
      >
        <Plane />
        {t("monitor.startFlight")}
      </Button>
    ) : startTargets.length > 1 ? (
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <Button disabled={!canStartFlight}>
            <Plane />
            {t("monitor.startFlight")}
            <ChevronDown />
          </Button>
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          {startTargets.map((trigger) => (
            <DropdownMenuItem
              key={trigger.processName}
              onSelect={() => {
                startFlight(trigger.processName);
              }}
            >
              {trigger.label}
            </DropdownMenuItem>
          ))}
        </DropdownMenuContent>
      </DropdownMenu>
    ) : null;

  const testButtons = (
    <div className="flex items-center gap-2">
      <Button
        variant="secondary"
        disabled={hasSession}
        onClick={() => void run(() => commands.testLaunch(profile.id))}
      >
        <PlayCircle />
        {t("monitor.testLaunch")}
      </Button>
      <Button variant="outline" disabled={hasSession} onClick={testClose}>
        <StopCircle />
        {t("monitor.testClose")}
      </Button>
    </div>
  );

  return (
    <header className="flex flex-wrap items-center justify-between gap-3 border-b px-6 py-4">
      <div className="flex min-w-0 items-center gap-3">
        <h1 className="truncate text-xl font-semibold tracking-tight">{profile.name}</h1>
        <Switch
          checked={profile.enabled}
          aria-label={t("profiles.enabledToggle")}
          onCheckedChange={(enabled) => void setEnabled(profile.id, enabled)}
        />
        <TriggerSelector
          triggers={profile.triggers}
          onChange={(triggers) => void save(withTriggers(profile, triggers))}
        />
      </div>

      <div className="flex items-center gap-2">
        <StatusBadge state={snapshot.state} startingSimulator={snapshot.startingSimulator} />
        {flightButton}
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant="ghost"
              size="icon"
              aria-label={paused ? t("monitor.resume") : t("monitor.pause")}
              onClick={() => void run(paused ? commands.resumeMonitor : commands.pauseMonitor)}
            >
              {paused ? <Play /> : <Pause />}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{paused ? t("monitor.resume") : t("monitor.pause")}</TooltipContent>
        </Tooltip>
        {hasSession ? (
          <Tooltip>
            <TooltipTrigger asChild>
              <span tabIndex={0}>{testButtons}</span>
            </TooltipTrigger>
            <TooltipContent>{t("monitor.testsDisabled")}</TooltipContent>
          </Tooltip>
        ) : (
          testButtons
        )}
      </div>

      <ConfirmDialog
        open={confirmingClose}
        title={t("monitor.confirmTestClose.title")}
        description={t("monitor.confirmTestClose.description")}
        confirmLabel={t("monitor.confirmTestClose.confirm")}
        onConfirm={() => {
          setConfirmingClose(false);
          void run(() => commands.testClose(profile.id));
        }}
        onOpenChange={setConfirmingClose}
      />
    </header>
  );
}
