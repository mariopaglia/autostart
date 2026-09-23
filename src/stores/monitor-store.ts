import { create } from "zustand";
import type { ItemRuntime } from "@/bindings/ItemRuntime";
import type { MonitorSnapshot } from "@/bindings/MonitorSnapshot";
import type { SessionLog } from "@/bindings/SessionLog";

interface MonitorStoreState {
  snapshot: MonitorSnapshot;
  sessionLog: SessionLog | null;
  setSnapshot: (snapshot: MonitorSnapshot) => void;
  setSessionLog: (sessionLog: SessionLog | null) => void;
}

const INITIAL_SNAPSHOT: MonitorSnapshot = {
  state: "idle",
  sessionProfileId: null,
  isTestSession: false,
  items: [],
  closesAtMs: null,
};

export const useMonitorStore = create<MonitorStoreState>()((set) => ({
  snapshot: INITIAL_SNAPSHOT,
  sessionLog: null,
  setSnapshot: (snapshot) => {
    set({ snapshot });
  },
  setSessionLog: (sessionLog) => {
    set({ sessionLog });
  },
}));

export function useItemRuntime(profileId: string, itemId: string): ItemRuntime | undefined {
  return useMonitorStore((state) =>
    state.snapshot.sessionProfileId === profileId
      ? state.snapshot.items.find((runtime) => runtime.itemId === itemId)
      : undefined,
  );
}

export function useHasSession(): boolean {
  return useMonitorStore((state) =>
    ["simRunning", "closePending", "closing"].includes(state.snapshot.state),
  );
}
