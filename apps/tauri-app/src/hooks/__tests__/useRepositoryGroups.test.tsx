import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, waitFor } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import type { ReactNode } from "react";
import {
  useRepositoryGroups,
  useRepositoryGroup,
  useCreateRepositoryGroup,
  useUpdateRepositoryGroup,
  useDeleteRepositoryGroup,
  repositoryGroupKeys,
} from "../useRepositoryGroups";
import * as client from "@/api/client";
import type { RepositoryGroup, ListRepositoryGroupsResult } from "@/api/types";

// Mock the API client
vi.mock("@/api/client", () => ({
  listRepositoryGroups: vi.fn(),
  getRepositoryGroup: vi.fn(),
  createRepositoryGroup: vi.fn(),
  updateRepositoryGroup: vi.fn(),
  deleteRepositoryGroup: vi.fn(),
}));

const mockRepositoryGroup: RepositoryGroup = {
  id: "group-1",
  workspaceId: "workspace-1",
  name: "Test Group",
  repositoryIds: ["repo-1", "repo-2"],
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
    return (
      <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
    );
  };
};

describe("useRepositoryGroups hooks", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("repositoryGroupKeys", () => {
    it("generates correct query keys", () => {
      expect(repositoryGroupKeys.all).toEqual(["repositoryGroups"]);
      expect(repositoryGroupKeys.lists()).toEqual(["repositoryGroups", "list"]);
      expect(repositoryGroupKeys.list({ limit: 10 })).toEqual([
        "repositoryGroups",
        "list",
        { limit: 10 },
      ]);
      expect(repositoryGroupKeys.details()).toEqual([
        "repositoryGroups",
        "detail",
      ]);
      expect(repositoryGroupKeys.detail("group-123")).toEqual([
        "repositoryGroups",
        "detail",
        "group-123",
      ]);
    });
  });

  describe("useRepositoryGroups", () => {
    it("fetches repository groups successfully", async () => {
      const mockResult: ListRepositoryGroupsResult = {
        groups: [mockRepositoryGroup],
        totalCount: 1,
      };

      vi.mocked(client.listRepositoryGroups).mockResolvedValue(mockResult);

      const { result } = renderHook(() => useRepositoryGroups({}), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockResult);
      expect(client.listRepositoryGroups).toHaveBeenCalledWith({});
    });

    it("passes params to listRepositoryGroups", async () => {
      const mockResult: ListRepositoryGroupsResult = {
        groups: [],
        totalCount: 0,
      };

      vi.mocked(client.listRepositoryGroups).mockResolvedValue(mockResult);

      const params = { limit: 10, offset: 5 };
      renderHook(() => useRepositoryGroups(params), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(client.listRepositoryGroups).toHaveBeenCalledWith(params);
      });
    });

    it("handles error", async () => {
      const error = new Error("Failed to fetch repository groups");
      vi.mocked(client.listRepositoryGroups).mockRejectedValue(error);

      const { result } = renderHook(() => useRepositoryGroups({}), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isError).toBe(true);
      });

      expect(result.current.error).toEqual(error);
    });
  });

  describe("useRepositoryGroup", () => {
    it("fetches single repository group successfully", async () => {
      vi.mocked(client.getRepositoryGroup).mockResolvedValue(
        mockRepositoryGroup
      );

      const { result } = renderHook(() => useRepositoryGroup("group-1"), {
        wrapper: createWrapper(),
      });

      await waitFor(() => {
        expect(result.current.isSuccess).toBe(true);
      });

      expect(result.current.data).toEqual(mockRepositoryGroup);
      expect(client.getRepositoryGroup).toHaveBeenCalledWith("group-1");
    });

    it("does not fetch when groupId is empty", () => {
      renderHook(() => useRepositoryGroup(""), { wrapper: createWrapper() });

      expect(client.getRepositoryGroup).not.toHaveBeenCalled();
    });
  });

  describe("useCreateRepositoryGroup", () => {
    it("creates repository group successfully", async () => {
      vi.mocked(client.createRepositoryGroup).mockResolvedValue(
        mockRepositoryGroup
      );

      const { result } = renderHook(() => useCreateRepositoryGroup(), {
        wrapper: createWrapper(),
      });

      const params = {
        name: "Test Group",
        repositoryIds: ["repo-1", "repo-2"],
      };

      await result.current.mutateAsync(params);

      expect(client.createRepositoryGroup).toHaveBeenCalledWith(params);
    });

    it("creates repository group without name", async () => {
      const groupWithoutName = { ...mockRepositoryGroup, name: undefined };
      vi.mocked(client.createRepositoryGroup).mockResolvedValue(
        groupWithoutName
      );

      const { result } = renderHook(() => useCreateRepositoryGroup(), {
        wrapper: createWrapper(),
      });

      const params = {
        repositoryIds: ["repo-1"],
      };

      await result.current.mutateAsync(params);

      expect(client.createRepositoryGroup).toHaveBeenCalledWith(params);
    });
  });

  describe("useUpdateRepositoryGroup", () => {
    it("updates repository group successfully", async () => {
      const updatedGroup = { ...mockRepositoryGroup, name: "Updated Name" };
      vi.mocked(client.updateRepositoryGroup).mockResolvedValue(updatedGroup);

      const { result } = renderHook(() => useUpdateRepositoryGroup(), {
        wrapper: createWrapper(),
      });

      await result.current.mutateAsync({
        groupId: "group-1",
        params: { name: "Updated Name", repositoryIds: ["repo-1", "repo-2"] },
      });

      expect(client.updateRepositoryGroup).toHaveBeenCalledWith("group-1", {
        name: "Updated Name",
        repositoryIds: ["repo-1", "repo-2"],
      });
    });

    it("updates repository group with new repositories", async () => {
      const updatedGroup = {
        ...mockRepositoryGroup,
        repositoryIds: ["repo-3"],
      };
      vi.mocked(client.updateRepositoryGroup).mockResolvedValue(updatedGroup);

      const { result } = renderHook(() => useUpdateRepositoryGroup(), {
        wrapper: createWrapper(),
      });

      await result.current.mutateAsync({
        groupId: "group-1",
        params: { repositoryIds: ["repo-3"] },
      });

      expect(client.updateRepositoryGroup).toHaveBeenCalledWith("group-1", {
        repositoryIds: ["repo-3"],
      });
    });
  });

  describe("useDeleteRepositoryGroup", () => {
    it("deletes repository group successfully", async () => {
      vi.mocked(client.deleteRepositoryGroup).mockResolvedValue(undefined);

      const { result } = renderHook(() => useDeleteRepositoryGroup(), {
        wrapper: createWrapper(),
      });

      await result.current.mutateAsync("group-1");

      expect(client.deleteRepositoryGroup).toHaveBeenCalledWith("group-1");
    });
  });
});
