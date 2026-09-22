import { useState } from "react";
import { open, save } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { commands } from "@/lib/tauri";

type CommandRunner = () => Promise<unknown>;

// Temporary screen to exercise every backend command during phase 1; replaced by the real UI in phase 2.
export function DebugScreen() {
  const [profileId, setProfileId] = useState("");
  const [output, setOutput] = useState("");

  async function run(label: string, runner: CommandRunner) {
    try {
      const result = await runner();
      setOutput(`${label} ✔\n${JSON.stringify(result, null, 2)}`);
    } catch (error) {
      setOutput(`${label} ✘\n${JSON.stringify(error, null, 2)}`);
    }
  }

  async function loadFirstProfile() {
    const profiles = await commands.getProfiles();
    setProfileId(profiles[0]?.id ?? "");
    return profiles;
  }

  async function inspectPickedExe() {
    const path = await open({ filters: [{ name: "Executable", extensions: ["exe"] }] });
    return path ? commands.inspectExe(path) : "cancelled";
  }

  async function addPickedExeToProfile() {
    const path = await open({ filters: [{ name: "Executable", extensions: ["exe"] }] });
    if (!path) return "cancelled";
    const info = await commands.inspectExe(path);
    const profiles = await commands.getProfiles();
    const profile = profiles.find((candidate) => candidate.id === profileId);
    if (!profile) return "load a profile first";
    return commands.saveProfile({
      ...profile,
      items: [
        ...profile.items,
        {
          type: "app",
          id: crypto.randomUUID(),
          name: info.productName,
          exePath: path,
          processName: info.processName,
          iconBase64: info.iconBase64,
          delayMs: 800,
          runAsAdmin: false,
          onClose: "graceful",
          enabled: true,
        },
      ],
    });
  }

  async function exportActive() {
    const path = await save({ defaultPath: "profile.json" });
    return path ? commands.exportProfile(profileId, path) : "cancelled";
  }

  async function importFile() {
    const path = await open({ filters: [{ name: "Profile", extensions: ["json"] }] });
    return path ? commands.importProfile(path) : "cancelled";
  }

  const actions: [string, CommandRunner][] = [
    ["get_profiles", loadFirstProfile],
    ["get_settings", commands.getSettings],
    ["is_elevated", commands.isElevated],
    ["list_running_processes", commands.listRunningProcesses],
    ["inspect_exe", inspectPickedExe],
    ["add exe to profile", addPickedExeToProfile],
    ["test_launch", () => commands.testLaunch(profileId)],
    ["test_close", () => commands.testClose(profileId)],
    ["export_profile", exportActive],
    ["import_profile", importFile],
  ];

  return (
    <main className="flex h-screen flex-col gap-4 bg-background p-6 text-foreground">
      <h1 className="text-xl font-semibold">AutoStart · debug</h1>
      <Input
        placeholder="profileId"
        value={profileId}
        onChange={(event) => {
          setProfileId(event.target.value);
        }}
      />
      <div className="flex flex-wrap gap-2">
        {actions.map(([label, runner]) => (
          <Button key={label} variant="secondary" onClick={() => void run(label, runner)}>
            {label}
          </Button>
        ))}
      </div>
      <pre className="flex-1 overflow-auto rounded-lg bg-muted p-4 text-xs">{output}</pre>
    </main>
  );
}
