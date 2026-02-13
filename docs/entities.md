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
  PR_CREATE
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
  PR_CREATION_READY
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

### ReviewInlineCommentStatus

```text
enum ReviewInlineCommentStatus {
  OPEN
  RESOLVED
  DELETED
}
```

### DiffSide

```text
enum DiffSide {
  OLD
  NEW
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
  INLINE_COMMENT_UPDATED
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
| repositoryIds | UUID[] | Y | Ordered repositories (first item is primary execution repository) |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

RepositoryGroup execution semantics:

1. RepositoryGroup is the unit of task execution scope.
2. `repositoryIds` order is significant.
3. repository group must contain at least one repository.
4. the first repository is the primary execution repository.
5. remaining repositories are attached as additional working directories for agent execution.

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
| latestCommitSha | string | N | Latest commit SHA generated for this task branch |
| generatedCommitCount | int | Y | Total number of generated commits across subtasks |
| latestPatchRef | string | N | Reference to persisted patch artifact |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

`latestPatchRef` is a derived artifact for diff rendering.
The authoritative source for PR creation and Commit to Local is the real git commit chain (`generatedCommits`).

UnitTask cancellation semantics:

1. user can cancel an in-progress UnitTask at any time.
2. canceling UnitTask stops active SubTask execution and active AgentSession processes.
3. UnitTask transitions to `CANCELLED` after cancellation is acknowledged.

### SubTask

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | SubTask ID |
| unitTaskId | UUID | Y | Parent UnitTask |
| type | SubTaskType | Y | Subtask category (includes small operational tasks like PR creation) |
| prompt | string | Y | Subtask-specific instruction |
| status | SubTaskStatus | Y | Current status |
| planModeEnabled | bool | Y | Uses plan-mode interaction |
| targetActionType | ActionType | N | Action this subtask resolves |
| baseCommitSha | string | N | Branch HEAD SHA before this subtask started |
| headCommitSha | string | N | Branch HEAD SHA after this subtask completed |
| generatedCommits | GeneratedCommit[] | Y | Ordered real git commit chain produced by this subtask |
| createdAt | timestamp | Y | Created time |
| updatedAt | timestamp | Y | Updated time |

SubTask cancellation semantics:

1. user can cancel an in-progress SubTask at any time.
2. cancellation terminates active AgentSession processes for that SubTask.
3. SubTask transitions to `CANCELLED` when cancellation completes.

### GeneratedCommit

| Field | Type | Required | Description |
|---|---|---|---|
| sha | string | Y | Commit SHA |
| parentShas | string[] | Y | Parent commit SHAs |
| title | string | Y | Commit subject line |
| body | string | N | Commit body |
| authoredAt | timestamp | Y | Commit authored time |

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

### ReviewInlineComment

| Field | Type | Required | Description |
|---|---|---|---|
| id | UUID | Y | Comment ID |
| unitTaskId | UUID | Y | Parent UnitTask |
| subTaskId | UUID | N | Optional SubTask scope |
| filePath | string | Y | Relative file path in diff |
| side | DiffSide | Y | `OLD` or `NEW` diff side |
| lineNumber | int | Y | Line number in selected diff side |
| body | string | Y | Comment text |
| status | ReviewInlineCommentStatus | Y | Comment status |
| authorUserId | UUID | Y | Author user ID |
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
8. `UnitTask 1:N ReviewInlineComment`

## Plan Mode Data Attachment

Plan mode metadata is attached to SubTask and AgentSession records.
