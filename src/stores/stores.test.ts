import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Profile } from "@/bindings/Profile";
import type { Settings } from "@/bindings/Settings";
import { createProfile } from "@/lib/profile-factory";
import { commands } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";
import { useProfilesStore } from "@/stores/profiles-store";
import { useSettingsStore } from "@/stores/settings-store";

vi.mock("@/lib/tauri", () => ({
  commands: {
    getProfiles: vi.fn(),
    saveProfile: vi.fn(),
    deleteProfile: vi.fn(),
    getSettings: vi.fn(),
    saveSettings: vi.fn(),
    setActiveProfile: vi.fn(),
  },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

vi.mock("sonner", () => ({ toast: { error: vi.fn(), success: vi.fn() } }));

const mocked = vi.mocked(commands);

function settingsFor(activeProfileId: string | null): Settings {
  return {
    activeProfileId,
    startWithWindows: false,
    startMinimized: true,
    gracefulTimeoutMs: 5000,
    closeOnlyIfLaunchedByApp: true,
    theme: "dark",
    language: "pt-BR",
    onboardingCompleted: true,
    checkUpdatesOnStartup: true,
  };
}

describe("profiles store", () => {
  let first: Profile;
  let second: Profile;

  beforeEach(async () => {
    vi.clearAllMocks();
    first = createProfile("First");
    second = createProfile("Second");
    mocked.getSettings.mockResolvedValue(settingsFor(second.id));
    mocked.getProfiles.mockResolvedValue([first, second]);
    await useSettingsStore.getState().load();
    await useProfilesStore.getState().load();
  });

  it("selects the active profile after loading", () => {
    expect(useProfilesStore.getState().selectedId).toBe(second.id);
  });

  it("keeps an optimistic save when the backend accepts it", async () => {
    mocked.saveProfile.mockResolvedValue({ ...first, name: "Renamed" });

    await useProfilesStore.getState().save({ ...first, name: "Renamed" });

    expect(useProfilesStore.getState().profiles[0]?.name).toBe("Renamed");
  });

  it("rolls back an optimistic save when the backend rejects it", async () => {
    mocked.saveProfile.mockRejectedValue({ kind: "validation", message: "name" });

    const saved = await useProfilesStore.getState().save({ ...first, name: "" });

    expect(saved).toBe(false);
    expect(useProfilesStore.getState().profiles[0]?.name).toBe("First");
  });

  it("applies a profile changed by the backend without saving it again", () => {
    useProfilesStore.getState().replace({ ...first, name: "Learned" });

    expect(useProfilesStore.getState().profiles[0]?.name).toBe("Learned");
    expect(mocked.saveProfile).not.toHaveBeenCalled();
  });

  it("moves selection to the new active profile after deleting the selected one", async () => {
    mocked.deleteProfile.mockResolvedValue(settingsFor(first.id));

    await useProfilesStore.getState().remove(second.id);

    expect(useProfilesStore.getState().profiles).toHaveLength(1);
    expect(useProfilesStore.getState().selectedId).toBe(first.id);
    expect(useSettingsStore.getState().settings?.activeProfileId).toBe(first.id);
  });
});

describe("monitor store", () => {
  it("replaces the snapshot", () => {
    useMonitorStore.getState().setSnapshot({
      state: "simRunning",
      sessionProfileId: "p",
      isTestSession: false,
      items: [],
    });

    expect(useMonitorStore.getState().snapshot.state).toBe("simRunning");
  });
});
