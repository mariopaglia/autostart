import type { LaunchItem } from "@/bindings/LaunchItem";
import type { Profile } from "@/bindings/Profile";
import { TRIGGER_PRESETS } from "@/lib/trigger-presets";

const [DEFAULT_TRIGGER] = TRIGGER_PRESETS;

export function createProfile(name: string): Profile {
  return {
    id: crypto.randomUUID(),
    name: name.trim(),
    trigger: DEFAULT_TRIGGER ?? { processName: "FlightSimulator2024.exe", label: "MSFS 2024" },
    items: [],
    enabled: true,
  };
}

export function withFreshIds(profile: Profile): Profile {
  return {
    ...profile,
    id: crypto.randomUUID(),
    items: profile.items.map((item): LaunchItem => ({ ...item, id: crypto.randomUUID() })),
  };
}

export function duplicateProfile(profile: Profile, copySuffix: string): Profile {
  return { ...withFreshIds(profile), name: `${profile.name}${copySuffix}`.slice(0, 80) };
}
