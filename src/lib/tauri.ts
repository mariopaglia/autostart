import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { ErrorKind } from "@/bindings/ErrorKind";
import type { ErrorPayload } from "@/bindings/ErrorPayload";
import type { ExeInfo } from "@/bindings/ExeInfo";
import type { ItemRuntime } from "@/bindings/ItemRuntime";
import type { MonitorSnapshot } from "@/bindings/MonitorSnapshot";
import type { ProcessInfo } from "@/bindings/ProcessInfo";
import type { Profile } from "@/bindings/Profile";
import type { SessionLog } from "@/bindings/SessionLog";
import type { Settings } from "@/bindings/Settings";
import type { TimelineEntry } from "@/bindings/TimelineEntry";

export const commands = {
  getProfiles: () => invoke<Profile[]>("get_profiles"),
  saveProfile: (profile: Profile) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (profileId: string) => invoke<Settings>("delete_profile", { profileId }),
  setActiveProfile: (profileId: string) => invoke<Settings>("set_active_profile", { profileId }),
  getSettings: () => invoke<Settings>("get_settings"),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  inspectExe: (path: string) => invoke<ExeInfo>("inspect_exe", { path }),
  findMissingExecutables: (paths: string[]) =>
    invoke<string[]>("find_missing_executables", { paths }),
  listRunningProcesses: () => invoke<ProcessInfo[]>("list_running_processes"),
  isElevated: () => invoke<boolean>("is_elevated"),
  importProfile: (path: string) => invoke<unknown>("import_profile", { path }),
  exportProfile: (profileId: string, path: string) =>
    invoke<null>("export_profile", { profileId, path }),
  getMonitorState: () => invoke<MonitorSnapshot>("get_monitor_state"),
  getSessionLog: () => invoke<SessionLog | null>("get_session_log"),
  pauseMonitor: () => invoke<null>("pause_monitor"),
  resumeMonitor: () => invoke<null>("resume_monitor"),
  testLaunch: (profileId: string) => invoke<null>("test_launch", { profileId }),
  testClose: (profileId: string) => invoke<null>("test_close", { profileId }),
};

export const monitorEvents = {
  onState: (handler: (snapshot: MonitorSnapshot) => void): Promise<UnlistenFn> =>
    listen<MonitorSnapshot>("monitor://state", (event) => {
      handler(event.payload);
    }),
  onItemStatus: (handler: (runtime: ItemRuntime) => void): Promise<UnlistenFn> =>
    listen<ItemRuntime>("monitor://item-status", (event) => {
      handler(event.payload);
    }),
  onLog: (handler: (entry: TimelineEntry) => void): Promise<UnlistenFn> =>
    listen<TimelineEntry>("monitor://log", (event) => {
      handler(event.payload);
    }),
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
