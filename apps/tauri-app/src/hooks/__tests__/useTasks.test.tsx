import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import {
  useTasks,
  useTask,
  useCreateUnitTask,
  useCreateCompositeTask,
  useApproveTask,
  useRejectTask,
  useRequestChanges,
  useCancelTask,
  taskKeys,
} from "../useTasks";
import * as client from "@/api/client";
import { UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import type { UnitTask, CompositeTask, ListTasksResult, TaskResponse } from "@/api/types";

// Mock the API client
vi.mock("@/api/client", () => ({
  listTasks: vi.fn(),
  getTask: vi.fn(),
  createUnitTask: vi.fn(),
  createCompositeTask: vi.fn(),
  approveTask: vi.fn(),
  rejectTask: vi.fn(),
  requestChanges: vi.fn(),
  cancelTask: vi.fn(),
}));

const mockUnitTask: UnitTask = {
  id: "unit-1",
  repositoryGroupId: "repo-1",
  agentTaskId: "agent-1",
  prompt: "Test prompt",
  title: "Test Task",
  status: UnitTaskStatus.InProgress,
  autoFixTaskIds: [],
  createdAt: "2024-01-15T10:00:00Z",
  updatedAt: "2024-01-15T12:00:00Z",
};

const mockCompositeTask: CompositeTask = {
  id: "composite-1",
  repositoryGroupId: "repo-1",
  planningTaskId: "planning-1",
  prompt: "Test composite prompt",
  title: "Test Composite",
  nodeIds: [],
  status: CompositeTaskStatus.Planning,
  createdAt: "2024-01-15T10:00:00Z",
  updatedAt: "2024-01-15T12:00:00Z",
};

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
    },
  });

  return function Wrapper({ children }: { children: ReactNode }) {
    return <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>;
  };
};

describe("useTasks hooks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("taskKeys", () => {
    it("generates correct query keys", () => {
      expect(taskKeys.all).toEqual(["tasks"]);
      expect(taskKeys.lists()).toEqual(["tasks", "list"]);
      expect(taskKeys.list({ limit: 10 })).toEqual(["tasks", "list", { limit: 10 }]);
      expect(taskKeys.details()).toEqual(["tasks", "detail"]);
      expect(taskKeys.detail("task-123")).toEqual(["tasks", "detail", "task-123"]);
    });
  });

  describe("useTasks", () => {
    it("fetches tasks successfully", async () => {
      const mockResult: ListTasksResult = {
        unitTasks: [mockUnitTask],
        compositeTasks: [mockCompositeTask],
        totalCount: 2,
      };

      vi.mocked(client.listTasks).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useTasks({}), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockResult);
      expect(client.listTasks).toHaveBeenCalledWith({});
    });

    it("passes params to listTasks", async () => {
      const mockResult: ListTasksResult = {
        unitTasks: [],
        compositeTasks: [],
        totalCount: 0,
      };

      vi.mocked(client.listTasks).mockResolvedValue(mockResult);

      const params = { limit: 10, offset: 5 };
      renderHook(() => useTasks(params), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(client.listTasks).toHaveBeenCalledWith(params);
      });
    });

    it("handles error", async () => {
      const error = new Error("Failed to fetch tasks");
      vi.mocked(client.listTasks).mockRejectedValue(error);

      const { result } = renderHook(() => useTasks({}), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toEqual(error);
    });
  });

  describe("useTask", () => {
    it("fetches single task successfully", async () => {
      const mockResponse: TaskResponse = { unitTask: mockUnitTask };
      vi.mocked(client.getTask).mockResolvedValue(mockResponse);

      const { result } = renderHook(() => useTask("unit-1"), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockResponse);
      expect(client.getTask).toHaveBeenCalledWith("unit-1");
    });

    it("does not fetch when taskId is empty", () => {
      renderHook(() => useTask(""), { wrapper: createWrapper() });

      expect(client.getTask).not.toHaveBeenCalled();
    });
  });

  describe("useCreateUnitTask", () => {
    it("creates unit task successfully", async () => {
      vi.mocked(client.createUnitTask).mockResolvedValue(mockUnitTask);

      const { result } = renderHook(() => useCreateUnitTask(), { wrapper: createWrapper() });

      const params = {
        repositoryGroupId: "repo-1",
        prompt: "Test prompt",
      };

      await result.current.mutateAsync(params);

      expect(client.createUnitTask).toHaveBeenCalledWith(params);
    });
  });

  describe("useCreateCompositeTask", () => {
    it("creates composite task successfully", async () => {
      vi.mocked(client.createCompositeTask).mockResolvedValue(mockCompositeTask);

      const { result } = renderHook(() => useCreateCompositeTask(), { wrapper: createWrapper() });

      const params = {
        repositoryGroupId: "repo-1",
        prompt: "Test composite prompt",
      };

      await result.current.mutateAsync(params);

      expect(client.createCompositeTask).toHaveBeenCalledWith(params);
    });
  });

  describe("useApproveTask", () => {
    it("approves task successfully", async () => {
      vi.mocked(client.approveTask).mockResolvedValue(undefined);

      const { result } = renderHook(() => useApproveTask(), { wrapper: createWrapper() });

      await result.current.mutateAsync("task-1");

      expect(client.approveTask).toHaveBeenCalledWith("task-1");
    });
  });

  describe("useRejectTask", () => {
    it("rejects task successfully", async () => {
      vi.mocked(client.rejectTask).mockResolvedValue(undefined);

      const { result } = renderHook(() => useRejectTask(), { wrapper: createWrapper() });

      await result.current.mutateAsync({ taskId: "task-1", reason: "Not approved" });

      expect(client.rejectTask).toHaveBeenCalledWith("task-1", "Not approved");
    });

    it("rejects task without reason", async () => {
      vi.mocked(client.rejectTask).mockResolvedValue(undefined);

      const { result } = renderHook(() => useRejectTask(), { wrapper: createWrapper() });

      await result.current.mutateAsync({ taskId: "task-1" });

      expect(client.rejectTask).toHaveBeenCalledWith("task-1", undefined);
    });
  });

  describe("useRequestChanges", () => {
    it("requests changes successfully", async () => {
      vi.mocked(client.requestChanges).mockResolvedValue(undefined);

      const { result } = renderHook(() => useRequestChanges(), { wrapper: createWrapper() });

      await result.current.mutateAsync({ taskId: "task-1", feedback: "Please fix this" });

      expect(client.requestChanges).toHaveBeenCalledWith("task-1", "Please fix this");
    });
  });

  describe("useCancelTask", () => {
    it("cancels task successfully", async () => {
      vi.mocked(client.cancelTask).mockResolvedValue(undefined);

      const { result } = renderHook(() => useCancelTask(), { wrapper: createWrapper() });

      await result.current.mutateAsync("task-1");

      expect(client.cancelTask).toHaveBeenCalledWith("task-1");
    });

    it("handles cancel task error", async () => {
      const error = new Error("Failed to cancel task");
      vi.mocked(client.cancelTask).mockRejectedValue(error);

      const { result } = renderHook(() => useCancelTask(), { wrapper: createWrapper() });

      await expect(result.current.mutateAsync("task-1")).rejects.toThrow("Failed to cancel task");
    });
  });
});
