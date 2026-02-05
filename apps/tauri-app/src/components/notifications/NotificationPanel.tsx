// Notification sidebar panel - slides in from the right
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
  formatTimeAgo,
  getNotificationPath,
} from "./utils";

function NotificationItem({
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

  const handleMarkAsRead = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      markAsRead(notification.id);
    },
    [notification.id, markAsRead]
  );

  const handleDelete = useCallback(
    (e: React.MouseEvent) => {
      e.stopPropagation();
      removeNotification(notification.id);
    },
    [notification.id, removeNotification]
  );

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
        "group flex flex-col gap-1 rounded-md border px-3 py-2 cursor-pointer transition-colors",
        notification.read
          ? "border-[hsl(var(--border))] bg-transparent opacity-70"
          : "border-[hsl(var(--border))] bg-[hsl(var(--muted))]"
      )}
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex items-center gap-2 min-w-0">
          <span
            className={cn(
              "inline-block h-2 w-2 shrink-0 rounded-full",
              categoryColor(notification.category)
            )}
          />
          <span className="text-xs text-[hsl(var(--muted-foreground))]">
            {categoryLabel(notification.category)}
          </span>
        </div>
        <span className="text-xs text-[hsl(var(--muted-foreground))] shrink-0">
          {formatTimeAgo(notification.createdAt)}
        </span>
      </div>

      <p className="text-sm font-medium truncate">{notification.title}</p>

      {notification.message && (
        <p className="text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
          {notification.message}
        </p>
      )}

      <div className="flex items-center gap-1 mt-1 opacity-0 group-hover:opacity-100 transition-opacity">
        {!notification.read && (
          <button
            onClick={handleMarkAsRead}
            className="text-xs text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))] px-1.5 py-0.5 rounded hover:bg-[hsl(var(--muted))]"
          >
            Mark as read
          </button>
        )}
        <button
          onClick={handleDelete}
          className="text-xs text-[hsl(var(--muted-foreground))] hover:text-red-500 px-1.5 py-0.5 rounded hover:bg-[hsl(var(--muted))]"
        >
          Delete
        </button>
      </div>
    </div>
  );
}

export function NotificationPanel() {
  const navigate = useNavigate();
  const { notifications, isOpen, setOpen, markAllAsRead, clearAll } =
    useNotificationCenterStore();
  const unreadCount = useNotificationCenterStore((s) => s.getUnreadCount());

  const handleNavigate = useCallback(
    (notification: PersistentNotification) => {
      const path = getNotificationPath(notification);
      if (path) {
        navigate(path);
        setOpen(false);
      }
    },
    [navigate, setOpen]
  );

  const handleViewAll = useCallback(() => {
    navigate("/notifications");
    setOpen(false);
  }, [navigate, setOpen]);

  if (!isOpen) return null;

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 z-40 bg-black/20"
        onClick={() => setOpen(false)}
        aria-hidden="true"
      />

      {/* Panel */}
      <aside
        className="fixed right-0 top-0 z-50 flex h-full w-80 flex-col border-l border-[hsl(var(--border))] bg-[hsl(var(--background))] shadow-lg"
        role="dialog"
        aria-label="Notifications"
      >
        {/* Header */}
        <div className="flex h-14 items-center justify-between border-b border-[hsl(var(--border))] px-4">
          <div className="flex items-center gap-2">
            <h2 className="text-sm font-semibold">Notifications</h2>
            {unreadCount > 0 && (
              <span className="inline-flex h-5 min-w-5 items-center justify-center rounded-full bg-[hsl(var(--primary))] px-1.5 text-xs font-medium text-[hsl(var(--primary-foreground))]">
                {unreadCount}
              </span>
            )}
          </div>
          <button
            onClick={() => setOpen(false)}
            className="rounded-md p-1 hover:bg-[hsl(var(--muted))]"
            aria-label="Close notifications"
          >
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            >
              <path d="M18 6 6 18" />
              <path d="m6 6 12 12" />
            </svg>
          </button>
        </div>

        {/* Actions bar */}
        {notifications.length > 0 && (
          <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-4 py-2">
            <button
              onClick={markAllAsRead}
              className="text-xs text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
            >
              Mark all as read
            </button>
            <button
              onClick={clearAll}
              className="text-xs text-[hsl(var(--muted-foreground))] hover:text-red-500"
            >
              Clear all
            </button>
          </div>
        )}

        {/* Notification list */}
        <div className="flex-1 overflow-y-auto p-2 space-y-2">
          {notifications.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-[hsl(var(--muted-foreground))]">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="32"
                height="32"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="mb-2 opacity-50"
              >
                <path d="M6 8a6 6 0 0 1 12 0c0 7 3 9 3 9H3s3-2 3-9" />
                <path d="M10.3 21a1.94 1.94 0 0 0 3.4 0" />
              </svg>
              <p className="text-sm">No notifications</p>
            </div>
          ) : (
            notifications.map((notification) => (
              <NotificationItem
                key={notification.id}
                notification={notification}
                onNavigate={handleNavigate}
              />
            ))
          )}
        </div>

        {/* Footer with View All link */}
        {notifications.length > 0 && (
          <div className="border-t border-[hsl(var(--border))] p-2">
            <button
              onClick={handleViewAll}
              className="w-full rounded-md py-2 text-center text-xs font-medium text-[hsl(var(--primary))] hover:bg-[hsl(var(--muted))]"
            >
              View all notifications
            </button>
          </div>
        )}
      </aside>
    </>
  );
}
