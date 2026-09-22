import { create } from "zustand";

export type Screen = "main" | "logs" | "settings";

interface UiState {
  screen: Screen;
  setScreen: (screen: Screen) => void;
}

export const useUiStore = create<UiState>()((set) => ({
  screen: "main",
  setScreen: (screen) => {
    set({ screen });
  },
}));
