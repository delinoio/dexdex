# Notification System

DeliDev uses Web Notification API as the primary desktop and mobile notification channel.

## Design Rules

1. primary channel: Web Notification API
2. in-app notification center is authoritative
3. notification emission is event-stream driven

## Trigger Sources

1. UnitTask enters `ACTION_REQUIRED`
2. SubTask enters `WAITING_FOR_PLAN_APPROVAL`
3. PR review activity requires remediation
4. PR CI failure
5. AgentSession failure

## Notification Flow

```
App startup requests Web Notification permission
      -> EventStreamService emits event
      -> client event reducer writes Notification record locally
      -> if allowed and app is backgrounded, call Web Notification API
      -> user click deep-links into task, PR, or review detail
```

## Permission Handling

1. request Web Notification permission at app startup
2. if permission is denied, do not auto-loop prompts during the same session
3. store local permission state cache
4. expose permission status and retry action in settings

## Deduplication

Use `(workspace_id, sequence, notification_type)` as dedupe key.
Do not dispatch duplicates for already processed sequence IDs.

## Categories

1. `TASK_ACTION_REQUIRED`
2. `PLAN_ACTION_REQUIRED`
3. `PR_REVIEW_ACTIVITY`
4. `PR_CI_FAILURE`
5. `AGENT_SESSION_FAILED`

## Delivery Rules

1. foreground: in-app toast and notification center
2. background: Web Notification API and notification center
3. no permission: notification center only

## UX Requirements

1. every notification includes deep link route
2. every notification includes created timestamp
3. unread state persists across restarts
4. mark-as-read syncs with server state

## Data Model

See `Notification` in `docs/entities.md`.

## Operational Logging

Client logs:

1. permission prompts and results
2. dispatch success and failure
3. click-through route handling

Server logs:

1. notification event generation reason
2. workspace, task, and PR correlation IDs
