import { create } from "zustand";
import type { Settings } from "@/bindings/Settings";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";

interface SettingsState {
  settings: Settings | null;
  load: () => Promise<void>;
  save: (patch: Partial<Settings>) => Promise<void>;
}

export const useSettingsStore = create<SettingsState>()((set, get) => ({
  settings: null,

  load: async () => {
    set({ settings: await commands.getSettings() });
  },

  save: async (patch) => {
    const current = get().settings;
    if (!current) return;
    try {
      set({ settings: await commands.saveSettings({ ...current, ...patch }) });
    } catch (error) {
      notifyError(error);
    }
  },
}));
