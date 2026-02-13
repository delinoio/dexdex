# Notification System (To-Be)

DeliDev uses Web Notification API as the primary desktop/mobile notification channel.

## Design Rules

1. Primary notification channel: Web Notification API
2. In-app notification center is always authoritative
3. Native notification plugins are not primary in this design
4. Notification emission is event-stream driven

## Trigger Sources

1. UnitTask enters `ACTION_REQUIRED`
2. SubTask enters `WAITING_FOR_PLAN_APPROVAL`
3. PR review activity requires remediation
4. PR CI failure
5. AgentSession failure

## Notification Flow

```
EventStreamService emits event
      -> Client event reducer writes Notification record locally
      -> If allowed and app backgrounded, call Web Notification API
      -> User click deep-links into task/pr/review detail
```

## Permission Handling

1. prompt only after user intent (settings toggle or first actionable event)
2. store local permission state cache
3. expose clear permission status in settings

## Deduplication

Use `(workspace_id, sequence, notification_type)` as dedupe key.
Do not dispatch duplicate browser notifications for already processed sequence IDs.

## Categories

1. `TASK_ACTION_REQUIRED`
2. `PLAN_ACTION_REQUIRED`
3. `PR_REVIEW_ACTIVITY`
4. `PR_CI_FAILURE`
5. `AGENT_SESSION_FAILED`

## Delivery Rules

1. Foreground: in-app toast + notification center
2. Background: Web Notification API + notification center
3. No permission: notification center only

## UX Requirements

1. all notifications include route deep link
2. all notifications expose created timestamp
3. unread state persists across restarts
4. mark-as-read is synchronized with server state

## Data Model

See `Notification` entity in `docs/entities.md`.

## Operational Logging

Client logs:

1. permission prompts and result
2. dispatch success/failure
3. click-through route handling

Server logs:

1. notification event generation reason
2. workspace/task/pr correlation IDs
