// Shared utility functions for notification UI components
import {
  NotificationCategory,
  type PersistentNotification,
} from "@/stores/notificationCenterStore";

export function categoryLabel(category: NotificationCategory): string {
  switch (category) {
    case NotificationCategory.TaskReviewReady:
      return "Review Ready";
    case NotificationCategory.PlanApproval:
      return "Plan Approval";
    case NotificationCategory.TaskFailed:
      return "Task Failed";
    case NotificationCategory.TtyInputRequest:
      return "Agent Question";
    case NotificationCategory.TaskCompleted:
      return "Task Completed";
  }
}

export function categoryColor(category: NotificationCategory): string {
  switch (category) {
    case NotificationCategory.TaskReviewReady:
      return "bg-blue-500";
    case NotificationCategory.PlanApproval:
      return "bg-yellow-500";
    case NotificationCategory.TaskFailed:
      return "bg-red-500";
    case NotificationCategory.TtyInputRequest:
      return "bg-purple-500";
    case NotificationCategory.TaskCompleted:
      return "bg-green-500";
  }
}

export function formatTimeAgo(timestamp: number): string {
  const diff = Date.now() - timestamp;
  const seconds = Math.floor(diff / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);

  if (days > 0) return `${days}d ago`;
  if (hours > 0) return `${hours}h ago`;
  if (minutes > 0) return `${minutes}m ago`;
  return "just now";
}

export function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  const now = new Date();
  const diff = now.getTime() - timestamp;
  const days = Math.floor(diff / (1000 * 60 * 60 * 24));

  if (days === 0) {
    return date.toLocaleTimeString(undefined, {
      hour: "2-digit",
      minute: "2-digit",
    });
  }
  if (days === 1) return "Yesterday";
  if (days < 7) return `${days} days ago`;
  return date.toLocaleDateString();
}

/**
 * Build the route path for a notification's associated task.
 * Returns undefined if the notification does not have taskId/taskType.
 */
export function getNotificationPath(
  notification: PersistentNotification
): string | undefined {
  if (!notification.taskId || !notification.taskType) return undefined;
  return notification.taskType === "unit_task"
    ? `/unit-tasks/${notification.taskId}`
    : `/composite-tasks/${notification.taskId}`;
}
