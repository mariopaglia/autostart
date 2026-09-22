import type { Trigger } from "@/bindings/Trigger";

export const TRIGGER_PRESETS: readonly Trigger[] = [
  { processName: "FlightSimulator2024.exe", label: "MSFS 2024" },
  { processName: "FlightSimulator.exe", label: "MSFS 2020" },
  { processName: "X-Plane.exe", label: "X-Plane 12" },
];

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
