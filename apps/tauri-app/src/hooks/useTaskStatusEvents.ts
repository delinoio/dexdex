// Hook that listens to task status Tauri events and invalidates react-query caches.
//
// This replaces any need for polling task status: the backend emits
// `task-status-changed` and `task-completed` events in real-time, and this
// hook ensures the UI queries are refreshed accordingly.
import { useEffect, useRef } from "react";
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
  // Store unlisteners in a ref so the cleanup function can access them
  // even when setup() resolves after the effect cleanup runs (e.g. in
  // React 18 StrictMode).
  const unlistenersRef = useRef<UnlistenFn[]>([]);

  useEffect(() => {
    let cancelled = false;

    async function setup() {
      try {
        // When a task's status changes, invalidate its detail and the list views
        const unlistenStatus = await listen<TaskStatusChangedEvent>(
          "task-status-changed",
          (event) => {
            const { taskId } = event.payload;
            queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
            queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
          },
        );

        if (cancelled) {
          unlistenStatus();
          return;
        }
        unlistenersRef.current.push(unlistenStatus);

        // When a task completes, invalidate its detail and the list views
        const unlistenCompleted = await listen<TaskCompletedEvent>(
          "task-completed",
          (event) => {
            const { taskId } = event.payload;
            queryClient.invalidateQueries({ queryKey: taskKeys.detail(taskId) });
            queryClient.invalidateQueries({ queryKey: taskKeys.lists() });
          },
        );

        if (cancelled) {
          unlistenCompleted();
          return;
        }
        unlistenersRef.current.push(unlistenCompleted);
      } catch {
        // Not in Tauri context (browser dev mode)
      }
    }

    setup();

    return () => {
      cancelled = true;
      for (const unlisten of unlistenersRef.current) {
        unlisten();
      }
      unlistenersRef.current = [];
    };
  }, [queryClient]);
}
