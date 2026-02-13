# DeliDev API Reference (To-Be, Connect RPC)

This document defines the target API contract for the rewrite.
All business communication is Connect RPC-based.

## Protocol

- Transport: HTTP/2 (fallback HTTP/1.1 where needed)
- Encoding: Protobuf (JSON debug view optional)
- RPC style: Connect RPC unary + server-streaming
- Auth: bearer token for authenticated workspaces

## API Design Rules

1. Connect RPC is the primary interface.
2. Tauri-specific commands must not define business contracts.
3. Public requests and responses use enums for known variants.
4. Streaming channels must emit typed events with monotonic sequence IDs.

## Service Overview

1. `WorkspaceService`
2. `RepositoryService`
3. `TaskService`
4. `SessionService`
5. `PrManagementService`
6. `ReviewAssistService`
7. `BadgeThemeService`
8. `NotificationService`
9. `EventStreamService`

---

## WorkspaceService

### CreateWorkspace

Creates a workspace endpoint profile.

Request:
- `name: string`
- `type: WorkspaceType`
- `endpoint_url: string`
- `auth_profile_id?: string`

Response:
- `workspace: Workspace`

### ListWorkspaces

Request:
- `limit: int32`
- `offset: int32`

Response:
- `workspaces: Workspace[]`
- `total_count: int32`

### UpdateWorkspace

Request:
- `workspace_id: string`
- `name?: string`
- `endpoint_url?: string`
- `auth_profile_id?: string`

Response:
- `workspace: Workspace`

### DeleteWorkspace

Request:
- `workspace_id: string`

Response:
- empty

### SetActiveWorkspace

Request:
- `workspace_id: string`

Response:
- empty

---

## RepositoryService

### AddRepository

Request:
- `workspace_id: string`
- `remote_url: string`
- `default_branch?: string`

Response:
- `repository: Repository`

### ListRepositories

Request:
- `workspace_id: string`
- `limit: int32`
- `offset: int32`

Response:
- `repositories: Repository[]`
- `total_count: int32`

### CreateRepositoryGroup

Request:
- `workspace_id: string`
- `name: string`
- `repository_ids: string[]`

Response:
- `group: RepositoryGroup`

### UpdateRepositoryGroup

Request:
- `group_id: string`
- `name?: string`
- `repository_ids?: string[]`

Response:
- `group: RepositoryGroup`

### DeleteRepositoryGroup

Request:
- `group_id: string`

Response:
- empty

---

## TaskService

### CreateUnitTask

Request:
- `workspace_id: string`
- `repository_group_id: string`
- `title: string`
- `prompt: string`
- `branch_name?: string`

Response:
- `task: UnitTask`

### ListUnitTasks

Request:
- `workspace_id: string`
- `statuses?: UnitTaskStatus[]`
- `action_types?: ActionType[]`
- `limit: int32`
- `offset: int32`

Response:
- `tasks: UnitTask[]`
- `total_count: int32`

### GetUnitTask

Request:
- `task_id: string`

Response:
- `task: UnitTask`

### UpdateUnitTaskStatus

Request:
- `task_id: string`
- `status: UnitTaskStatus`
- `reason?: string`

Response:
- `task: UnitTask`

### CreateSubTask

Request:
- `unit_task_id: string`
- `type: SubTaskType`
- `prompt: string`
- `plan_mode_enabled: bool`
- `target_action_type?: ActionType`

Response:
- `sub_task: SubTask`

### ListSubTasks

Request:
- `unit_task_id: string`

Response:
- `sub_tasks: SubTask[]`

### RetrySubTask

Request:
- `sub_task_id: string`

Response:
- `sub_task: SubTask`

### CancelSubTask

Request:
- `sub_task_id: string`

Response:
- `sub_task: SubTask`

### SubmitPlanDecision

Used when a plan-mode session requests explicit decision.

Request:
- `sub_task_id: string`
- `decision: enum { APPROVE, REVISE, REJECT }`
- `feedback?: string`

Response:
- `sub_task: SubTask`

---

## SessionService

### ListAgentSessions

Request:
- `sub_task_id: string`

Response:
- `sessions: AgentSession[]`

### GetAgentSessionLog

Request:
- `session_id: string`
- `cursor?: string`

Response:
- `events: SessionOutputEvent[]`
- `next_cursor?: string`

### StopAgentSession

Request:
- `session_id: string`

Response:
- `session: AgentSession`

### SubmitSessionInput

Request:
- `session_id: string`
- `input: string`

Response:
- empty

---

## PrManagementService

### TrackPullRequest

Request:
- `unit_task_id: string`
- `repository_id: string`
- `provider: enum`
- `pr_number: int32`
- `pr_url: string`

Response:
- `tracking: PullRequestTracking`

### ListTrackedPullRequests

Request:
- `workspace_id: string`
- `statuses?: PrStatus[]`
- `limit: int32`
- `offset: int32`

Response:
- `items: PullRequestTracking[]`
- `total_count: int32`

### RunAutoFixNow

Manual one-click remediation.

Request:
- `pr_tracking_id: string`
- `reason: enum { REVIEW_EVENT, CI_FAILURE, MANUAL }`

Response:
- `sub_task: SubTask`

### SetAutoFixPolicy

Request:
- `pr_tracking_id: string`
- `auto_fix_enabled: bool`
- `max_auto_fix_attempts: int32`

Response:
- `tracking: PullRequestTracking`

---

## ReviewAssistService

### ListReviewAssistItems

Request:
- `workspace_id: string`
- `statuses?: ReviewAssistStatus[]`
- `limit: int32`
- `offset: int32`

Response:
- `items: ReviewAssistItem[]`
- `total_count: int32`

### ResolveReviewAssistItem

Request:
- `item_id: string`
- `status: ReviewAssistStatus`

Response:
- `item: ReviewAssistItem`

---

## BadgeThemeService

### ListBadgeThemes

Request:
- `workspace_id: string`

Response:
- `themes: BadgeTheme[]`

### UpsertBadgeTheme

Request:
- `workspace_id: string`
- `action_type: ActionType`
- `color_key: BadgeColorKey`

Response:
- `theme: BadgeTheme`

---

## NotificationService

### ListNotifications

Request:
- `workspace_id: string`
- `limit: int32`
- `offset: int32`

Response:
- `notifications: Notification[]`
- `total_count: int32`

### MarkNotificationRead

Request:
- `notification_id: string`

Response:
- `notification: Notification`

---

## EventStreamService

### StreamWorkspaceEvents (Server Streaming)

Request:
- `workspace_id: string`
- `from_sequence?: uint64`

Response stream:
- `WorkspaceEventEnvelope`
  - `sequence: uint64`
  - `event_type: StreamEventType`
  - `emitted_at: timestamp`
  - `payload: oneof`

Payload variants:

1. `TaskUpdatedEvent`
2. `SubTaskUpdatedEvent`
3. `SessionOutputEvent`
4. `SessionStateChangedEvent`
5. `PrUpdatedEvent`
6. `ReviewAssistUpdatedEvent`
7. `NotificationCreatedEvent`

---

## Errors

Standard Connect error mapping:

1. `INVALID_ARGUMENT`
2. `UNAUTHENTICATED`
3. `PERMISSION_DENIED`
4. `NOT_FOUND`
5. `FAILED_PRECONDITION`
6. `RESOURCE_EXHAUSTED`
7. `INTERNAL`
8. `UNAVAILABLE`

Error details should include:

- `code`
- `message`
- `request_id`
- optional typed detail payload

---

## Backward Compatibility

1. `CompositeTask` APIs are removed from active contract.
2. Legacy compatibility (if needed) must be isolated behind migration adapters and excluded from new client code.
3. New client surfaces must not depend on mode-specific endpoints.
