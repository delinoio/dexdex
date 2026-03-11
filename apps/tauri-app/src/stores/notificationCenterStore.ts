// Persistent notification center store with Zustand
// This store manages persistent notifications (notification center / inbox),
// separate from the ephemeral toast notifications in notificationStore.ts.
import { create } from "zustand";
import { persist } from "zustand/middleware";

export enum NotificationCategory {
  TaskReviewReady = "task_review_ready",
  PlanApproval = "plan_approval",
  TaskFailed = "task_failed",
  TtyInputRequest = "tty_input_request",
  TaskCompleted = "task_completed",
}

export interface PersistentNotification {
  id: string;
  category: NotificationCategory;
  title: string;
  message: string;
  read: boolean;
  // Link target for click-to-navigate
  taskId?: string;
  taskType?: "unit_task" | "composite_task";
  createdAt: number;
}

interface NotificationCenterState {
  notifications: PersistentNotification[];
  isOpen: boolean;

  // Actions
  addNotification: (
    notification: Omit<PersistentNotification, "id" | "createdAt" | "read">
  ) => string;
  removeNotification: (id: string) => void;
  markAsRead: (id: string) => void;
  markAllAsRead: () => void;
  clearAll: () => void;
  setOpen: (open: boolean) => void;
  toggleOpen: () => void;

  // Computed-like helpers
  getUnreadCount: () => number;
}

let centerNotificationIdCounter = 0;

// Maximum number of notifications stored. When exceeded, oldest read
// notifications are trimmed first, then oldest unread if still over limit.
export const MAX_NOTIFICATIONS = 200;

export const useNotificationCenterStore = create<NotificationCenterState>()(
  persist(
    (set, get) => ({
      notifications: [],
      isOpen: false,

      addNotification: (notification) => {
        const id = `notif-${Date.now()}-${++centerNotificationIdCounter}`;
        const newNotification: PersistentNotification = {
          ...notification,
          id,
          read: false,
          createdAt: Date.now(),
        };

        set((state) => {
          let updated = [newNotification, ...state.notifications];

          // Trim if over limit: drop oldest read notifications first,
          // then oldest unread if still over.
          if (updated.length > MAX_NOTIFICATIONS) {
            const unread = updated.filter((n) => !n.read);
            const read = updated.filter((n) => n.read);
            // Keep all unread + as many recent read as fit within the limit
            const readSlots = Math.max(0, MAX_NOTIFICATIONS - unread.length);
            updated = [...unread, ...read.slice(0, readSlots)].sort(
              (a, b) => b.createdAt - a.createdAt
            );
            // If still over (too many unread), hard-cap at limit
            if (updated.length > MAX_NOTIFICATIONS) {
              updated = updated.slice(0, MAX_NOTIFICATIONS);
            }
          }

          return { notifications: updated };
        });

        return id;
      },

      removeNotification: (id) =>
        set((state) => ({
          notifications: state.notifications.filter((n) => n.id !== id),
        })),

      markAsRead: (id) =>
        set((state) => ({
          notifications: state.notifications.map((n) =>
            n.id === id ? { ...n, read: true } : n
          ),
        })),

      markAllAsRead: () =>
        set((state) => ({
          notifications: state.notifications.map((n) => ({ ...n, read: true })),
        })),

      clearAll: () => set({ notifications: [] }),

      setOpen: (open) => set({ isOpen: open }),

      toggleOpen: () => set((state) => ({ isOpen: !state.isOpen })),

      getUnreadCount: () => {
        return get().notifications.filter((n) => !n.read).length;
      },
    }),
    {
      name: "dexdex-notification-center",
      partialize: (state) => ({
        notifications: state.notifications,
      }),
    }
  )
);
