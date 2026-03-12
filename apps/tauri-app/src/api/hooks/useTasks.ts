import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  UnitTask,
  ListTasksRequest,
  ListTasksResponse,
  GetTaskResponse,
  CreateTaskRequest,
  CreateTaskResponse,
  ApproveSubTaskRequest,
  RequestChangesRequest,
  ApprovePlanRequest,
  RevisePlanRequest,
  CreatePrRequest,
  StopTaskRequest,
  RetrySubTaskRequest,
} from '../types';

export function useTasks(params: ListTasksRequest = {}) {
  return useQuery({
    queryKey: ['tasks', params],
    queryFn: () =>
      rpcCall<ListTasksRequest, ListTasksResponse>('TaskService', 'List', {
        limit: 100,
        offset: 0,
        ...params,
      }),
  });
}

export function useTask(taskId: string) {
  return useQuery({
    queryKey: ['tasks', taskId],
    queryFn: () =>
      rpcCall<{ taskId: string }, GetTaskResponse>('TaskService', 'Get', { taskId }),
    enabled: !!taskId,
  });
}

export function useCreateTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (params: CreateTaskRequest) =>
      rpcCall<CreateTaskRequest, CreateTaskResponse>('TaskService', 'Create', params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

export function useDeleteTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (taskId: string) =>
      rpcCall<{ taskId: string }, Record<string, never>>('TaskService', 'Delete', { taskId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

export function useStopTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (taskId: string) =>
      rpcCall<StopTaskRequest, Record<string, never>>('TaskService', 'Stop', { taskId }),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: ['tasks', taskId] });
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

export function useApproveSubTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (subTaskId: string) =>
      rpcCall<ApproveSubTaskRequest, Record<string, never>>('SubTaskService', 'Approve', {
        subTaskId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
      queryClient.invalidateQueries({ queryKey: ['subtasks'] });
    },
  });
}

export function useRequestChanges() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ subTaskId, feedback }: { subTaskId: string; feedback: string }) =>
      rpcCall<RequestChangesRequest, Record<string, never>>('SubTaskService', 'RequestChanges', {
        subTaskId,
        feedback,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
      queryClient.invalidateQueries({ queryKey: ['subtasks'] });
    },
  });
}

export function useApprovePlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (subTaskId: string) =>
      rpcCall<ApprovePlanRequest, Record<string, never>>('SubTaskService', 'ApprovePlan', {
        subTaskId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subtasks'] });
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

export function useRevisePlan() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ subTaskId, feedback }: { subTaskId: string; feedback: string }) =>
      rpcCall<RevisePlanRequest, Record<string, never>>('SubTaskService', 'RevisePlan', {
        subTaskId,
        feedback,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subtasks'] });
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

export function useCreatePr() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (taskId: string) =>
      rpcCall<CreatePrRequest, { prUrl: string }>('TaskService', 'CreatePr', { taskId }),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: ['tasks', taskId] });
      queryClient.invalidateQueries({ queryKey: ['pr-trackings'] });
    },
  });
}

export function useRetrySubTask() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (subTaskId: string) =>
      rpcCall<RetrySubTaskRequest, Record<string, never>>('SubTaskService', 'Retry', {
        subTaskId,
      }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['subtasks'] });
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
    },
  });
}

// Helper: get one UnitTask by id from cache
export function useTaskById(taskId: string): UnitTask | undefined {
  const queryClient = useQueryClient();
  const cached = queryClient.getQueryData<GetTaskResponse>(['tasks', taskId]);
  return cached?.task;
}
