import { describe, it, expect, beforeEach, vi } from "vitest";
import {
  useNotificationCenterStore,
  NotificationCategory,
} from "../notificationCenterStore";

describe("notificationCenterStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useNotificationCenterStore.setState({
      notifications: [],
      isOpen: false,
    });
  });

  describe("addNotification", () => {
    it("adds a notification to the store", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "Task ready for review",
        message: "Task 123 is ready.",
        taskId: "task-123",
        taskType: "unit_task",
      });

      const notifications =
        useNotificationCenterStore.getState().notifications;
      expect(notifications.length).toBe(1);
      expect(notifications[0].category).toBe(
        NotificationCategory.TaskReviewReady
      );
      expect(notifications[0].title).toBe("Task ready for review");
      expect(notifications[0].message).toBe("Task 123 is ready.");
      expect(notifications[0].taskId).toBe("task-123");
      expect(notifications[0].taskType).toBe("unit_task");
    });

    it("adds notifications in reverse chronological order (newest first)", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "First",
        message: "First notification",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "Second",
        message: "Second notification",
      });

      const notifications =
        useNotificationCenterStore.getState().notifications;
      expect(notifications[0].title).toBe("Second");
      expect(notifications[1].title).toBe("First");
    });

    it("marks notifications as unread by default", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskCompleted,
        title: "Done",
        message: "Task completed.",
      });

      const notification =
        useNotificationCenterStore.getState().notifications[0];
      expect(notification.read).toBe(false);
    });

    it("assigns unique IDs", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "B",
        message: "",
      });

      const notifications =
        useNotificationCenterStore.getState().notifications;
      expect(notifications[0].id).not.toBe(notifications[1].id);
    });

    it("sets createdAt timestamp", () => {
      const now = Date.now();
      vi.setSystemTime(now);

      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "Test",
        message: "",
      });

      const notification =
        useNotificationCenterStore.getState().notifications[0];
      expect(notification.createdAt).toBeGreaterThanOrEqual(now);

      vi.useRealTimers();
    });

    it("returns the notification ID", () => {
      const id = useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "Test",
        message: "",
      });

      expect(typeof id).toBe("string");
      expect(id).toContain("notif-");
    });
  });

  describe("removeNotification", () => {
    it("removes a notification by ID", () => {
      const id = useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "To remove",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "To keep",
        message: "",
      });

      useNotificationCenterStore.getState().removeNotification(id);

      const notifications =
        useNotificationCenterStore.getState().notifications;
      expect(notifications.length).toBe(1);
      expect(notifications[0].title).toBe("To keep");
    });

    it("does nothing if ID does not exist", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "Test",
        message: "",
      });

      useNotificationCenterStore
        .getState()
        .removeNotification("non-existent");

      expect(
        useNotificationCenterStore.getState().notifications.length
      ).toBe(1);
    });
  });

  describe("markAsRead", () => {
    it("marks a specific notification as read", () => {
      const id = useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "Unread",
        message: "",
      });

      expect(
        useNotificationCenterStore.getState().notifications[0].read
      ).toBe(false);

      useNotificationCenterStore.getState().markAsRead(id);

      expect(
        useNotificationCenterStore.getState().notifications[0].read
      ).toBe(true);
    });

    it("does not affect other notifications", () => {
      const id1 = useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "First",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "Second",
        message: "",
      });

      useNotificationCenterStore.getState().markAsRead(id1);

      const notifications =
        useNotificationCenterStore.getState().notifications;
      // Second was added after first, so it's at index 0 (newest first)
      expect(notifications[0].read).toBe(false);
      expect(notifications[1].read).toBe(true);
    });
  });

  describe("markAllAsRead", () => {
    it("marks all notifications as read", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "B",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskCompleted,
        title: "C",
        message: "",
      });

      useNotificationCenterStore.getState().markAllAsRead();

      const notifications =
        useNotificationCenterStore.getState().notifications;
      expect(notifications.every((n) => n.read)).toBe(true);
    });
  });

  describe("clearAll", () => {
    it("removes all notifications", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "B",
        message: "",
      });

      useNotificationCenterStore.getState().clearAll();

      expect(
        useNotificationCenterStore.getState().notifications.length
      ).toBe(0);
    });
  });

  describe("panel open/close", () => {
    it("toggles open state", () => {
      expect(useNotificationCenterStore.getState().isOpen).toBe(false);

      useNotificationCenterStore.getState().toggleOpen();
      expect(useNotificationCenterStore.getState().isOpen).toBe(true);

      useNotificationCenterStore.getState().toggleOpen();
      expect(useNotificationCenterStore.getState().isOpen).toBe(false);
    });

    it("sets open state directly", () => {
      useNotificationCenterStore.getState().setOpen(true);
      expect(useNotificationCenterStore.getState().isOpen).toBe(true);

      useNotificationCenterStore.getState().setOpen(false);
      expect(useNotificationCenterStore.getState().isOpen).toBe(false);
    });
  });

  describe("getUnreadCount", () => {
    it("returns 0 when no notifications", () => {
      expect(
        useNotificationCenterStore.getState().getUnreadCount()
      ).toBe(0);
    });

    it("returns count of unread notifications", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "B",
        message: "",
      });

      expect(
        useNotificationCenterStore.getState().getUnreadCount()
      ).toBe(2);
    });

    it("decreases when notification is marked as read", () => {
      const id = useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "B",
        message: "",
      });

      useNotificationCenterStore.getState().markAsRead(id);

      expect(
        useNotificationCenterStore.getState().getUnreadCount()
      ).toBe(1);
    });

    it("returns 0 after marking all as read", () => {
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskReviewReady,
        title: "A",
        message: "",
      });
      useNotificationCenterStore.getState().addNotification({
        category: NotificationCategory.TaskFailed,
        title: "B",
        message: "",
      });

      useNotificationCenterStore.getState().markAllAsRead();

      expect(
        useNotificationCenterStore.getState().getUnreadCount()
      ).toBe(0);
    });
  });

  describe("notification categories", () => {
    it("supports all notification categories", () => {
      const categories = [
        NotificationCategory.TaskReviewReady,
        NotificationCategory.PlanApproval,
        NotificationCategory.TaskFailed,
        NotificationCategory.TtyInputRequest,
        NotificationCategory.TaskCompleted,
      ];

      for (const category of categories) {
        useNotificationCenterStore.getState().addNotification({
          category,
          title: `Test ${category}`,
          message: "",
        });
      }

      expect(
        useNotificationCenterStore.getState().notifications.length
      ).toBe(categories.length);
    });
  });
});
