import { create } from "zustand";

export type ViewType = "office" | "task" | "debug" | "chat" | "settings";

interface ViewState {
  activeView: ViewType;
  setActiveView: (view: ViewType) => void;
}

export const useViewStore = create<ViewState>((set) => ({
  activeView: "office",
  setActiveView: (view) => set({ activeView: view }),
}));
