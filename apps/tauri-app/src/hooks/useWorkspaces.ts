// React Query hooks for workspace management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createWorkspace,
  deleteWorkspace,
  getDefaultWorkspaceId,
  getWorkspace,
  listWorkspaces,
  updateWorkspace,
} from "@/api/client";
import type {
  CreateWorkspaceParams,
  ListWorkspacesParams,
  UpdateWorkspaceParams,
} from "@/api/types";

// Query keys
export const workspaceKeys = {
  all: ["workspaces"] as const,
  lists: () => [...workspaceKeys.all, "list"] as const,
  list: (params: ListWorkspacesParams) =>
    [...workspaceKeys.lists(), params] as const,
  details: () => [...workspaceKeys.all, "detail"] as const,
  detail: (id: string) => [...workspaceKeys.details(), id] as const,
  defaultId: () => [...workspaceKeys.all, "default"] as const,
};

// Query hooks

export function useWorkspaces(params: ListWorkspacesParams = {}) {
  return useQuery({
    queryKey: workspaceKeys.list(params),
    queryFn: () => listWorkspaces(params),
  });
}

export function useWorkspace(workspaceId: string) {
  return useQuery({
    queryKey: workspaceKeys.detail(workspaceId),
    queryFn: () => getWorkspace(workspaceId),
    enabled: !!workspaceId,
  });
}

export function useDefaultWorkspaceId() {
  return useQuery({
    queryKey: workspaceKeys.defaultId(),
    queryFn: () => getDefaultWorkspaceId(),
  });
}

// Mutation hooks

export function useCreateWorkspace() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateWorkspaceParams) => createWorkspace(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.lists() });
    },
  });
}

export function useUpdateWorkspace() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      workspaceId,
      params,
    }: {
      workspaceId: string;
      params: UpdateWorkspaceParams;
    }) => updateWorkspace(workspaceId, params),
    onSuccess: (_, variables) => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.lists() });
      queryClient.invalidateQueries({
        queryKey: workspaceKeys.detail(variables.workspaceId),
      });
    },
  });
}

export function useDeleteWorkspace() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (workspaceId: string) => deleteWorkspace(workspaceId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: workspaceKeys.lists() });
    },
  });
}
