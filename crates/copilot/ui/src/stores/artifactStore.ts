import { create } from "zustand";

export type ArtifactType = "document" | "code" | "chart" | "image";

export interface Artifact {
  id: string;
  name: string;
  type: ArtifactType;
  agent: string;
  size: number; // bytes
  timestamp: string;
  /** Optional text content for preview (code/document types). */
  content?: string;
}

interface ArtifactState {
  artifacts: Artifact[];
  selectedArtifactId: string | null;
  setArtifacts: (artifacts: Artifact[]) => void;
  addArtifact: (artifact: Artifact) => void;
  selectArtifact: (id: string | null) => void;
  getSelectedArtifact: () => Artifact | undefined;
}

const mockArtifacts: Artifact[] = [
  {
    id: "1",
    name: "research-notes.md",
    type: "document",
    agent: "Scout",
    size: 4200,
    timestamp: "2026-03-18T10:23:00Z",
    content:
      "# Research Notes\n\n## AI Framework Comparison\n\n### TensorFlow\n- Mature ecosystem\n- Strong production tooling\n- Large model zoo\n\n### PyTorch\n- Dominant in research\n- Dynamic computation graphs\n- Growing production support",
  },
  {
    id: "2",
    name: "comparison-chart.svg",
    type: "chart",
    agent: "Sage",
    size: 12800,
    timestamp: "2026-03-18T10:31:00Z",
  },
  {
    id: "3",
    name: "final-report.pdf",
    type: "document",
    agent: "Quill",
    size: 28400,
    timestamp: "2026-03-18T10:45:00Z",
    content:
      "AI Framework Comparison Report\n\nExecutive Summary: This report evaluates five major AI frameworks across performance, ecosystem maturity, and developer experience...",
  },
];

export const useArtifactStore = create<ArtifactState>((set, get) => ({
  artifacts: mockArtifacts,
  selectedArtifactId: null,
  setArtifacts: (artifacts) => set({ artifacts }),
  addArtifact: (artifact) =>
    set((state) => ({
      artifacts: [...state.artifacts, artifact],
    })),
  selectArtifact: (id) => set({ selectedArtifactId: id }),
  getSelectedArtifact: () => {
    const state = get();
    return state.artifacts.find((a) => a.id === state.selectedArtifactId);
  },
}));
