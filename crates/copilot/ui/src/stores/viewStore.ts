import { create } from "zustand";

export type ViewType = "office" | "task" | "debug" | "chat";

interface ViewState {
  activeView: ViewType;
  setActiveView: (view: ViewType) => void;
}

export const useViewStore = create<ViewState>((set) => ({
  activeView: "office",
  setActiveView: (view) => set({ activeView: view }),
}));
