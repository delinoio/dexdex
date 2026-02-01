import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import {
  useWorkspaces,
  useWorkspace,
  useDefaultWorkspaceId,
  useCreateWorkspace,
  useUpdateWorkspace,
  useDeleteWorkspace,
  workspaceKeys,
} from "../useWorkspaces";
import * as client from "@/api/client";
import type { Workspace, ListWorkspacesResult } from "@/api/types";

// Mock the API client
vi.mock("@/api/client", () => ({
  listWorkspaces: vi.fn(),
  getWorkspace: vi.fn(),
  getDefaultWorkspaceId: vi.fn(),
  createWorkspace: vi.fn(),
  updateWorkspace: vi.fn(),
  deleteWorkspace: vi.fn(),
}));

const mockWorkspace: Workspace = {
  id: "workspace-1",
  name: "Test Workspace",
  description: "A test workspace",
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

describe("useWorkspaces hooks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("workspaceKeys", () => {
    it("generates correct query keys", () => {
      expect(workspaceKeys.all).toEqual(["workspaces"]);
      expect(workspaceKeys.lists()).toEqual(["workspaces", "list"]);
      expect(workspaceKeys.list({ limit: 10 })).toEqual(["workspaces", "list", { limit: 10 }]);
      expect(workspaceKeys.details()).toEqual(["workspaces", "detail"]);
      expect(workspaceKeys.detail("ws-123")).toEqual(["workspaces", "detail", "ws-123"]);
      expect(workspaceKeys.defaultId()).toEqual(["workspaces", "default"]);
    });
  });

  describe("useWorkspaces", () => {
    it("fetches workspaces successfully", async () => {
      const mockResult: ListWorkspacesResult = {
        workspaces: [mockWorkspace],
        totalCount: 1,
      };

      vi.mocked(client.listWorkspaces).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useWorkspaces({}), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockResult);
      expect(client.listWorkspaces).toHaveBeenCalledWith({});
    });

    it("passes params to listWorkspaces", async () => {
      const mockResult: ListWorkspacesResult = {
        workspaces: [],
        totalCount: 0,
      };

      vi.mocked(client.listWorkspaces).mockResolvedValue(mockResult);

      const params = { limit: 10, offset: 5 };
      renderHook(() => useWorkspaces(params), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(client.listWorkspaces).toHaveBeenCalledWith(params);
      });
    });

    it("handles error", async () => {
      const error = new Error("Failed to fetch workspaces");
      vi.mocked(client.listWorkspaces).mockRejectedValue(error);

      const { result } = renderHook(() => useWorkspaces({}), { wrapper: createWrapper() });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toEqual(error);
    });
  });

  describe("useWorkspace", () => {
    it("fetches single workspace successfully", async () => {
      vi.mocked(client.getWorkspace).mockResolvedValue(mockWorkspace);

      const { result } = renderHook(() => useWorkspace("workspace-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockWorkspace);
      expect(client.getWorkspace).toHaveBeenCalledWith("workspace-1");
    });

    it("does not fetch when workspaceId is empty", () => {
      renderHook(() => useWorkspace(""), { wrapper: createWrapper() });

      expect(client.getWorkspace).not.toHaveBeenCalled();
    });
  });

  describe("useDefaultWorkspaceId", () => {
    it("fetches default workspace ID successfully", async () => {
      vi.mocked(client.getDefaultWorkspaceId).mockResolvedValue("default-workspace-id");

      const { result } = renderHook(() => useDefaultWorkspaceId(), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toBe("default-workspace-id");
      expect(client.getDefaultWorkspaceId).toHaveBeenCalled();
    });
  });

  describe("useCreateWorkspace", () => {
    it("creates workspace successfully", async () => {
      vi.mocked(client.createWorkspace).mockResolvedValue(mockWorkspace);

      const { result } = renderHook(() => useCreateWorkspace(), { wrapper: createWrapper() });

      const params = {
        name: "Test Workspace",
        description: "A test workspace",
      };

      await result.current.mutateAsync(params);

      expect(client.createWorkspace).toHaveBeenCalledWith(params);
    });
  });

  describe("useUpdateWorkspace", () => {
    it("updates workspace successfully", async () => {
      const updatedWorkspace = { ...mockWorkspace, name: "Updated Name" };
      vi.mocked(client.updateWorkspace).mockResolvedValue(updatedWorkspace);

      const { result } = renderHook(() => useUpdateWorkspace(), { wrapper: createWrapper() });

      await result.current.mutateAsync({
        workspaceId: "workspace-1",
        params: { name: "Updated Name" },
      });

      expect(client.updateWorkspace).toHaveBeenCalledWith("workspace-1", { name: "Updated Name" });
    });
  });

  describe("useDeleteWorkspace", () => {
    it("deletes workspace successfully", async () => {
      vi.mocked(client.deleteWorkspace).mockResolvedValue(undefined);

      const { result } = renderHook(() => useDeleteWorkspace(), { wrapper: createWrapper() });

      await result.current.mutateAsync("workspace-1");

      expect(client.deleteWorkspace).toHaveBeenCalledWith("workspace-1");
    });
  });
});
