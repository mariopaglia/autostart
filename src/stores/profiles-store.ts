import { create } from "zustand";
import type { Profile } from "@/bindings/Profile";
import type { ProfilesUpdate } from "@/bindings/ProfilesUpdate";
import { i18n } from "@/i18n";
import { notifyError, notifyInfo } from "@/lib/notify";
import { commands } from "@/lib/tauri";

interface ProfilesState {
  profiles: Profile[];
  selectedId: string | null;
  load: () => Promise<void>;
  select: (profileId: string) => void;
  save: (profile: Profile) => Promise<boolean>;
  setEnabled: (profileId: string, enabled: boolean) => Promise<void>;
  remove: (profileId: string) => Promise<void>;
  replace: (profile: Profile) => void;
}

function upsert(profiles: Profile[], profile: Profile): Profile[] {
  const exists = profiles.some((candidate) => candidate.id === profile.id);
  return exists
    ? profiles.map((candidate) => (candidate.id === profile.id ? profile : candidate))
    : [...profiles, profile];
}

/** Tells the pilot which profiles stopped being watched because another one took their simulator. */
function announce(update: ProfilesUpdate, requested: Profile | undefined) {
  const names = update.profiles
    .filter((profile) => update.disabledProfileIds.includes(profile.id))
    .map((profile) => profile.name);
  if (names.length > 0) {
    notifyInfo(i18n.t("profiles.conflictDisabled", { names: names.join(", ") }));
  }

  const saved = update.profiles.find((profile) => profile.id === requested?.id);
  if (requested?.enabled && saved && !saved.enabled) {
    notifyInfo(i18n.t("profiles.createdDisabled", { name: saved.name }));
  }
}

export const useProfilesStore = create<ProfilesState>()((set, get) => ({
  profiles: [],
  selectedId: null,

  load: async () => {
    const profiles = await commands.getProfiles();
    const preferred = profiles.find((profile) => profile.enabled) ?? profiles[0];
    set({ profiles, selectedId: preferred?.id ?? null });
  },

  select: (profileId) => {
    set({ selectedId: profileId });
  },

  save: async (profile) => {
    const previous = get().profiles;
    set({ profiles: upsert(previous, profile) });
    try {
      const update = await commands.saveProfile(profile);
      set({ profiles: update.profiles });
      announce(update, profile);
      return true;
    } catch (error) {
      set({ profiles: previous });
      notifyError(error);
      return false;
    }
  },

  setEnabled: async (profileId, enabled) => {
    try {
      const update = await commands.setProfileEnabled(profileId, enabled);
      set({ profiles: update.profiles });
      announce(update, undefined);
    } catch (error) {
      notifyError(error);
    }
  },

  replace: (profile) => {
    set({ profiles: upsert(get().profiles, profile) });
  },

  remove: async (profileId) => {
    try {
      const profiles = await commands.deleteProfile(profileId);
      const selectedId =
        get().selectedId === profileId ? (profiles[0]?.id ?? null) : get().selectedId;
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
