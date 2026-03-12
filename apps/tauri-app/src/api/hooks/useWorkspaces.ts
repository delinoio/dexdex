import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListWorkspacesRequest,
  ListWorkspacesResponse,
  GetWorkspaceRequest,
  GetWorkspaceResponse,
  CreateWorkspaceRequest,
  UpdateWorkspaceRequest,
  DeleteWorkspaceRequest,
  Workspace,
} from '../types';

export function useWorkspaces() {
  return useQuery({
    queryKey: ['workspaces'],
    queryFn: () =>
      rpcCall<ListWorkspacesRequest, ListWorkspacesResponse>('WorkspaceService', 'List', {
        limit: 100,
        offset: 0,
      }),
  });
}

export function useWorkspace(workspaceId: string) {
  return useQuery({
    queryKey: ['workspaces', workspaceId],
    queryFn: () =>
      rpcCall<GetWorkspaceRequest, GetWorkspaceResponse>('WorkspaceService', 'Get', {
        workspaceId,
      }),
    enabled: !!workspaceId,
  });
}

export function useCreateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: CreateWorkspaceRequest) =>
      rpcCall<CreateWorkspaceRequest, { workspace: Workspace }>('WorkspaceService', 'Create', params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workspaces'] });
    },
  });
}

export function useUpdateWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: UpdateWorkspaceRequest) =>
      rpcCall<UpdateWorkspaceRequest, { workspace: Workspace }>('WorkspaceService', 'Update', params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workspaces'] });
    },
  });
}

export function useDeleteWorkspace() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (workspaceId: string) =>
      rpcCall<DeleteWorkspaceRequest, Record<string, never>>('WorkspaceService', 'Delete', {
        workspaceId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['workspaces'] });
    },
  });
}
