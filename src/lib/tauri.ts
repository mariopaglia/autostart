import { getVersion } from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { warn } from "@tauri-apps/plugin-log";
import { openUrl } from "@tauri-apps/plugin-opener";
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
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
import { appCandidatesSchema, dropResolutionSchema } from "@/schemas/app-candidate";

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
  listInstalledApps: () =>
    invoke<unknown>("list_installed_apps").then((apps) => appCandidatesSchema.parse(apps)),
  listOpenApps: () =>
    invoke<unknown>("list_open_apps").then((apps) => appCandidatesSchema.parse(apps)),
  resolveDroppedPaths: (paths: string[]) =>
    invoke<unknown>("resolve_dropped_paths", { paths }).then((resolution) =>
      dropResolutionSchema.parse(resolution),
    ),
  isElevated: () => invoke<boolean>("is_elevated"),
  relaunchAsAdmin: () => invoke<null>("relaunch_as_admin"),
  openLogDir: () => invoke<null>("open_log_dir"),
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

export const system = {
  getAppVersion: () => getVersion(),
  openUrl: (url: string) => openUrl(url),
  relaunch: () => relaunch(),
  logWarning: (message: string) => warn(message),
};

export type { Update };

export const updater = {
  check: () => check(),
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

export const profileEvents = {
  onChanged: (handler: (profile: Profile) => void): Promise<UnlistenFn> =>
    listen<Profile>("profiles://changed", (event) => {
      handler(event.payload);
    }),
};

export const settingsEvents = {
  onChanged: (handler: (settings: Settings) => void): Promise<UnlistenFn> =>
    listen<Settings>("settings://changed", (event) => {
      handler(event.payload);
    }),
};

export type FileDropEvent =
  { kind: "enter" } | { kind: "leave" } | { kind: "drop"; paths: string[] };

export const windowEvents = {
  onFileDrop: (handler: (event: FileDropEvent) => void): Promise<UnlistenFn> =>
    getCurrentWebview().onDragDropEvent(({ payload }) => {
      if (payload.type === "enter") handler({ kind: "enter" });
      if (payload.type === "leave") handler({ kind: "leave" });
      if (payload.type === "drop") handler({ kind: "drop", paths: payload.paths });
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
