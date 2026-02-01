import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { useNotificationStore, notify } from "../notificationStore";

describe("notificationStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useNotificationStore.setState({ notifications: [] });
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("addNotification", () => {
    it("adds a notification to the store", () => {
      useNotificationStore.getState().addNotification({
        type: "success",
        title: "Success",
        message: "Operation completed",
      });

      const notifications = useNotificationStore.getState().notifications;
      expect(notifications.length).toBe(1);
      expect(notifications[0].type).toBe("success");
      expect(notifications[0].title).toBe("Success");
      expect(notifications[0].message).toBe("Operation completed");
    });

    it("assigns unique IDs to notifications", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "First",
      });
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Second",
      });

      const notifications = useNotificationStore.getState().notifications;
      expect(notifications[0].id).not.toBe(notifications[1].id);
    });

    it("sets default duration of 5000ms", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
      });

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.duration).toBe(5000);
    });

    it("accepts custom duration", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
        duration: 10000,
      });

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.duration).toBe(10000);
    });

    it("auto-removes notification after duration", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
        duration: 3000,
      });

      expect(useNotificationStore.getState().notifications.length).toBe(1);

      vi.advanceTimersByTime(3000);

      expect(useNotificationStore.getState().notifications.length).toBe(0);
    });

    it("does not auto-remove if duration is 0", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
        duration: 0,
      });

      vi.advanceTimersByTime(10000);

      expect(useNotificationStore.getState().notifications.length).toBe(1);
    });

    it("returns the notification ID", () => {
      const id = useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
      });

      expect(typeof id).toBe("string");
      expect(id).toContain("notification-");
    });

    it("sets createdAt timestamp", () => {
      const now = Date.now();
      vi.setSystemTime(now);

      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
      });

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.createdAt).toBe(now);
    });
  });

  describe("removeNotification", () => {
    it("removes a specific notification by ID", () => {
      const id1 = useNotificationStore.getState().addNotification({
        type: "info",
        title: "First",
        duration: 0, // Don't auto-remove
      });
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Second",
        duration: 0,
      });

      useNotificationStore.getState().removeNotification(id1);

      const notifications = useNotificationStore.getState().notifications;
      expect(notifications.length).toBe(1);
      expect(notifications[0].title).toBe("Second");
    });

    it("does nothing if ID does not exist", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Test",
        duration: 0,
      });

      useNotificationStore.getState().removeNotification("non-existent-id");

      expect(useNotificationStore.getState().notifications.length).toBe(1);
    });
  });

  describe("clearAllNotifications", () => {
    it("removes all notifications", () => {
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "First",
        duration: 0,
      });
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Second",
        duration: 0,
      });
      useNotificationStore.getState().addNotification({
        type: "info",
        title: "Third",
        duration: 0,
      });

      useNotificationStore.getState().clearAllNotifications();

      expect(useNotificationStore.getState().notifications.length).toBe(0);
    });
  });

  describe("notify helpers", () => {
    it("notify.success creates a success notification", () => {
      notify.success("Success Title", "Success Message");

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.type).toBe("success");
      expect(notification.title).toBe("Success Title");
      expect(notification.message).toBe("Success Message");
    });

    it("notify.error creates an error notification", () => {
      notify.error("Error Title", "Error Message");

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.type).toBe("error");
      expect(notification.title).toBe("Error Title");
      expect(notification.message).toBe("Error Message");
    });

    it("notify.warning creates a warning notification", () => {
      notify.warning("Warning Title", "Warning Message");

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.type).toBe("warning");
      expect(notification.title).toBe("Warning Title");
      expect(notification.message).toBe("Warning Message");
    });

    it("notify.info creates an info notification", () => {
      notify.info("Info Title", "Info Message");

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.type).toBe("info");
      expect(notification.title).toBe("Info Title");
      expect(notification.message).toBe("Info Message");
    });

    it("notify helpers work without message", () => {
      notify.success("Title Only");

      const notification = useNotificationStore.getState().notifications[0];
      expect(notification.title).toBe("Title Only");
      expect(notification.message).toBeUndefined();
    });
  });
});
