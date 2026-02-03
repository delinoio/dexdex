// Task creation preferences state management with Zustand
import { create } from "zustand";
import { persist } from "zustand/middleware";
import { AiAgentType } from "@/api/types";

interface TaskCreationPreferences {
  // Last selected repository/group selection (can be group ID or repository ID with prefix)
  lastSelection: string | null;
  // Last selected agent type for unit tasks
  lastAgentType: AiAgentType;
  // Last selected planning agent type for composite tasks
  lastPlanningAgentType: AiAgentType;
  // Last selected execution agent type for composite tasks
  lastExecutionAgentType: AiAgentType;
  // Last composite mode state
  lastIsComposite: boolean;
}

interface TaskCreationState extends TaskCreationPreferences {
  setLastSelection: (selection: string | null) => void;
  setLastAgentType: (agentType: AiAgentType) => void;
  setLastPlanningAgentType: (agentType: AiAgentType) => void;
  setLastExecutionAgentType: (agentType: AiAgentType) => void;
  setLastIsComposite: (isComposite: boolean) => void;
}

export const useTaskCreationStore = create<TaskCreationState>()(
  persist(
    (set) => ({
      lastSelection: null,
      lastAgentType: AiAgentType.ClaudeCode,
      lastPlanningAgentType: AiAgentType.ClaudeCode,
      lastExecutionAgentType: AiAgentType.ClaudeCode,
      lastIsComposite: false,

      setLastSelection: (selection) => set({ lastSelection: selection }),
      setLastAgentType: (agentType) => set({ lastAgentType: agentType }),
      setLastPlanningAgentType: (agentType) =>
        set({ lastPlanningAgentType: agentType }),
      setLastExecutionAgentType: (agentType) =>
        set({ lastExecutionAgentType: agentType }),
      setLastIsComposite: (isComposite) =>
        set({ lastIsComposite: isComposite }),
    }),
    {
      name: "delidev-task-creation-store",
    }
  )
);
