import type { Trigger } from "@/bindings/Trigger";

export const TRIGGER_PRESETS: readonly Trigger[] = [
  { processName: "FlightSimulator2024.exe", label: "MSFS 2024" },
  { processName: "FlightSimulator.exe", label: "MSFS 2020" },
  { processName: "X-Plane.exe", label: "X-Plane 12" },
];

export const MAX_TRIGGERS = 5;

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
