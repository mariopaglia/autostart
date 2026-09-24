import { create } from "zustand";
import type { SessionLog } from "@/bindings/SessionLog";
import { notifyError } from "@/lib/notify";
import { commands } from "@/lib/tauri";

interface LogsStoreState {
  history: SessionLog[];
  load: () => Promise<void>;
  clear: () => Promise<void>;
}

export const useLogsStore = create<LogsStoreState>()((set) => ({
  history: [],
  load: async () => {
    try {
      set({ history: await commands.getSessionHistory() });
    } catch (error) {
      notifyError(error);
    }
  },
  clear: async () => {
    try {
      await commands.clearSessionHistory();
      set({ history: [] });
    } catch (error) {
      notifyError(error);
    }
  },
}));
