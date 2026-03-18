import { create } from "zustand";

interface LayoutState {
  sidebarCollapsed: boolean;
  rightPanelCollapsed: boolean;
  toggleSidebar: () => void;
  toggleRightPanel: () => void;
  setSidebarCollapsed: (collapsed: boolean) => void;
  setRightPanelCollapsed: (collapsed: boolean) => void;
}

export const useLayoutStore = create<LayoutState>((set) => ({
  sidebarCollapsed: false,
  rightPanelCollapsed: false,
  toggleSidebar: () =>
    set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),
  toggleRightPanel: () =>
    set((state) => ({ rightPanelCollapsed: !state.rightPanelCollapsed })),
  setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
  setRightPanelCollapsed: (collapsed) =>
    set({ rightPanelCollapsed: collapsed }),
}));
