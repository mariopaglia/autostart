import { create } from "zustand";
import { i18n } from "@/i18n";
import { notifyError, notifySuccess } from "@/lib/notify";
import { system, updater, type Update } from "@/lib/tauri";

type UpdateStatus = "idle" | "checking" | "available" | "installing";

interface UpdatesState {
  status: UpdateStatus;
  update: Update | null;
  downloadedBytes: number;
  totalBytes: number | null;
  check: (options: { silent: boolean }) => Promise<void>;
  install: () => Promise<void>;
  dismiss: () => void;
}

export const useUpdatesStore = create<UpdatesState>()((set, get) => ({
  status: "idle",
  update: null,
  downloadedBytes: 0,
  totalBytes: null,

  check: async ({ silent }) => {
    if (get().status !== "idle") return;
    set({ status: "checking" });
    try {
      const update = await updater.check();
      set({ status: update ? "available" : "idle", update });
      if (!update && !silent) notifySuccess(i18n.t("updates.upToDate"));
    } catch (error) {
      set({ status: "idle" });
      // Startup checks run without the user asking, so offline machines must not see an error.
      void system.logWarning(`update check failed: ${String(error)}`);
      if (!silent) notifyError(error, i18n.t("updates.checkFailed"));
    }
  },

  install: async () => {
    const { update } = get();
    if (!update) return;
    set({ status: "installing", downloadedBytes: 0, totalBytes: null });
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") set({ totalBytes: event.data.contentLength ?? null });
        if (event.event === "Progress") {
          set((state) => ({ downloadedBytes: state.downloadedBytes + event.data.chunkLength }));
        }
      });
      await system.relaunch();
    } catch (error) {
      set({ status: "available" });
      void system.logWarning(`update install failed: ${String(error)}`);
      notifyError(error, i18n.t("updates.installFailed"));
    }
  },

  dismiss: () => {
    set({ status: "idle", update: null });
  },
}));
