# Main Server (Go) - To-Be Design

Main Server is the control plane for DeliDev.
It exposes Connect RPC APIs and coordinates task/PR/event lifecycle.

## Responsibilities

1. Workspace endpoint and auth management
2. Repository and RepositoryGroup lifecycle
3. UnitTask/SubTask orchestration state
4. PR tracking and polling scheduler
5. Review-assist generation and status updates
6. Event stream fan-out to clients
7. Worker coordination and job dispatch

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ Main Server (Go)                                             │
│                                                              │
│  Connect RPC Handlers                                        │
│   ├── WorkspaceService                                       │
│   ├── RepositoryService                                      │
│   ├── TaskService                                            │
│   ├── SessionService                                         │
│   ├── PrManagementService                                    │
│   ├── ReviewAssistService                                    │
│   ├── BadgeThemeService                                      │
│   ├── NotificationService                                    │
│   └── EventStreamService                                     │
│                                                              │
│  Domain Services                                             │
│   ├── TaskCoordinator                                        │
│   ├── PRPoller                                               │
│   ├── ReviewAssistEngine                                     │
│   ├── EventBroker                                            │
│   └── WorkerRouter                                           │
│                                                              │
│  Storage                                                     │
│   ├── Postgres (primary)                                     │
│   └── Redis (optional: stream cursor/cache/locks)            │
└──────────────────────────────────────────────────────────────┘
```

## Connect RPC Priority

Main server is the canonical business interface.
No client workflow should require Tauri-only command contracts.

## Data Ownership

Main server stores and owns:

1. Workspace records
2. Repository metadata and grouping
3. UnitTask/SubTask state machines
4. AgentSession metadata and log pointers
5. PR tracking state and auto-fix counters
6. Review assist items
7. Badge theme settings
8. Notification records
9. Event sequence offsets

## Task Orchestration Flow

1. `CreateUnitTask` persists task with `QUEUED` status.
2. Scheduler enqueues initial SubTask (`INITIAL_IMPLEMENTATION`).
3. WorkerRouter assigns job to worker server.
4. Worker emits lifecycle/log events.
5. Main server updates UnitTask action state and emits stream events.

## PR Polling and Auto-Fix

1. PRPoller periodically checks tracked PRs.
2. On "changes requested" or "CI failed":
- create ReviewAssistItem
- mark UnitTask action as required
- emit notification and stream event
3. If auto-fix policy is enabled:
- create remediation SubTask
- dispatch to worker

## Event Broker

Event broker requirements:

1. monotonic sequence per workspace
2. replay from sequence cursor
3. best-effort fan-out to connected clients
4. durable enqueue before publish

## Worker Coordination

Main server routes executable SubTasks to worker servers using:

1. worker health status
2. workspace affinity (optional)
3. concurrency caps
4. retry budget

## Authentication and Authorization

1. Endpoint-auth profile per workspace
2. Bearer token validation for remote/shared deployments
3. Workspace-scoped authorization checks for every RPC

## Configuration (Target)

| Key | Required | Description |
|---|---|---|
| `DELIDEV_HTTP_ADDR` | Y | Connect RPC bind address |
| `DELIDEV_DATABASE_URL` | Y | Postgres DSN |
| `DELIDEV_EVENT_BACKEND` | N | `postgres` or `redis` |
| `DELIDEV_WORKER_RPC_TIMEOUT` | N | Worker call timeout |
| `DELIDEV_PR_POLL_INTERVAL_SEC` | N | PR polling interval |
| `DELIDEV_AUTH_ISSUER_URL` | N | OIDC issuer |
| `DELIDEV_AUTH_AUDIENCE` | N | expected token audience |

## Logging and Metrics

Structured logs must include:

1. `workspace_id`
2. `unit_task_id`
3. `sub_task_id`
4. `session_id`
5. `pr_tracking_id`
6. `request_id`

Key metrics:

1. task queue latency
2. subtask success/failure rate
3. stream delivery lag
4. PR poll cycle duration
5. auto-fix success ratio

## Failure Handling

1. Worker unavailable: SubTask returns to queue with backoff.
2. PR provider API failure: preserve last known state and retry on next poll cycle.
3. Stream client disconnect: client resumes from last sequence.
4. Duplicate event processing: idempotency by `event_id` and sequence checks.

## Migration Note

`CompositeTask` and its graph execution logic are not part of this target server design.
