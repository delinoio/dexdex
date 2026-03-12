// Task creation preferences state management with Zustand
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface TaskCreationPreferences {
  // Last selected repository group ID
  lastRepositoryGroupId: string | null;
}

interface TaskCreationState extends TaskCreationPreferences {
  setLastRepositoryGroupId: (id: string | null) => void;
}

export const useTaskCreationStore = create<TaskCreationState>()(
  persist(
    (set) => ({
      lastRepositoryGroupId: null,
      setLastRepositoryGroupId: (id) => set({ lastRepositoryGroupId: id }),
    }),
    {
      name: "dexdex-task-creation-store",
    }
  )
);
