import type { Profile } from "@/bindings/Profile";

export function needsElevationWarning(profile: Profile | undefined, isElevated: boolean): boolean {
  if (isElevated || !profile) return false;
  return profile.items.some(
    (item) => item.type === "app" && item.enabled && item.runAsAdmin && item.onClose !== "keep",
  );
}
