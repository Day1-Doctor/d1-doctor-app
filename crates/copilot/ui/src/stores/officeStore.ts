import { create } from 'zustand';

interface Office {
  id: string;
  name: string;
  agentCount: number;
  skillCount: number;
  fileCount: number;
  taskProgress: number;
}

interface OfficeState {
  offices: Office[];
  addOffice: (office: Office) => void;
  renameOffice: (id: string, name: string) => void;
  removeOffice: (id: string) => void;
}

const defaultOffices: Office[] = [
  { id: 'office-1', name: "Dr. Bob's Office", agentCount: 6, skillCount: 16, fileCount: 0, taskProgress: 0 },
];

export const useOfficeStore = create<OfficeState>((set) => ({
  offices: JSON.parse(localStorage.getItem('day1-offices') || 'null') || defaultOffices,
  addOffice: (office) => set((state) => {
    const offices = [...state.offices, office];
    localStorage.setItem('day1-offices', JSON.stringify(offices));
    return { offices };
  }),
  renameOffice: (id, name) => set((state) => {
    const offices = state.offices.map(o => o.id === id ? { ...o, name } : o);
    localStorage.setItem('day1-offices', JSON.stringify(offices));
    return { offices };
  }),
  removeOffice: (id) => set((state) => {
    const offices = state.offices.filter(o => o.id !== id);
    localStorage.setItem('day1-offices', JSON.stringify(offices));
    return { offices };
  }),
}));
