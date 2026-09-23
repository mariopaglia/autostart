import { open } from "@tauri-apps/plugin-dialog";
import type { ExecutableChoice } from "@/lib/app-candidates";
import { commands } from "@/lib/tauri";

export async function pickExecutable(filterName: string): Promise<ExecutableChoice | null> {
  const path = await open({ filters: [{ name: filterName, extensions: ["exe"] }] });
  if (!path) return null;
  return { path, info: await commands.inspectExe(path) };
}
