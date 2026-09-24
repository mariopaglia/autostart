import { useState } from "react";
import { Check, Cpu, Plane, Plus, Rocket, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ProcessInfo } from "@/bindings/ProcessInfo";
import type { Trigger } from "@/bindings/Trigger";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import {
  addTrigger,
  hasTrigger,
  MAX_TRIGGERS,
  removeTrigger,
  TRIGGER_PRESETS,
  triggerFromProcessName,
  withLaunchTarget,
} from "@/lib/trigger-presets";
import { processNameSchema } from "@/schemas/profile";
import { LaunchTargetEditor } from "./LaunchTargetEditor";

interface TriggerSelectorProps {
  triggers: Trigger[];
  onChange: (triggers: Trigger[]) => void;
}

export function TriggerSelector({ triggers, onChange }: TriggerSelectorProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [search, setSearch] = useState("");
  const [running, setRunning] = useState<ProcessInfo[]>([]);

  async function handleOpenChange(nextOpen: boolean) {
    setOpen(nextOpen);
    setSearch("");
    if (!nextOpen) return;
    try {
      setRunning(await commands.listRunningProcesses());
    } catch (error) {
      notifyError(error);
    }
  }

  function add(trigger: Trigger) {
    onChange(addTrigger(triggers, trigger));
    setOpen(false);
  }

  const customTrigger = triggerFromProcessName(search);
  const canUseCustom =
    search.trim() !== "" &&
    processNameSchema.safeParse(customTrigger.processName).success &&
    !hasTrigger(triggers, customTrigger);
  const isFull = triggers.length >= MAX_TRIGGERS;

  function option(trigger: Trigger, icon: React.ReactNode, key: string, value: string) {
    const added = hasTrigger(triggers, trigger);
    return (
      <CommandItem
        key={key}
        value={value}
        disabled={added}
        onSelect={() => {
          add(trigger);
        }}
      >
        {icon}
        <span className="truncate">{trigger.label}</span>
        <span className="truncate text-xs text-muted-foreground">{trigger.processName}</span>
        {added && <Check className="ml-auto" />}
      </CommandItem>
    );
  }

  return (
    <div
      role="group"
      className="flex flex-wrap items-center gap-1.5"
      aria-label={t("trigger.listLabel")}
    >
      {triggers.map((trigger) => (
        <Badge
          key={trigger.processName}
          variant="secondary"
          className="h-7 gap-1.5 pr-1 pl-2"
          title={trigger.processName}
        >
          <Popover>
            <PopoverTrigger asChild>
              <button
                type="button"
                className="flex items-center gap-1.5 rounded-sm hover:underline"
                aria-label={t("trigger.launchTarget.edit", { label: trigger.label })}
              >
                <Plane />
                <span className="max-w-40 truncate">{trigger.label}</span>
                {trigger.launchTarget && (
                  <Rocket
                    className="size-3 text-sky-600 dark:text-sky-400"
                    aria-label={t("trigger.launchTarget.set")}
                  />
                )}
              </button>
            </PopoverTrigger>
            <PopoverContent className="w-96" align="start">
              <LaunchTargetEditor
                trigger={trigger}
                onChange={(launchTarget) => {
                  onChange(withLaunchTarget(triggers, trigger.processName, launchTarget));
                }}
              />
            </PopoverContent>
          </Popover>
          <button
            type="button"
            className="rounded-sm p-0.5 hover:bg-muted-foreground/20 disabled:opacity-40"
            aria-label={t("trigger.remove", { label: trigger.label })}
            disabled={triggers.length === 1}
            onClick={() => {
              onChange(removeTrigger(triggers, trigger));
            }}
          >
            <X className="size-3" />
          </button>
        </Badge>
      ))}

      <Popover open={open} onOpenChange={(next) => void handleOpenChange(next)}>
        <PopoverTrigger asChild>
          <Button variant="ghost" size="sm" disabled={isFull} aria-label={t("trigger.add")}>
            <Plus />
            {t("trigger.add")}
          </Button>
        </PopoverTrigger>
        <PopoverContent className="w-80 p-0" align="start">
          <Command>
            <CommandInput
              placeholder={t("trigger.search")}
              value={search}
              onValueChange={setSearch}
            />
            <CommandList>
              <CommandEmpty>{t("trigger.empty")}</CommandEmpty>
              {canUseCustom && (
                <CommandGroup>
                  <CommandItem
                    value={`custom-${customTrigger.processName}`}
                    onSelect={() => {
                      add(customTrigger);
                    }}
                  >
                    {t("trigger.useCustom", { processName: customTrigger.processName })}
                  </CommandItem>
                </CommandGroup>
              )}
              <CommandGroup heading={t("trigger.presets")}>
                {TRIGGER_PRESETS.map((preset) =>
                  option(
                    preset,
                    <Plane />,
                    preset.processName,
                    `${preset.label} ${preset.processName}`,
                  ),
                )}
              </CommandGroup>
              <CommandGroup heading={t("trigger.running")}>
                {running.map((process) =>
                  option(triggerFromProcessName(process.name), <Cpu />, process.name, process.name),
                )}
              </CommandGroup>
            </CommandList>
          </Command>
        </PopoverContent>
      </Popover>
    </div>
  );
}
