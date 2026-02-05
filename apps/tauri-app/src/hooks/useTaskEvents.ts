// Event-driven query invalidation for task state changes.
//
// Listens to Tauri events (`task-status-changed`, `task-completed`) and
// automatically invalidates the relevant react-query caches so the UI
// stays in sync without polling.

import { useEffect } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useQueryClient } from "@tanstack/react-query";
import { taskKeys } from "@/hooks/useTasks";
import type { TaskStatusChangedEvent, TaskCompletedEvent } from "@/api/types";

/**
 * Listens for task lifecycle events and invalidates react-query caches.
 *
 * Mount this hook once near the top of your component tree (e.g., in your
 * layout or App component) so that task list and detail queries are
 * refreshed automatically whenever the backend emits status changes.
 */
export function useTaskEvents() {
  const queryClient = useQueryClient();

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    const setup = async () => {
      // When any task status changes, invalidate its detail and the task list
      const unlistenStatus = await listen<TaskStatusChangedEvent>(
        "task-status-changed",
        (event) => {
          const { taskId } = event.payload;
          queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
          queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
        },
      );
      unlisteners.push(unlistenStatus);

      // When a task completes, invalidate its detail and the task list
      const unlistenCompleted = await listen<TaskCompletedEvent>(
        "task-completed",
        (event) => {
          const { taskId } = event.payload;
          queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
          queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
        },
      );
      unlisteners.push(unlistenCompleted);
    };

    setup().catch((err) => {
      console.error("Failed to set up task event listeners:", err);
    });

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, [queryClient]);
}
