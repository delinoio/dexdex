# Main Server (Go)

Main Server is the control plane for DeliDev.
It exposes Connect RPC APIs and coordinates task, PR, and event lifecycles.

## Responsibilities

1. workspace endpoint and auth management
2. repository and repository group lifecycle
3. UnitTask and SubTask orchestration state
4. PR tracking and polling scheduler
5. review-assist generation and updates
6. inline comment lifecycle for code review diff
7. event stream fan-out to clients
8. worker coordination, job dispatch, and cancellation propagation

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
│   ├── ReviewCommentService                                   │
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
│   ├── PostgreSQL (recommended)                               │
│   ├── SQLite (supported for local deployments)               │
│   └── Redis (required: event propagation and replay)          │
└──────────────────────────────────────────────────────────────┘
```

## Connect RPC Priority

Main server is the canonical business interface.
No client workflow requires Tauri-only business contracts.

## Data Ownership

Main server stores and owns:

1. workspace records
2. repository metadata and grouping
3. ordered RepositoryGroup membership and primary repository selection (first repository in group)
4. UnitTask and SubTask state machines
5. SubTask commit-chain metadata and commit ancestry
6. AgentSession metadata and log pointers
7. PR tracking state and auto-fix counters
8. review assist items
9. review inline comments and status
10. badge theme settings
11. notification records
12. event sequence offsets

## Task Orchestration Flow

1. `CreateUnitTask` persists task with `QUEUED` status.
2. scheduler enqueues initial SubTask (`INITIAL_IMPLEMENTATION`).
3. WorkerRouter assigns job to worker server.
4. worker emits lifecycle and log events.
5. main server updates UnitTask action state and emits stream events.

Cancellation flow:

1. user calls `CancelUnitTask` or `CancelSubTask`.
2. main server sends cancellation signal to worker runner immediately.
3. main server persists `CANCELLED` status after worker acknowledgement or timeout policy.
4. cancellation status is published through event stream.

## PR Polling and Auto-Fix

1. PRPoller periodically checks tracked PRs.
2. On `changes_requested` or `ci_failed`:
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

## Redis Event Propagation

Main server uses Redis as the required event backbone.

1. publish every domain event to Redis stream channels
2. use Redis pub/sub fan-out for connected stream workers
3. persist ordered event envelopes with sequence metadata
4. replay from Redis stream offsets on reconnect

## Worker Coordination

Main server routes executable SubTasks using:

1. worker health status
2. workspace affinity (optional)
3. concurrency caps
4. retry budget

## Database Support

Main server supports both PostgreSQL and SQLite.

1. PostgreSQL is the recommended database for shared and production deployments.
2. SQLite is supported for local and single-node deployments.
3. Both backends use the same logical schema and migration policy.

## Authentication and Authorization

1. endpoint-auth profile per workspace
2. bearer token validation for shared deployments
3. workspace-scoped authorization checks for every RPC

## Configuration

| Key | Required | Description |
|---|---|---|
| `DELIDEV_HTTP_ADDR` | Y | Connect RPC bind address |
| `DELIDEV_DATABASE_URL` | Y | PostgreSQL or SQLite DSN (PostgreSQL recommended) |
| `DELIDEV_REDIS_URL` | Y | Redis connection URL for event propagation |
| `DELIDEV_REDIS_STREAM_PREFIX` | N | Redis key prefix for workspace event streams |
| `DELIDEV_WORKER_RPC_TIMEOUT` | N | Worker call timeout |
| `DELIDEV_PR_POLL_INTERVAL_SEC` | N | PR polling interval |
| `DELIDEV_AUTH_ISSUER_URL` | N | OIDC issuer |
| `DELIDEV_AUTH_AUDIENCE` | N | expected token audience |

Database URL examples:

1. PostgreSQL (recommended): `postgres://localhost:5432/delidev`
2. SQLite (local): `sqlite:///Users/<user>/.delidev/main-server.db`

## Logging and Metrics

Structured logs include:

1. `workspace_id`
2. `unit_task_id`
3. `sub_task_id`
4. `session_id`
5. `pr_tracking_id`
6. `request_id`

Key metrics:

1. task queue latency
2. subtask success and failure rate
3. stream delivery lag
4. PR poll cycle duration
5. auto-fix success ratio

## Failure Handling

1. worker unavailable: SubTask returns to queue with backoff
2. PR provider API failure: keep last known state and retry on next poll cycle
3. stream client disconnect: client resumes from last sequence
4. duplicate event processing: idempotency by `event_id` and sequence checks
