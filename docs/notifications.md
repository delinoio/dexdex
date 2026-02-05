# Notification System

This document describes the DeliDev notification system, covering both persistent in-app notifications (notification center) and platform desktop notifications.

## Overview

DeliDev has two complementary notification subsystems:

| Subsystem | Purpose | Persistence |
|-----------|---------|-------------|
| **Toast Notifications** | Ephemeral feedback for user actions (e.g., "Settings saved") | Transient (auto-dismiss) |
| **Notification Center** | Persistent record of system events (task status changes, agent questions) | Persisted to localStorage |
| **Desktop Notifications** | OS-level notifications when app is not focused | Platform-managed |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Tauri Backend (Rust)                     │
│                                                             │
│  notifications.rs ──► Desktop Notification (OS-level)       │
│  events.rs ──────────► Tauri Event Bus                      │
└────────────────────────────┬────────────────────────────────┘
                             │ Events
                             ▼
┌─────────────────────────────────────────────────────────────┐
│                   Frontend (React/TypeScript)                │
│                                                             │
│  useNotificationEvents.ts                                   │
│    ├── Listens to Tauri events                              │
│    ├── Creates persistent notifications in center store     │
│    └── Sends desktop notifications when window unfocused    │
│                                                             │
│  notificationCenterStore.ts (Zustand + persist)             │
│    ├── Stores persistent notifications                      │
│    ├── Read/unread state management                         │
│    └── Persisted to localStorage                            │
│                                                             │
│  notificationStore.ts (Zustand)                             │
│    └── Ephemeral toast notifications (auto-dismiss)         │
└─────────────────────────────────────────────────────────────┘
```

## Notification Categories

| Category | Trigger | Description |
|----------|---------|-------------|
| `task_review_ready` | Task status → `in_review` | AI completed work, awaiting human review |
| `plan_approval` | Task status → `pending_approval` | Composite task plan ready for approval |
| `task_failed` | Task status → `failed` | Task execution failed |
| `tty_input_request` | TTY input event | AI agent is asking a question |
| `task_completed` | Task completed event | Task finished successfully |

## Frontend Components

### Notification Center Store

**File:** `src/stores/notificationCenterStore.ts`

Zustand store with `persist` middleware. Stores notifications in `localStorage` under `delidev-notification-center`.

```typescript
interface PersistentNotification {
  id: string;
  category: NotificationCategory;
  title: string;
  message: string;
  read: boolean;
  taskId?: string;
  taskType?: "unit_task" | "composite_task";
  createdAt: number;
}
```

**Actions:**
- `addNotification()` — Add a new notification (unread by default). Automatically trims when exceeding `MAX_NOTIFICATIONS` (200), removing oldest read notifications first.
- `removeNotification(id)` — Delete a notification
- `markAsRead(id)` — Mark a single notification as read
- `markAllAsRead()` — Mark all notifications as read
- `clearAll()` — Remove all notifications
- `toggleOpen()` / `setOpen()` — Open/close the notification panel
- `getUnreadCount()` — Get the number of unread notifications

**Limits:**
- Maximum 200 notifications are stored (`MAX_NOTIFICATIONS`). When exceeded, oldest read notifications are trimmed first. If still over the limit (e.g., all unread), the oldest notifications are dropped.

### Shared Utilities

**File:** `src/components/notifications/utils.ts`

Helper functions used by both the panel and full-page components:
- `categoryLabel(category)` — Human-readable label for a notification category
- `categoryColor(category)` — Tailwind background color class for a category
- `formatTimeAgo(timestamp)` — Relative time string (e.g., "5m ago", "2d ago")
- `formatTime(timestamp)` — Absolute or relative time for full-page view
- `getNotificationPath(notification)` — Route path for a notification's associated task

### Notification Panel

**File:** `src/components/notifications/NotificationPanel.tsx`

A slide-out sidebar panel from the right side of the screen. Features:
- Header with unread count badge
- "Mark all as read" and "Clear all" action buttons
- Scrollable list of notification items
- Each item shows: category badge, title, message, timestamp
- Hover actions: "Mark as read" and "Delete" buttons
- Click navigates to the relevant task page
- "View all notifications" link to the full page
- Backdrop overlay for dismissal

### Notifications Page

**File:** `src/pages/Notifications.tsx`

Full-page view at `/notifications` route. Features:
- Header with unread count
- Bulk actions: "Mark all as read", "Clear all"
- Notification rows with: unread indicator, category, title, message, timestamp
- Hover actions: mark as read (checkmark icon) and delete (trash icon)
- Click navigates to the relevant task page
- Empty state illustration when no notifications

### Sidebar Integration

**File:** `src/components/layout/Sidebar.tsx`

Bell icon button in the sidebar with:
- Unread count badge (red circle with count, shows "99+" for > 99)
- Toggles the notification panel open/close
- Respects sidebar collapsed state

### Mobile Navigation

**File:** `src/components/mobile/MobileNavigation.tsx`

"Alerts" tab in mobile bottom navigation linking to `/notifications` page.

## Event Listener Hook

**File:** `src/hooks/useNotificationEvents.ts`

Initializes once in `App.tsx`. Listens to three Tauri event types:

1. **`task-status-changed`** — Creates notifications for `in_review`, `pending_approval`, and `failed` status transitions
2. **`task-completed`** — Creates notifications for successful completions and failures with error details
3. **`tty-input-request`** — Creates notifications when agents ask questions

Also sends desktop notifications via `@tauri-apps/plugin-notification` when `document.hasFocus()` is `false`.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `n` | Toggle notification panel |
| `Escape` | Close notification panel (among other dialogs) |

## Desktop Notifications

Desktop notifications are sent in two places:

1. **Rust backend** (`src-tauri/src/notifications.rs`): Sends OS notifications via `tauri-plugin-notification` for task events during execution.
2. **Frontend** (`useNotificationEvents.ts`): Sends OS notifications when the window is not focused and Tauri events arrive.

Permission is requested once on app startup via `useNotificationPermission` hook.

## Toast Notifications (Ephemeral)

**File:** `src/stores/notificationStore.ts`

Separate from the notification center. Used for immediate user feedback:

```typescript
notify.success("Settings saved", "Your settings have been saved.");
notify.error("Failed to save", "Please try again.");
notify.warning("Rate limit", "Approaching API rate limit.");
notify.info("Update available", "A new version is available.");
```

Toasts auto-dismiss after 5 seconds by default.

## Configuration

Notification preferences are configured in global settings (`~/.delidev/config.toml`):

```toml
[notification]
enabled = true
approvalRequest = true
userQuestion = true
reviewReady = true
```

These settings control which desktop notifications are sent from the Rust backend.
