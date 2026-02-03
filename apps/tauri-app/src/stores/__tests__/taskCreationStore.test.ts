import { describe, it, expect, beforeEach } from "vitest";
import { useTaskCreationStore } from "../taskCreationStore";
import { AiAgentType } from "@/api/types";

describe("taskCreationStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useTaskCreationStore.setState({
      lastSelection: null,
      lastAgentType: AiAgentType.ClaudeCode,
      lastPlanningAgentType: AiAgentType.ClaudeCode,
      lastExecutionAgentType: AiAgentType.ClaudeCode,
      lastIsComposite: false,
    });
  });

  describe("lastSelection", () => {
    it("stores last repository/group selection", () => {
      expect(useTaskCreationStore.getState().lastSelection).toBeNull();

      useTaskCreationStore.getState().setLastSelection("group-123");
      expect(useTaskCreationStore.getState().lastSelection).toBe("group-123");
    });

    it("stores repository selection with prefix", () => {
      useTaskCreationStore.getState().setLastSelection("__repo__repo-456");
      expect(useTaskCreationStore.getState().lastSelection).toBe("__repo__repo-456");
    });

    it("can clear last selection", () => {
      useTaskCreationStore.getState().setLastSelection("group-123");
      useTaskCreationStore.getState().setLastSelection(null);
      expect(useTaskCreationStore.getState().lastSelection).toBeNull();
    });
  });

  describe("lastAgentType", () => {
    it("has default value of ClaudeCode", () => {
      expect(useTaskCreationStore.getState().lastAgentType).toBe(AiAgentType.ClaudeCode);
    });

    it("stores last agent type for unit tasks", () => {
      useTaskCreationStore.getState().setLastAgentType(AiAgentType.Aider);
      expect(useTaskCreationStore.getState().lastAgentType).toBe(AiAgentType.Aider);
    });

    it("can set any valid agent type", () => {
      const agentTypes = [
        AiAgentType.ClaudeCode,
        AiAgentType.OpenCode,
        AiAgentType.GeminiCli,
        AiAgentType.CodexCli,
        AiAgentType.Aider,
        AiAgentType.Amp,
      ];

      for (const agentType of agentTypes) {
        useTaskCreationStore.getState().setLastAgentType(agentType);
        expect(useTaskCreationStore.getState().lastAgentType).toBe(agentType);
      }
    });
  });

  describe("lastPlanningAgentType", () => {
    it("has default value of ClaudeCode", () => {
      expect(useTaskCreationStore.getState().lastPlanningAgentType).toBe(AiAgentType.ClaudeCode);
    });

    it("stores last planning agent type for composite tasks", () => {
      useTaskCreationStore.getState().setLastPlanningAgentType(AiAgentType.GeminiCli);
      expect(useTaskCreationStore.getState().lastPlanningAgentType).toBe(AiAgentType.GeminiCli);
    });
  });

  describe("lastExecutionAgentType", () => {
    it("has default value of ClaudeCode", () => {
      expect(useTaskCreationStore.getState().lastExecutionAgentType).toBe(AiAgentType.ClaudeCode);
    });

    it("stores last execution agent type for composite tasks", () => {
      useTaskCreationStore.getState().setLastExecutionAgentType(AiAgentType.OpenCode);
      expect(useTaskCreationStore.getState().lastExecutionAgentType).toBe(AiAgentType.OpenCode);
    });
  });

  describe("lastIsComposite", () => {
    it("has default value of false", () => {
      expect(useTaskCreationStore.getState().lastIsComposite).toBe(false);
    });

    it("stores last composite mode preference", () => {
      useTaskCreationStore.getState().setLastIsComposite(true);
      expect(useTaskCreationStore.getState().lastIsComposite).toBe(true);

      useTaskCreationStore.getState().setLastIsComposite(false);
      expect(useTaskCreationStore.getState().lastIsComposite).toBe(false);
    });
  });

  describe("multiple preferences", () => {
    it("can store all preferences independently", () => {
      useTaskCreationStore.getState().setLastSelection("__repo__repo-123");
      useTaskCreationStore.getState().setLastAgentType(AiAgentType.Aider);
      useTaskCreationStore.getState().setLastPlanningAgentType(AiAgentType.GeminiCli);
      useTaskCreationStore.getState().setLastExecutionAgentType(AiAgentType.OpenCode);
      useTaskCreationStore.getState().setLastIsComposite(true);

      const state = useTaskCreationStore.getState();
      expect(state.lastSelection).toBe("__repo__repo-123");
      expect(state.lastAgentType).toBe(AiAgentType.Aider);
      expect(state.lastPlanningAgentType).toBe(AiAgentType.GeminiCli);
      expect(state.lastExecutionAgentType).toBe(AiAgentType.OpenCode);
      expect(state.lastIsComposite).toBe(true);
    });
  });
});
