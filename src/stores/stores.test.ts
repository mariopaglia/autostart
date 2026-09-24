import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Profile } from "@/bindings/Profile";
import { createProfile } from "@/lib/profile-factory";
import { commands } from "@/lib/tauri";
import { useMonitorStore } from "@/stores/monitor-store";
import { useProfilesStore } from "@/stores/profiles-store";

vi.mock("@/lib/tauri", () => ({
  commands: {
    getProfiles: vi.fn(),
    saveProfile: vi.fn(),
    setProfileEnabled: vi.fn(),
    deleteProfile: vi.fn(),
  },
  errorKindOf: () => "internal",
  isAppError: () => false,
}));

const toast = vi.hoisted(() => ({ error: vi.fn(), success: vi.fn(), info: vi.fn() }));
vi.mock("sonner", () => ({ toast }));

const mocked = vi.mocked(commands);

describe("profiles store", () => {
  let first: Profile;
  let second: Profile;

  beforeEach(async () => {
    vi.clearAllMocks();
    first = { ...createProfile("First"), enabled: false };
    second = createProfile("Second");
    mocked.getProfiles.mockResolvedValue([first, second]);
    await useProfilesStore.getState().load();
  });

  it("selects the first enabled profile after loading", () => {
    expect(useProfilesStore.getState().selectedId).toBe(second.id);
  });

  it("keeps the profiles returned by the backend after saving", async () => {
    const renamed = { ...first, name: "Renamed" };
    mocked.saveProfile.mockResolvedValue({ profiles: [renamed, second], disabledProfileIds: [] });

    await useProfilesStore.getState().save(renamed);

    expect(useProfilesStore.getState().profiles[0]?.name).toBe("Renamed");
    expect(toast.info).not.toHaveBeenCalled();
  });

  it("rolls back an optimistic save when the backend rejects it", async () => {
    mocked.saveProfile.mockRejectedValue({ kind: "validation", message: "name" });

    const saved = await useProfilesStore.getState().save({ ...first, name: "" });

    expect(saved).toBe(false);
    expect(useProfilesStore.getState().profiles[0]?.name).toBe("First");
  });

  it("names the profiles disabled when enabling one with the same simulator", async () => {
    mocked.setProfileEnabled.mockResolvedValue({
      profiles: [
        { ...first, enabled: true },
        { ...second, enabled: false },
      ],
      disabledProfileIds: [second.id],
    });

    await useProfilesStore.getState().setEnabled(first.id, true);

    expect(useProfilesStore.getState().profiles.map((profile) => profile.enabled)).toEqual([
      true,
      false,
    ]);
    expect(toast.info).toHaveBeenCalledWith(expect.stringContaining("Second"));
  });

  it("tells the pilot when a new profile was created disabled", async () => {
    const created = createProfile("Third");
    mocked.saveProfile.mockResolvedValue({
      profiles: [first, second, { ...created, enabled: false }],
      disabledProfileIds: [],
    });

    await useProfilesStore.getState().save(created);

    expect(toast.info).toHaveBeenCalledWith(expect.stringContaining("Third"));
  });

  it("applies a profile changed by the backend without saving it again", () => {
    useProfilesStore.getState().replace({ ...first, name: "Learned" });

    expect(useProfilesStore.getState().profiles[0]?.name).toBe("Learned");
    expect(mocked.saveProfile).not.toHaveBeenCalled();
  });

  it("moves selection to the first remaining profile after deleting the selected one", async () => {
    mocked.deleteProfile.mockResolvedValue([first]);

    await useProfilesStore.getState().remove(second.id);

    expect(useProfilesStore.getState().profiles).toEqual([first]);
    expect(useProfilesStore.getState().selectedId).toBe(first.id);
  });
});

describe("monitor store", () => {
  it("replaces the snapshot", () => {
    useMonitorStore.getState().setSnapshot({
      state: "closePending",
      sessionProfileId: "p",
      isTestSession: false,
      items: [],
      closesAtMs: 1_000,
      startingSimulator: null,
    });

    expect(useMonitorStore.getState().snapshot.state).toBe("closePending");
  });
});
