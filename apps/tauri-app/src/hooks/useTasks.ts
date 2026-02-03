// React Query hooks for task management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  approveTask,
  createCompositeTask,
  createUnitTask,
  getAgentTask,
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

// Query keys
export const taskKeys = {
  all: ["tasks"] as const,
  lists: () => [...taskKeys.all, "list"] as const,
  list: (params: ListTasksParams) => [...taskKeys.lists(), params] as const,
  details: () => [...taskKeys.all, "detail"] as const,
  detail: (id: string) => [...taskKeys.details(), id] as const,
  agentTasks: () => [...taskKeys.all, "agentTask"] as const,
  agentTask: (id: string) => [...taskKeys.agentTasks(), id] as const,
};

// Query hooks

export function useTasks(params: ListTasksParams = {}) {
  return useQuery({
    queryKey: taskKeys.list(params),
    queryFn: () => listTasks(params),
  });
}

export function useTask(taskId: string) {
  return useQuery({
    queryKey: taskKeys.detail(taskId),
    queryFn: () => getTask(taskId),
    enabled: !!taskId,
  });
}

export function useAgentTask(agentTaskId: string) {
  return useQuery({
    queryKey: taskKeys.agentTask(agentTaskId),
    queryFn: () => getAgentTask(agentTaskId),
    enabled: !!agentTaskId,
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
