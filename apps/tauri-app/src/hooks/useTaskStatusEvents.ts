// Hook that listens to task status Tauri events and invalidates react-query caches.
//
// This replaces any need for polling task status: the backend emits
// `task-status-changed` and `task-completed` events in real-time, and this
// hook ensures the UI queries are refreshed accordingly.
import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { taskKeys } from "@/hooks/useTasks";
import type { TaskStatusChangedEvent, TaskCompletedEvent } from "@/api/types";

/**
 * Listens for task lifecycle events from the Tauri backend and invalidates
 * the corresponding react-query caches so views update immediately.
 *
 * Should be mounted once at the app level (e.g. in `AppRoutes`).
 */
export function useTaskStatusEvents(): void {
  const queryClient = useQueryClient();

  useEffect(() => {
    const controller = new AbortController();
    const unlisteners: UnlistenFn[] = [];

    async function setup() {
      try {
        if (controller.signal.aborted) return;

        // When a task's status changes, invalidate its detail, list views,
        // and composite task nodes (for task graph updates on plan completion)
        const unlistenStatus = await listen<TaskStatusChangedEvent>(
          "task-status-changed",
          (event) => {
            const { taskId } = event.payload;
            queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
            queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
            queryClient.invalidateQueries({ queryKey: taskKeys.compositeNodes(taskId) });
          },
        );

        if (controller.signal.aborted) {
          unlistenStatus();
          return;
        }
        unlisteners.push(unlistenStatus);

        // When a task completes, invalidate its detail and the list views
        const unlistenCompleted = await listen<TaskCompletedEvent>(
          "task-completed",
          (event) => {
            const { taskId } = event.payload;
            queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
            queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
          },
        );

        if (controller.signal.aborted) {
          unlistenCompleted();
          return;
        }
        unlisteners.push(unlistenCompleted);
      } catch {
        // Not in Tauri context (browser dev mode)
      }
    }

    setup();

    return () => {
      controller.abort();
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, [queryClient]);
}
