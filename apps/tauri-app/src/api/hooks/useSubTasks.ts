import { useQuery } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListSubTasksRequest,
  ListSubTasksResponse,
  GetSubTaskRequest,
  GetSubTaskResponse,
} from '../types';

export function useSubTasks(unitTaskId: string) {
  return useQuery({
    queryKey: ['subtasks', unitTaskId],
    queryFn: () =>
      rpcCall<ListSubTasksRequest, ListSubTasksResponse>('SubTaskService', 'List', {
        unitTaskId,
      }),
    enabled: !!unitTaskId,
    refetchInterval: 3000,
  });
}

export function useSubTask(subTaskId: string) {
  return useQuery({
    queryKey: ['subtasks', 'single', subTaskId],
    queryFn: () =>
      rpcCall<GetSubTaskRequest, GetSubTaskResponse>('SubTaskService', 'Get', { subTaskId }),
    enabled: !!subTaskId,
  });
}
