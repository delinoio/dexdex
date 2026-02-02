// React Query hooks for repository group management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  createRepositoryGroup,
  deleteRepositoryGroup,
  getRepositoryGroup,
  listRepositoryGroups,
  updateRepositoryGroup,
} from "@/api/client";
import type {
  CreateRepositoryGroupParams,
  ListRepositoryGroupsParams,
  UpdateRepositoryGroupParams,
} from "@/api/types";

// Query keys
export const repositoryGroupKeys = {
  all: ["repositoryGroups"] as const,
  lists: () => [...repositoryGroupKeys.all, "list"] as const,
  list: (params: ListRepositoryGroupsParams) =>
    [...repositoryGroupKeys.lists(), params] as const,
  details: () => [...repositoryGroupKeys.all, "detail"] as const,
  detail: (id: string) => [...repositoryGroupKeys.details(), id] as const,
};

// Query hooks

export function useRepositoryGroups(params: ListRepositoryGroupsParams = {}) {
  return useQuery({
    queryKey: repositoryGroupKeys.list(params),
    queryFn: () => listRepositoryGroups(params),
  });
}

export function useRepositoryGroup(groupId: string) {
  return useQuery({
    queryKey: repositoryGroupKeys.detail(groupId),
    queryFn: () => getRepositoryGroup(groupId),
    enabled: !!groupId,
  });
}

// Mutation hooks

export function useCreateRepositoryGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateRepositoryGroupParams) =>
      createRepositoryGroup(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: repositoryGroupKeys.lists() });
    },
  });
}

export function useUpdateRepositoryGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({
      groupId,
      params,
    }: {
      groupId: string;
      params: UpdateRepositoryGroupParams;
    }) => updateRepositoryGroup(groupId, params),
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: repositoryGroupKeys.lists() });
      queryClient.invalidateQueries({
        queryKey: repositoryGroupKeys.detail(variables.groupId),
      });
    },
  });
}

export function useDeleteRepositoryGroup() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (groupId: string) => deleteRepositoryGroup(groupId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: repositoryGroupKeys.lists() });
    },
  });
}
