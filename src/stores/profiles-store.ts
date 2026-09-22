import { create } from "zustand";
import type { Profile } from "@/bindings/Profile";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";
import { useSettingsStore } from "@/stores/settings-store";

interface ProfilesState {
  profiles: Profile[];
  selectedId: string | null;
  load: () => Promise<void>;
  select: (profileId: string) => void;
  save: (profile: Profile) => Promise<boolean>;
  remove: (profileId: string) => Promise<void>;
  replace: (profile: Profile) => void;
}

function upsert(profiles: Profile[], profile: Profile): Profile[] {
  const exists = profiles.some((candidate) => candidate.id === profile.id);
  return exists
    ? profiles.map((candidate) => (candidate.id === profile.id ? profile : candidate))
    : [...profiles, profile];
}

export const useProfilesStore = create<ProfilesState>()((set, get) => ({
  profiles: [],
  selectedId: null,

  load: async () => {
    const profiles = await commands.getProfiles();
    const activeId = useSettingsStore.getState().settings?.activeProfileId ?? null;
    set({ profiles, selectedId: activeId ?? profiles[0]?.id ?? null });
  },

  select: (profileId) => {
    set({ selectedId: profileId });
  },

  save: async (profile) => {
    const previous = get().profiles;
    set({ profiles: upsert(previous, profile) });
    try {
      await commands.saveProfile(profile);
      return true;
    } catch (error) {
      set({ profiles: previous });
      notifyError(error);
      return false;
    }
  },

  replace: (profile) => {
    set({ profiles: upsert(get().profiles, profile) });
  },

  remove: async (profileId) => {
    try {
      const settings = await commands.deleteProfile(profileId);
      useSettingsStore.getState().replace(settings);
      const profiles = get().profiles.filter((profile) => profile.id !== profileId);
      const selectedId =
        get().selectedId === profileId ? (settings.activeProfileId ?? null) : get().selectedId;
      set({ profiles, selectedId });
    } catch (error) {
      notifyError(error);
    }
  },
}));

export function useSelectedProfile(): Profile | undefined {
  return useProfilesStore((state) =>
    state.profiles.find((profile) => profile.id === state.selectedId),
  );
}
