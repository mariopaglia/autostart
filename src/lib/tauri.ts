import { invoke } from "@tauri-apps/api/core";
import type { ErrorKind } from "@/bindings/ErrorKind";
import type { ErrorPayload } from "@/bindings/ErrorPayload";
import type { ExeInfo } from "@/bindings/ExeInfo";
import type { ItemRuntime } from "@/bindings/ItemRuntime";
import type { ProcessInfo } from "@/bindings/ProcessInfo";
import type { Profile } from "@/bindings/Profile";
import type { Settings } from "@/bindings/Settings";

export const commands = {
  getProfiles: () => invoke<Profile[]>("get_profiles"),
  saveProfile: (profile: Profile) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (profileId: string) => invoke<Settings>("delete_profile", { profileId }),
  setActiveProfile: (profileId: string) => invoke<Settings>("set_active_profile", { profileId }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  inspectExe: (path: string) => invoke<ExeInfo>("inspect_exe", { path }),
  listRunningProcesses: () => invoke<ProcessInfo[]>("list_running_processes"),
  isElevated: () => invoke<boolean>("is_elevated"),
  importProfile: (path: string) => invoke<unknown>("import_profile", { path }),
  exportProfile: (profileId: string, path: string) =>
    invoke<null>("export_profile", { profileId, path }),
  testLaunch: (profileId: string) => invoke<ItemRuntime[]>("test_launch", { profileId }),
  testClose: (profileId: string) => invoke<ItemRuntime[]>("test_close", { profileId }),
};

export function isAppError(error: unknown): error is ErrorPayload {
  return (
    typeof error === "object" &&
    error !== null &&
    "kind" in error &&
    "message" in error &&
    typeof error.message === "string"
  );
}

export function errorKindOf(error: unknown): ErrorKind {
  return isAppError(error) ? error.kind : "internal";
}
