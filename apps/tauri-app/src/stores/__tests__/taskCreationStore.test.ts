import { describe, it, expect, beforeEach } from "vitest";
import { useTaskCreationStore } from "../taskCreationStore";

describe("taskCreationStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useTaskCreationStore.setState({
      lastRepositoryGroupId: null,
    });
  });

  describe("lastRepositoryGroupId", () => {
    it("has default value of null", () => {
      expect(useTaskCreationStore.getState().lastRepositoryGroupId).toBeNull();
    });

    it("stores last repository group id", () => {
      useTaskCreationStore.getState().setLastRepositoryGroupId("group-123");
      expect(useTaskCreationStore.getState().lastRepositoryGroupId).toBe("group-123");
    });

    it("can clear last repository group id", () => {
      useTaskCreationStore.getState().setLastRepositoryGroupId("group-123");
      useTaskCreationStore.getState().setLastRepositoryGroupId(null);
      expect(useTaskCreationStore.getState().lastRepositoryGroupId).toBeNull();
    });
  });
});
