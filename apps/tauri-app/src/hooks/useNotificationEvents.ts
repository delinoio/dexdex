// Hook that listens to Tauri backend events and populates the notification center.
// Also sends desktop notifications when the window is not focused.
import { useEffect, useRef } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import {
  useNotificationCenterStore,
  NotificationCategory,
} from "@/stores/notificationCenterStore";
import type {
  TaskStatusChangedEvent,
  TaskCompletedEvent,
  TtyInputRequestEvent,
} from "@/api/types";

export function useNotificationEvents(): void {
  const initialized = useRef(false);

  useEffect(() => {
    if (initialized.current) return;
    initialized.current = true;

    const unlisteners: UnlistenFn[] = [];
    const { addNotification } = useNotificationCenterStore.getState();

    async function sendDesktopNotification(title: string, body: string) {
      try {
        if (!document.hasFocus()) {
          const granted = await isPermissionGranted();
          if (granted) {
            sendNotification({ title, body });
          }
        }
      } catch {
        // Not in Tauri context or permission not granted
      }
    }

    async function setup() {
      try {
        // Listen for task status changes
        const unlistenStatus = await listen<TaskStatusChangedEvent>(
          "task-status-changed",
          (event) => {
            const { taskId, taskType, newStatus } = event.payload;

            if (newStatus === "in_review") {
              const title = "Task ready for review";
              const message = `Task ${taskId} is waiting for your review.`;
              addNotification({
                category: NotificationCategory.TaskReviewReady,
                title,
                message,
                taskId,
                taskType,
              });
              sendDesktopNotification(title, message);
            }

            if (newStatus === "pending_approval") {
              const title = "Plan ready for approval";
              const message = `A plan for task ${taskId} needs your approval.`;
              addNotification({
                category: NotificationCategory.PlanApproval,
                title,
                message,
                taskId,
                taskType,
              });
              sendDesktopNotification(title, message);
            }

            if (newStatus === "failed") {
              const title = "Task failed";
              const message = `Task ${taskId} has failed.`;
              addNotification({
                category: NotificationCategory.TaskFailed,
                title,
                message,
                taskId,
                taskType,
              });
              sendDesktopNotification(title, message);
            }
          }
        );
        unlisteners.push(unlistenStatus);

        // Listen for task completed events
        const unlistenCompleted = await listen<TaskCompletedEvent>(
          "task-completed",
          (event) => {
            const { taskId, taskType, success, error } = event.payload;

            if (success) {
              const title = "Task completed";
              const message = `Task ${taskId} completed successfully.`;
              addNotification({
                category: NotificationCategory.TaskCompleted,
                title,
                message,
                taskId,
                taskType,
              });
              sendDesktopNotification(title, message);
            } else if (error) {
              const title = "Task failed";
              const message = error;
              addNotification({
                category: NotificationCategory.TaskFailed,
                title,
                message,
                taskId,
                taskType,
              });
              sendDesktopNotification(title, message);
            }
          }
        );
        unlisteners.push(unlistenCompleted);

        // Listen for TTY input request events
        const unlistenTty = await listen<TtyInputRequestEvent>(
          "tty-input-request",
          (event) => {
            const { taskId, question } = event.payload;
            const title = "Agent is asking a question";
            const message =
              question.length > 200
                ? `${question.slice(0, 197)}...`
                : question;
            addNotification({
              category: NotificationCategory.TtyInputRequest,
              title,
              message,
              taskId,
              taskType: "unit_task",
            });
            sendDesktopNotification(title, message);
          }
        );
        unlisteners.push(unlistenTty);
      } catch {
        // Not in Tauri context (browser dev mode)
      }
    }

    setup();

    return () => {
      for (const unlisten of unlisteners) {
        unlisten();
      }
    };
  }, []);
}
