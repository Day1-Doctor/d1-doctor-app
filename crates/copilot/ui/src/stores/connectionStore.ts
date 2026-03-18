import { create } from "zustand";

interface ConnectionState {
  isConnected: boolean;
  lastEventAt: string | null;
  setConnected: (connected: boolean) => void;
  setLastEvent: (timestamp: string) => void;
}

export const useConnectionStore = create<ConnectionState>((set) => ({
  isConnected: false,
  lastEventAt: null,
  setConnected: (connected) => set({ isConnected: connected }),
  setLastEvent: (timestamp) => set({ lastEventAt: timestamp }),
}));
