import { useState } from "react";
import { Check, ChevronsUpDown, Cpu, Plane } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { ProcessInfo } from "@/bindings/ProcessInfo";
import type { Trigger } from "@/bindings/Trigger";
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
import { TRIGGER_PRESETS, triggerFromProcessName } from "@/lib/trigger-presets";
import { processNameSchema } from "@/schemas/profile";

interface TriggerSelectorProps {
  trigger: Trigger;
  onChange: (trigger: Trigger) => void;
}

export function TriggerSelector({ trigger, onChange }: TriggerSelectorProps) {
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

  function choose(next: Trigger) {
    onChange(next);
    setOpen(false);
  }

  const customTrigger = triggerFromProcessName(search);
  const canUseCustom =
    search.trim() !== "" && processNameSchema.safeParse(customTrigger.processName).success;
  const isSelected = (processName: string) =>
    processName.toLowerCase() === trigger.processName.toLowerCase();

  return (
    <Popover open={open} onOpenChange={(next) => void handleOpenChange(next)}>
      <PopoverTrigger asChild>
        <Button
          variant="outline"
          className="max-w-64 justify-between"
          aria-label={t("trigger.change")}
        >
          <Plane />
          <span className="truncate">{trigger.label}</span>
          <span className="truncate text-xs text-muted-foreground">{trigger.processName}</span>
          <ChevronsUpDown className="opacity-50" />
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
                    choose(customTrigger);
                  }}
                >
                  {t("trigger.useCustom", { processName: customTrigger.processName })}
                </CommandItem>
              </CommandGroup>
            )}
            <CommandGroup heading={t("trigger.presets")}>
              {TRIGGER_PRESETS.map((preset) => (
                <CommandItem
                  key={preset.processName}
                  value={`${preset.label} ${preset.processName}`}
                  onSelect={() => {
                    choose(preset);
                  }}
                >
                  <Plane />
                  <span>{preset.label}</span>
                  <span className="text-xs text-muted-foreground">{preset.processName}</span>
                  {isSelected(preset.processName) && <Check className="ml-auto" />}
                </CommandItem>
              ))}
            </CommandGroup>
            <CommandGroup heading={t("trigger.running")}>
              {running.map((process) => (
                <CommandItem
                  key={process.name}
                  value={process.name}
                  onSelect={() => {
                    choose(triggerFromProcessName(process.name));
                  }}
                >
                  <Cpu />
                  <span className="truncate">{process.name}</span>
                  {isSelected(triggerFromProcessName(process.name).processName) && (
                    <Check className="ml-auto" />
                  )}
                </CommandItem>
              ))}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
