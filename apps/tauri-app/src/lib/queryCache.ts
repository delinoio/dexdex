// Utilities for optimistic cache updates on task mutations.
//
// When a subtask operation (Create PR, Commit to Local, Request Changes) is
// triggered, the backend transitions the unit task to InProgress before
// spawning the agent.  Optimistically updating the cache ensures the UI
// reflects the new status immediately instead of waiting for a refetch.

import type { QueryClient } from "@tanstack/react-query";
import { UnitTaskStatus, type ListTasksResult, type TaskResponse } from "@/api/types";
import { taskKeys } from "@/hooks/useTasks";

/**
 * Optimistically updates the unit task status in both the detail and list
 * caches.  Returns the previous cache values so the caller can roll back on
 * error via `onError`.
 */
export function setUnitTaskStatusInCache(
  queryClient: QueryClient,
  taskId: string,
  newStatus: UnitTaskStatus,
): { previousDetail: TaskResponse | undefined; previousLists: [readonly unknown[], ListTasksResult | undefined][] } {
  // Snapshot & update detail cache
  const detailKey = taskKeys.detail(taskId);
  const previousDetail = queryClient.getQueryData<TaskResponse>(detailKey);
  if (previousDetail?.unitTask) {
    queryClient.setQueryData<TaskResponse>(detailKey, {
      ...previousDetail,
      unitTask: { ...previousDetail.unitTask, status: newStatus },
    });
  }

  // Snapshot & update all list caches
  const previousLists: [readonly unknown[], ListTasksResult | undefined][] = [];
  const listQueries = queryClient.getQueriesData<ListTasksResult>({ queryKey: taskKeys.lists() });
  for (const [key, data] of listQueries) {
    previousLists.push([key, data]);
    if (data) {
      queryClient.setQueryData<ListTasksResult>(key, {
        ...data,
        unitTasks: data.unitTasks.map((t) =>
          t.id === taskId ? { ...t, status: newStatus } : t
        ),
      });
    }
  }

  return { previousDetail, previousLists };
}

/**
 * Rolls back optimistic updates using the snapshots returned by
 * `setUnitTaskStatusInCache`.
 */
export function rollbackUnitTaskStatusInCache(
  queryClient: QueryClient,
  taskId: string,
  snapshot: { previousDetail: TaskResponse | undefined; previousLists: [readonly unknown[], ListTasksResult | undefined][] },
): void {
  if (snapshot.previousDetail !== undefined) {
    queryClient.setQueryData(taskKeys.detail(taskId), snapshot.previousDetail);
  }
  for (const [key, data] of snapshot.previousLists) {
    if (data !== undefined) {
      queryClient.setQueryData(key, data);
    }
  }
}
