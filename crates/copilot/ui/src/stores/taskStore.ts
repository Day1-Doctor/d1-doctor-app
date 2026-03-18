import { create } from "zustand";

export type TaskStatus =
  | "pending"
  | "running"
  | "paused"
  | "completed"
  | "failed"
  | "cancelled";

export interface TaskStep {
  id: string;
  title: string;
  status: "pending" | "running" | "completed" | "failed" | "cancelled";
  agentName?: string;
  agentRole?: string;
  duration?: number; // ms
  isParallel?: boolean;
  parallelGroup?: string;
}

export interface Task {
  id: string;
  title: string;
  status: TaskStatus;
  steps: TaskStep[];
  totalDuration?: number; // ms
}

interface TaskState {
  tasks: Task[];
  activeTaskId: string | null;
  setTasks: (tasks: Task[]) => void;
  setActiveTask: (id: string | null) => void;
  updateTaskStatus: (id: string, status: TaskStatus) => void;
  updateStepStatus: (
    taskId: string,
    stepId: string,
    status: TaskStep["status"],
    duration?: number,
  ) => void;
  getActiveTask: () => Task | undefined;
}

const mockSteps: TaskStep[] = [
  {
    id: "1",
    title: "Research AI frameworks",
    status: "completed",
    agentName: "Scout",
    agentRole: "researcher",
    duration: 45000,
  },
  {
    id: "2",
    title: "Analyze findings",
    status: "running",
    agentName: "Sage",
    agentRole: "analyst",
  },
  {
    id: "3",
    title: "Write comparison report",
    status: "pending",
    agentName: "Quill",
    agentRole: "writer",
  },
  {
    id: "4",
    title: "Save to workspace",
    status: "pending",
    agentName: "Atlas",
    agentRole: "operator",
  },
];

const mockParallelSteps: TaskStep[] = [
  {
    id: "p-0",
    title: "Research",
    status: "completed",
    agentName: "Scout",
    agentRole: "researcher",
    duration: 32000,
  },
  {
    id: "p-1a",
    title: "Analyze data",
    status: "running",
    agentName: "Sage",
    agentRole: "analyst",
    isParallel: true,
    parallelGroup: "group-1",
  },
  {
    id: "p-1b",
    title: "Review sources",
    status: "running",
    agentName: "Quill",
    agentRole: "writer",
    isParallel: true,
    parallelGroup: "group-1",
  },
  {
    id: "p-2",
    title: "Write report",
    status: "pending",
    agentName: "Quill",
    agentRole: "writer",
  },
];

const mockTasks: Task[] = [
  {
    id: "task-1",
    title: "Compare AI Frameworks",
    status: "running",
    steps: mockSteps,
    totalDuration: 45000,
  },
  {
    id: "task-2",
    title: "Research Report with Parallel Analysis",
    status: "running",
    steps: mockParallelSteps,
    totalDuration: 32000,
  },
];

export const useTaskStore = create<TaskState>((set, get) => ({
  tasks: mockTasks,
  activeTaskId: "task-1",
  setTasks: (tasks) => set({ tasks }),
  setActiveTask: (id) => set({ activeTaskId: id }),
  updateTaskStatus: (id, status) =>
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === id ? { ...t, status } : t,
      ),
    })),
  updateStepStatus: (taskId, stepId, status, duration) =>
    set((state) => ({
      tasks: state.tasks.map((t) =>
        t.id === taskId
          ? {
              ...t,
              steps: t.steps.map((s) =>
                s.id === stepId
                  ? { ...s, status, ...(duration != null ? { duration } : {}) }
                  : s,
              ),
            }
          : t,
      ),
    })),
  getActiveTask: () => {
    const state = get();
    return state.tasks.find((t) => t.id === state.activeTaskId);
  },
}));
