// Full-page notifications list
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { useNotifications, useMarkNotificationRead, useMarkAllNotificationsRead } from "@/api/hooks/useNotifications";
import { useUiStore } from "@/stores/uiStore";
import type { Notification, NotificationType } from "@/api/types";

function formatNotificationType(type: NotificationType): string {
  switch (type) {
    case "task_action_required":
      return "Action Required";
    case "plan_action_required":
      return "Plan Approval";
    case "pr_review_activity":
      return "PR Review";
    case "pr_ci_failure":
      return "CI Failure";
    case "agent_session_failed":
      return "Session Failed";
    default:
      return type;
  }
}

function NotificationRow({ notification }: { notification: Notification }) {
  const navigate = useNavigate();
  const markRead = useMarkNotificationRead();
  const isRead = !!notification.readAt;

  const handleClick = () => {
    if (!isRead) {
      markRead.mutate(notification.id);
    }
    if (notification.deepLink) {
      navigate(notification.deepLink);
    }
  };

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={handleClick}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          handleClick();
        }
      }}
      className={cn(
        "group flex items-center gap-4 rounded-lg border px-4 py-3 cursor-pointer transition-colors",
        isRead
          ? "border-[hsl(var(--border))] bg-transparent"
          : "border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
      )}
    >
      {/* Unread indicator */}
      <div className="shrink-0">
        {!isRead ? (
          <span className="inline-block h-2.5 w-2.5 rounded-full bg-[hsl(var(--primary))]" />
        ) : (
          <span className="inline-block h-2.5 w-2.5" />
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <span className="text-xs font-medium text-[hsl(var(--muted-foreground))]">
            {formatNotificationType(notification.notificationType)}
          </span>
        </div>
        <p className={cn("text-sm truncate", !isRead && "font-medium")}>
          {notification.title}
        </p>
        {notification.body && (
          <p className="text-xs text-[hsl(var(--muted-foreground))] mt-0.5 line-clamp-1">
            {notification.body}
          </p>
        )}
      </div>

      {/* Time */}
      <span className="text-xs text-[hsl(var(--muted-foreground))] shrink-0">
        <FormattedDateTime date={notification.createdAt} />
      </span>
    </div>
  );
}

export function Notifications() {
  const currentWorkspaceId = useUiStore((s) => s.currentWorkspaceId);
  const { data, isLoading } = useNotifications(currentWorkspaceId ?? "");
  const markAllRead = useMarkAllNotificationsRead();

  const notifications = data?.notifications ?? [];
  const unreadCount = notifications.filter((n) => !n.readAt).length;

  if (!currentWorkspaceId) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-[hsl(var(--muted-foreground))]">
          Select a workspace to view notifications.
        </p>
      </div>
    );
  }

  return (
    <div className="h-full overflow-y-auto">
      <div className="mx-auto max-w-3xl p-6">
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <div>
            <h1 className="text-xl font-semibold">Notifications</h1>
            {unreadCount > 0 && (
              <p className="text-sm text-[hsl(var(--muted-foreground))] mt-1">
                {unreadCount} unread
              </p>
            )}
          </div>
          {unreadCount > 0 && (
            <button
              onClick={() => markAllRead.mutate(currentWorkspaceId)}
              className="rounded-md px-3 py-1.5 text-xs font-medium text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--muted))] hover:text-[hsl(var(--foreground))]"
            >
              Mark all as read
            </button>
          )}
        </div>

        {/* List */}
        {isLoading ? (
          <p className="text-sm text-[hsl(var(--muted-foreground))]">Loading...</p>
        ) : notifications.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-16 text-[hsl(var(--muted-foreground))]">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="48"
              height="48"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="mb-4 opacity-40"
            >
              <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
              <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
            </svg>
            <p className="text-sm">No notifications yet</p>
            <p className="text-xs mt-1">
              Notifications about tasks, reviews, and agent activity will appear here.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {notifications.map((notification) => (
              <NotificationRow key={notification.id} notification={notification} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
