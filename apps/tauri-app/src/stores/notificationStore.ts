// Notification state management with Zustand
import { create } from "zustand";

export type NotificationType = "success" | "error" | "warning" | "info";

export interface Notification {
  id: string;
  type: NotificationType;
  title: string;
  message?: string;
  duration?: number;
  createdAt: number;
}

interface NotificationState {
  notifications: Notification[];
  addNotification: (
    notification: Omit<Notification, "id" | "createdAt">
  ) => string;
  removeNotification: (id: string) => void;
  clearAllNotifications: () => void;
}

let notificationIdCounter = 0;

export const useNotificationStore = create<NotificationState>((set, get) => ({
  notifications: [],

  addNotification: (notification) => {
    const id = `notification-${++notificationIdCounter}`;
    const newNotification: Notification = {
      ...notification,
      id,
      createdAt: Date.now(),
      duration: notification.duration ?? 5000,
    };

    set((state) => ({
      notifications: [...state.notifications, newNotification],
    }));

    // Auto-remove after duration
    if (newNotification.duration && newNotification.duration > 0) {
      setTimeout(() => {
        get().removeNotification(id);
      }, newNotification.duration);
    }

    return id;
  },

  removeNotification: (id) =>
    set((state) => ({
      notifications: state.notifications.filter((n) => n.id !== id),
    })),

  clearAllNotifications: () => set({ notifications: [] }),
}));

// Convenience functions for creating notifications
export const notify = {
  success: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({
      type: "success",
      title,
      message,
    }),
  error: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({
      type: "error",
      title,
      message,
    }),
  warning: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({
      type: "warning",
      title,
      message,
    }),
  info: (title: string, message?: string) =>
    useNotificationStore.getState().addNotification({
      type: "info",
      title,
      message,
    }),
};
