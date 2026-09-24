import { open } from "@tauri-apps/plugin-dialog";
import type { ExecutableChoice } from "@/lib/app-candidates";
import { commands } from "@/lib/tauri";

export async function pickExecutablePath(filterName: string): Promise<string | null> {
  return open({ filters: [{ name: filterName, extensions: ["exe"] }] });
}

export async function pickExecutable(filterName: string): Promise<ExecutableChoice | null> {
  const path = await pickExecutablePath(filterName);
  if (!path) return null;
  return { path, info: await commands.inspectExe(path) };
}
