import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListRepositoriesRequest,
  ListRepositoriesResponse,
  ListRepositoryGroupsRequest,
  ListRepositoryGroupsResponse,
  Repository,
  RepositoryGroup,
} from '../types';

export function useRepositories(workspaceId?: string) {
  return useQuery({
    queryKey: ['repositories', workspaceId],
    queryFn: () =>
      rpcCall<ListRepositoriesRequest, ListRepositoriesResponse>('RepositoryService', 'List', {
        workspaceId,
        limit: 100,
        offset: 0,
      }),
  });
}

export function useRepositoryGroups(workspaceId?: string) {
  return useQuery({
    queryKey: ['repository-groups', workspaceId],
    queryFn: () =>
      rpcCall<ListRepositoryGroupsRequest, ListRepositoryGroupsResponse>(
        'RepositoryGroupService',
        'List',
        { workspaceId, limit: 100, offset: 0 }
      ),
  });
}

export function useAddRepository() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspaceId?: string; remoteUrl: string; defaultBranch?: string }) =>
      rpcCall<typeof params, { repository: Repository }>('RepositoryService', 'Add', params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });
}

export function useRemoveRepository() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (repositoryId: string) =>
      rpcCall<{ repositoryId: string }, Record<string, never>>('RepositoryService', 'Remove', {
        repositoryId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repositories'] });
    },
  });
}

export function useCreateRepositoryGroup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: { workspaceId?: string; name: string; repositoryIds: string[] }) =>
      rpcCall<typeof params, { group: RepositoryGroup }>('RepositoryGroupService', 'Create', params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repository-groups'] });
    },
  });
}

export function useUpdateRepositoryGroup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: {
      groupId: string;
      name?: string;
      repositoryIds?: string[];
    }) =>
      rpcCall<typeof params, { group: RepositoryGroup }>(
        'RepositoryGroupService',
        'Update',
        params
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repository-groups'] });
    },
  });
}

export function useDeleteRepositoryGroup() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (groupId: string) =>
      rpcCall<{ groupId: string }, Record<string, never>>('RepositoryGroupService', 'Delete', {
        groupId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['repository-groups'] });
    },
  });
}
