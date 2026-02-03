// React Query hooks for repository management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  addRepository,
  listRepositories,
  removeRepository,
} from "@/api/client";
import type { AddRepositoryParams, ListRepositoriesParams } from "@/api/types";
import { repositoryGroupKeys } from "./useRepositoryGroups";

// Query keys
export const repositoryKeys = {
  all: ["repositories"] as const,
  lists: () => [...repositoryKeys.all, "list"] as const,
  list: (params: ListRepositoriesParams) =>
    [...repositoryKeys.lists(), params] as const,
  details: () => [...repositoryKeys.all, "detail"] as const,
  detail: (id: string) => [...repositoryKeys.details(), id] as const,
};

// Query hooks

export function useRepositories(params: ListRepositoriesParams = {}) {
  return useQuery({
    queryKey: repositoryKeys.list(params),
    queryFn: () => listRepositories(params),
  });
}

// Mutation hooks

export function useAddRepository() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: AddRepositoryParams) => addRepository(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: repositoryKeys.lists() });
    },
  });
}

export function useRemoveRepository() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (repositoryId: string) => removeRepository(repositoryId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: repositoryKeys.lists() });
      // Also invalidate repository groups since the backend removes
      // the deleted repository from all groups
      queryClient.invalidateQueries({ queryKey: repositoryGroupKeys.lists() });
    },
  });
}
