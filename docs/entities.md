# DeliDev Entities

This document defines the core entity model for DeliDev.
All architecture, API, and UI documents align with this file.

## Conventions

1. IDs are UUID strings.
2. Timestamps are RFC3339 UTC.
3. Known variants use enums, not free-form strings.

## Core Enums

### WorkspaceType

```text
enum WorkspaceType {
  LOCAL_ENDPOINT
  REMOTE_ENDPOINT
}
```

### UnitTaskStatus

```text
enum UnitTaskStatus {
  QUEUED
  IN_PROGRESS
  ACTION_REQUIRED
  BLOCKED
  COMPLETED
  FAILED
  CANCELLED
}
```

### SubTaskType

```text
enum SubTaskType {
  INITIAL_IMPLEMENTATION
  REQUEST_CHANGES
  PR_REVIEW_FIX
  PR_CI_FIX
  MANUAL_RETRY
}
```

### SubTaskStatus

```text
enum SubTaskStatus {
  QUEUED
  IN_PROGRESS
  WAITING_FOR_PLAN_APPROVAL
  WAITING_FOR_USER_INPUT
  COMPLETED
  FAILED
  CANCELLED
}
```

### AgentSessionStatus

```text
enum AgentSessionStatus {
  STARTING
  RUNNING
  WAITING_FOR_INPUT
  COMPLETED
  FAILED
  CANCELLED
}
```

### ActionType

```text
enum ActionType {
  REVIEW_REQUESTED
  PLAN_APPROVAL_REQUIRED
  CI_FAILED
  MERGE_CONFLICT
  SECURITY_ALERT
  USER_INPUT_REQUIRED
}
```

### BadgeColorKey

```text
enum BadgeColorKey {
  BLUE
  GREEN
  YELLOW
  ORANGE
  RED
  GRAY
}
```

### PrStatus

```text
enum PrStatus {
  OPEN
  APPROVED
  CHANGES_REQUESTED
  MERGED
  CLOSED
  CI_FAILED
}
```

### ReviewAssistStatus

```text
enum ReviewAssistStatus {
  OPEN
  ACKNOWLEDGED
  RESOLVED
  DISMISSED
}
```

### NotificationType

```text
enum NotificationType {
  TASK_ACTION_REQUIRED
  PLAN_ACTION_REQUIRED
  PR_REVIEW_ACTIVITY
  PR_CI_FAILURE
  AGENT_SESSION_FAILED
}
```

### StreamEventType

```text
enum StreamEventType {
  TASK_UPDATED
  SUBTASK_UPDATED
  SESSION_OUTPUT
  SESSION_STATE_CHANGED
  PR_UPDATED
  REVIEW_ASSIST_UPDATED
  NOTIFICATION_CREATED
}
```

## Core Entities

### Workspace

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Workspace ID |
| name | string | Y | User-facing name |
| type | WorkspaceType | Y | Endpoint locality |
| endpointUrl | string | Y | Main server URL |
| authProfileId | UUID | N | Credential profile reference |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### Repository

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Repository ID |
| workspaceId | UUID | Y | Parent workspace |
| remoteUrl | string | Y | Git remote URL |
| defaultBranch | string | Y | Default branch |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### RepositoryGroup

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Group ID |
| workspaceId | UUID | Y | Parent workspace |
| name | string | Y | Group name |
| repositoryIds | UUID[] | Y | Included repositories |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### UnitTask

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Task ID |
| workspaceId | UUID | Y | Parent workspace |
| repositoryGroupId | UUID | Y | Scope of repositories |
| title | string | Y | Short title |
| prompt | string | Y | High-level objective |
| branchName | string | N | Preferred branch name |
| status | UnitTaskStatus | Y | Current status |
| actionTypes | ActionType[] | Y | Current required actions |
| prTrackingIds | UUID[] | Y | Related PR tracking entries |
| latestPatchRef | string | N | Reference to persisted patch artifact |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### SubTask

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | SubTask ID |
| unitTaskId | UUID | Y | Parent UnitTask |
| type | SubTaskType | Y | Subtask category |
| prompt | string | Y | Subtask-specific instruction |
| status | SubTaskStatus | Y | Current status |
| planModeEnabled | bool | Y | Uses plan-mode interaction |
| targetActionType | ActionType | N | Action this subtask resolves |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### AgentSession

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Session ID |
| subTaskId | UUID | Y | Parent SubTask |
| agentType | enum | Y | Coding agent type |
| model | string | N | Model name |
| status | AgentSessionStatus | Y | Runtime status |
| tokenUsage | TokenUsageMetrics | N | Usage and cost metrics |
| startedAt | timestamp | N | Start time |
| completedAt | timestamp | N | End time |
| createdAt | timestamp | Y | Created time |

### TokenUsageMetrics

| Field | Type | Required | Description |
|---|---|---|---|
| provider | string | Y | Agent/provider identifier |
| model | string | N | Model name reported by provider |
| inputTokens | int64 | N | Input tokens |
| outputTokens | int64 | N | Output tokens |
| cacheReadTokens | int64 | N | Tokens read from cache |
| cacheWriteTokens | int64 | N | Tokens written to cache |
| totalTokens | int64 | N | Total tokens consumed |
| totalCostUsd | decimal | N | Total session cost in USD |
| pricingVersion | string | N | Pricing table/version identifier |
| rawUsagePayload | json | N | Raw provider usage payload for audit/debug |
| capturedAt | timestamp | Y | Last usage capture time |

### PullRequestTracking

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Tracking record ID |
| unitTaskId | UUID | Y | Parent task |
| provider | enum | Y | GitHub/GitLab/etc |
| repositoryId | UUID | Y | Repository |
| prNumber | int | Y | PR number |
| prUrl | string | Y | Canonical URL |
| status | PrStatus | Y | Latest known PR status |
| lastPolledAt | timestamp | N | Last poll time |
| autoFixEnabled | bool | Y | Auto-run setting |
| maxAutoFixAttempts | int | Y | Retry cap |
| autoFixAttemptsUsed | int | Y | Current attempts |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### ReviewAssistItem

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Item ID |
| unitTaskId | UUID | Y | Parent task |
| prTrackingId | UUID | Y | Related PR |
| sourceType | ActionType | Y | Trigger signal |
| title | string | Y | Human-readable summary |
| details | string | N | Suggested context |
| status | ReviewAssistStatus | Y | Item state |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### BadgeTheme

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Theme ID |
| workspaceId | UUID | Y | Scope |
| actionType | ActionType | Y | Action mapped by this rule |
| colorKey | BadgeColorKey | Y | Selected color token |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

### Notification

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Notification ID |
| workspaceId | UUID | Y | Scope |
| type | NotificationType | Y | Notification category |
| title | string | Y | Title |
| body | string | Y | Message body |
| deepLink | string | N | App route |
| readAt | timestamp | N | Read time |
| createdAt | timestamp | Y | Created time |

## Entity Relationships

1. `Workspace 1:N Repository`
2. `Workspace 1:N UnitTask`
3. `Workspace 1:N BadgeTheme`
4. `UnitTask 1:N SubTask`
5. `SubTask 1:N AgentSession`
6. `UnitTask 1:N PullRequestTracking`
7. `PullRequestTracking 1:N ReviewAssistItem`

## Plan Mode Data Attachment

Plan mode metadata is attached to SubTask and AgentSession records.
