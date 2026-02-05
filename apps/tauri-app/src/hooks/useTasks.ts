// React Query hooks for task management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  approveTask,
  cancelTask,
  createCompositeTask,
  createUnitTask,
  getCompositeTaskNodes,
  getTask,
  listTasks,
  rejectTask,
  requestChanges,
} from "@/api/client";
import type {
  CreateCompositeTaskParams,
  CreateUnitTaskParams,
  ListTasksParams,
} from "@/api/types";
import { CompositeTaskStatus, UnitTaskStatus } from "@/api/types";

/// Polling interval for active tasks (in milliseconds).
const ACTIVE_TASK_POLL_INTERVAL = 2000;

// Query keys
export const taskKeys = {
  all: ["tasks"] as const,
  lists: () => [...taskKeys.all, "list"] as const,
  list: (params: ListTasksParams) => [...taskKeys.lists(), params] as const,
  details: () => [...taskKeys.all, "detail"] as const,
  detail: (id: string) => [...taskKeys.details(), id] as const,
  compositeNodes: (compositeTaskId: string) =>
    [...taskKeys.all, "compositeNodes", compositeTaskId] as const,
};

// Query hooks

export function useTasks(params: ListTasksParams = {}) {
  return useQuery({
    queryKey: taskKeys.list(params),
    queryFn: () => listTasks(params),
    refetchInterval: (query) => {
      const data = query.state.data;
      // Poll while any task is in an active state
      const hasActiveUnit = data?.unitTasks?.some(
        (t) => t.status === UnitTaskStatus.InProgress
      );
      const hasActiveComposite = data?.compositeTasks?.some(
        (t) =>
          t.status === CompositeTaskStatus.Planning ||
          t.status === CompositeTaskStatus.InProgress
      );
      return hasActiveUnit || hasActiveComposite
        ? ACTIVE_TASK_POLL_INTERVAL
        : false;
    },
  });
}

export function useTask(taskId: string) {
  return useQuery({
    queryKey: taskKeys.detail(taskId),
    queryFn: () => getTask(taskId),
    enabled: !!taskId,
    refetchInterval: (query) => {
      const data = query.state.data;
      // Poll while task is in an active state
      const unitStatus = data?.unitTask?.status;
      const compositeStatus = data?.compositeTask?.status;
      const isActive =
        unitStatus === UnitTaskStatus.InProgress ||
        compositeStatus === CompositeTaskStatus.Planning ||
        compositeStatus === CompositeTaskStatus.InProgress;
      return isActive ? ACTIVE_TASK_POLL_INTERVAL : false;
    },
  });
}

export function useCompositeTaskNodes(compositeTaskId: string) {
  return useQuery({
    queryKey: taskKeys.compositeNodes(compositeTaskId),
    queryFn: () => getCompositeTaskNodes(compositeTaskId),
    enabled: !!compositeTaskId,
    refetchInterval: (query) => {
      // Poll while there are no nodes yet (plan may still be generating)
      const nodes = query.state.data?.nodes;
      return !nodes || nodes.length === 0 ? ACTIVE_TASK_POLL_INTERVAL : false;
    },
  });
}

// Mutation hooks

export function useCreateUnitTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateUnitTaskParams) => createUnitTask(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useCreateCompositeTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (params: CreateCompositeTaskParams) => createCompositeTask(params),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useApproveTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => approveTask(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
      queryClient.invalidateQueries({
        queryKey: taskKeys.compositeNodes(taskId),
      });
    },
  });
}

export function useRejectTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ taskId, reason }: { taskId: string; reason?: string }) =>
      rejectTask(taskId, reason),
    onSuccess: (_data, { taskId }) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useRequestChanges() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ taskId, feedback }: { taskId: string; feedback: string }) =>
      requestChanges(taskId, feedback),
    onSuccess: (_data, { taskId }) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useCancelTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => cancelTask(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}
