// Full-page notifications list
import { useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import {
  useNotificationCenterStore,
  type PersistentNotification,
} from "@/stores/notificationCenterStore";
import {
  categoryLabel,
  categoryColor,
  formatTime,
  getNotificationPath,
} from "@/components/notifications/utils";

function NotificationRow({
  notification,
  onNavigate,
}: {
  notification: PersistentNotification;
  onNavigate: (notification: PersistentNotification) => void;
}) {
  const { removeNotification, markAsRead } = useNotificationCenterStore();

  const handleClick = useCallback(() => {
    markAsRead(notification.id);
    onNavigate(notification);
  }, [notification, markAsRead, onNavigate]);

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
        notification.read
          ? "border-[hsl(var(--border))] bg-transparent"
          : "border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
      )}
    >
      {/* Unread indicator */}
      <div className="shrink-0">
        {!notification.read ? (
          <span
            className={cn(
              "inline-block h-2.5 w-2.5 rounded-full",
              categoryColor(notification.category)
            )}
          />
        ) : (
          <span className="inline-block h-2.5 w-2.5" />
        )}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-0.5">
          <span className="text-xs font-medium text-[hsl(var(--muted-foreground))]">
            {categoryLabel(notification.category)}
          </span>
        </div>
        <p
          className={cn(
            "text-sm truncate",
            !notification.read && "font-medium"
          )}
        >
          {notification.title}
        </p>
        {notification.message && (
          <p className="text-xs text-[hsl(var(--muted-foreground))] mt-0.5 line-clamp-1">
            {notification.message}
          </p>
        )}
      </div>

      {/* Time */}
      <span className="text-xs text-[hsl(var(--muted-foreground))] shrink-0">
        {formatTime(notification.createdAt)}
      </span>

      {/* Actions */}
      <div className="flex items-center gap-1 shrink-0 opacity-0 group-hover:opacity-100 transition-opacity">
        {!notification.read && (
          <button
            onClick={(e) => {
              e.stopPropagation();
              markAsRead(notification.id);
            }}
            className="rounded-md p-1.5 hover:bg-[hsl(var(--muted))] text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
            title="Mark as read"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="16"
              height="16"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
          </button>
        )}
        <button
          onClick={(e) => {
            e.stopPropagation();
            removeNotification(notification.id);
          }}
          className="rounded-md p-1.5 hover:bg-[hsl(var(--muted))] text-[hsl(var(--muted-foreground))] hover:text-red-500"
          title="Delete notification"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          >
            <path d="M3 6h18" />
            <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6" />
            <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2" />
          </svg>
        </button>
      </div>
    </div>
  );
}

export function Notifications() {
  const navigate = useNavigate();
  const { notifications, markAllAsRead, clearAll } =
    useNotificationCenterStore();
  const unreadCount = useNotificationCenterStore((s) => s.getUnreadCount());

  const handleNavigate = useCallback(
    (notification: PersistentNotification) => {
      const path = getNotificationPath(notification);
      if (path) {
        navigate(path);
      }
    },
    [navigate]
  );

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
          {notifications.length > 0 && (
            <div className="flex items-center gap-2">
              <button
                onClick={markAllAsRead}
                className="rounded-md px-3 py-1.5 text-xs font-medium text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--muted))] hover:text-[hsl(var(--foreground))]"
              >
                Mark all as read
              </button>
              <button
                onClick={clearAll}
                className="rounded-md px-3 py-1.5 text-xs font-medium text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--muted))] hover:text-red-500"
              >
                Clear all
              </button>
            </div>
          )}
        </div>

        {/* List */}
        {notifications.length === 0 ? (
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
              Notifications about tasks, reviews, and agent activity will appear
              here.
            </p>
          </div>
        ) : (
          <div className="space-y-2">
            {notifications.map((notification) => (
              <NotificationRow
                key={notification.id}
                notification={notification}
                onNavigate={handleNavigate}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
