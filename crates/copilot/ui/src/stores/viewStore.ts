import { create } from "zustand";

export type ViewType =
  | "valley"
  | "office"
  | "task"
  | "debug"
  | "chat"
  | "settings";

interface ViewState {
  activeView: ViewType;
  setActiveView: (view: ViewType) => void;
}

export const useViewStore = create<ViewState>((set) => ({
  activeView: "valley",
  setActiveView: (view) => set({ activeView: view }),
}));
