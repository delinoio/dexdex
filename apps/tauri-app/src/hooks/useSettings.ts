// React Query hooks for settings management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  getGlobalSettings,
  getRepositorySettings,
  updateGlobalSettings,
  updateRepositorySettings,
} from "@/api/client";
import type { GlobalSettings, RepositorySettings } from "@/api/types";

// Query keys
export const settingsKeys = {
  all: ["settings"] as const,
  global: () => [...settingsKeys.all, "global"] as const,
  repository: (id: string) => [...settingsKeys.all, "repository", id] as const,
};

// Query hooks

export function useGlobalSettings() {
  return useQuery({
    queryKey: settingsKeys.global(),
    queryFn: () => getGlobalSettings(),
  });
}

export function useRepositorySettings(repositoryId: string) {
  return useQuery({
    queryKey: settingsKeys.repository(repositoryId),
    queryFn: () => getRepositorySettings(repositoryId),
    enabled: !!repositoryId,
  });
}

// Mutation hooks

export function useUpdateGlobalSettings() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: Partial<GlobalSettings>) =>
      updateGlobalSettings(settings),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: settingsKeys.global() });
    },
  });
}

export function useUpdateRepositorySettings(repositoryId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (settings: Partial<RepositorySettings>) =>
      updateRepositorySettings(repositoryId, settings),
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: settingsKeys.repository(repositoryId),
      });
    },
  });
}
