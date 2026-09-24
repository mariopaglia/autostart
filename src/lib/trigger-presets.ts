import type { Profile } from "@/bindings/Profile";
import type { Trigger } from "@/bindings/Trigger";

export const TRIGGER_PRESETS: readonly Trigger[] = [
  { processName: "FlightSimulator2024.exe", label: "MSFS 2024" },
  { processName: "FlightSimulator.exe", label: "MSFS 2020" },
  { processName: "X-Plane.exe", label: "X-Plane 12" },
];

export const MAX_TRIGGERS = 5;

export interface LaunchTargetPreset {
  source: "steam" | "microsoftStore";
  target: string;
}

const LAUNCH_TARGET_PRESETS: Readonly<Record<string, readonly LaunchTargetPreset[]>> = {
  "flightsimulator.exe": [
    { source: "steam", target: "steam://rungameid/1250410" },
    {
      source: "microsoftStore",
      target: "shell:AppsFolder\\Microsoft.FlightSimulator_8wekyb3d8bbwe!App",
    },
  ],
  "flightsimulator2024.exe": [
    { source: "steam", target: "steam://rungameid/2537590" },
    {
      source: "microsoftStore",
      target: "shell:AppsFolder\\Microsoft.Limitless_8wekyb3d8bbwe!App",
    },
  ],
};

/** Ready-made ways to start the known simulators; other simulators have none. */
export function launchTargetPresets(trigger: Trigger): readonly LaunchTargetPreset[] {
  return LAUNCH_TARGET_PRESETS[trigger.processName.toLowerCase()] ?? [];
}

export function withLaunchTarget(
  triggers: readonly Trigger[],
  processName: string,
  launchTarget: string | undefined,
): Trigger[] {
  return triggers.map((trigger) => {
    if (trigger.processName.toLowerCase() !== processName.toLowerCase()) return trigger;
    const withoutTarget: Trigger = { processName: trigger.processName, label: trigger.label };
    return launchTarget ? { ...withoutTarget, launchTarget } : withoutTarget;
  });
}

function sameProcess(first: Trigger, second: Trigger): boolean {
  return first.processName.toLowerCase() === second.processName.toLowerCase();
}

export function hasTrigger(triggers: readonly Trigger[], trigger: Trigger): boolean {
  return triggers.some((existing) => sameProcess(existing, trigger));
}

/** Ignores repeated simulators and keeps the list within the allowed size. */
export function addTrigger(triggers: readonly Trigger[], trigger: Trigger): Trigger[] {
  if (hasTrigger(triggers, trigger) || triggers.length >= MAX_TRIGGERS) return [...triggers];
  return [...triggers, trigger];
}

/** A profile always keeps at least one simulator. */
export function removeTrigger(triggers: readonly Trigger[], trigger: Trigger): Trigger[] {
  if (triggers.length <= 1) return [...triggers];
  return triggers.filter((existing) => !sameProcess(existing, trigger));
}

/** Items restricted to a removed simulator lose that restriction; with none left they apply to all. */
export function withTriggers(profile: Profile, triggers: Trigger[]): Profile {
  const kept = new Set(triggers.map((trigger) => trigger.processName.toLowerCase()));
  return {
    ...profile,
    triggers,
    items: profile.items.map((item) => ({
      ...item,
      onlyForTriggers: item.onlyForTriggers.filter((processName) =>
        kept.has(processName.toLowerCase()),
      ),
    })),
  };
}

/** Labels of the simulators an item is restricted to, in the profile's order. */
export function restrictionLabels(
  triggers: readonly Trigger[],
  onlyFor: readonly string[],
): string[] {
  const restricted = new Set(onlyFor.map((processName) => processName.toLowerCase()));
  return triggers
    .filter((trigger) => restricted.has(trigger.processName.toLowerCase()))
    .map((trigger) => trigger.label);
}

const SIMCONNECT_TRIGGERS: readonly string[] = ["flightsimulator.exe", "flightsimulator2024.exe"];

/** True when any of the profile's simulators is MSFS, the only one with SimConnect. */
export function supportsSimConnect(triggers: readonly Trigger[]): boolean {
  return triggers.some((trigger) =>
    SIMCONNECT_TRIGGERS.includes(trigger.processName.trim().toLowerCase()),
  );
}

export function triggerFromProcessName(processName: string): Trigger {
  const trimmed = processName.trim();
  const processNameWithExtension = /\.exe$/i.test(trimmed) ? trimmed : `${trimmed}.exe`;
  const preset = TRIGGER_PRESETS.find(
    (candidate) => candidate.processName.toLowerCase() === processNameWithExtension.toLowerCase(),
  );
  return (
    preset ?? {
      processName: processNameWithExtension,
      label: processNameWithExtension.replace(/\.exe$/i, ""),
    }
  );
}
