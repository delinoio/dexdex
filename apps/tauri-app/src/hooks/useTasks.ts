// React Query hooks for task management
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  approveTask,
  cancelTask,
  commitToLocal,
  createCompositeTask,
  createPr,
  createUnitTask,
  deleteTask,
  dismissApproval,
  getCompositeTaskNodes,
  getTask,
  listTasks,
  rejectTask,
  requestChanges,
  updatePlanWithPrompt,
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
  compositeNodes: (compositeTaskId: string) =>
    [...taskKeys.all, "compositeNodes", compositeTaskId] as const,
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

export function useCompositeTaskNodes(compositeTaskId: string) {
  return useQuery({
    queryKey: taskKeys.compositeNodes(compositeTaskId),
    queryFn: () => getCompositeTaskNodes(compositeTaskId),
    enabled: !!compositeTaskId,
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
      // Invalidate composite nodes so the task graph refetches after approval
      // (approval creates CompositeTaskNode records in the backend)
      queryClient.invalidateQueries({ queryKey: taskKeys.compositeNodes(taskId) });
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

export function useDeleteTask() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => deleteTask(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useUpdatePlan() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ taskId, prompt }: { taskId: string; prompt: string }) =>
      updatePlanWithPrompt(taskId, prompt),
    onSuccess: (_data, { taskId }) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useDismissApproval() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => dismissApproval(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useCreatePr() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => createPr(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}

export function useCommitToLocal() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (taskId: string) => commitToLocal(taskId),
    onSuccess: (_data, taskId) => {
      queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
      queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
    },
  });
}
