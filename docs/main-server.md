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
7. validate and persist normalized coding-agent message payloads
8. event stream fan-out to clients
9. worker coordination, job dispatch, and cancellation propagation

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
│   ├── SQLite (single-instance mode)                          │
│   ├── PostgreSQL (scale mode)                                │
│   ├── In-memory event broker (single-instance mode)          │
│   └── Redis (scale mode; optional otherwise)                 │
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
6. AgentSession metadata and normalized session output events
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
4. worker emits normalized lifecycle and session output events.
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
4. durable enqueue before publish in `SCALE` mode

## Deployment Modes

Main server supports two deployment modes.

1. `SINGLE_INSTANCE`:
- use SQLite as primary database
- use in-memory event propagation inside the main-server process
- no Redis dependency required

2. `SCALE`:
- use PostgreSQL as primary database
- use Redis as event propagation and replay backbone
- designed for multi-instance/shared deployments

## Event Propagation Backends

Event broker backend depends on deployment mode.

1. in-memory backend (`SINGLE_INSTANCE`):
- publish and subscribe in process memory
- supports single-process fan-out
- replay is limited to process lifetime

2. Redis backend (`SCALE`):
- publish domain events to Redis streams
- use Redis pub/sub fan-out for connected stream workers
- persist ordered event envelopes with sequence metadata
- replay from Redis stream offsets on reconnect

## Worker Coordination

Main server routes executable SubTasks using:

1. worker health status
2. workspace affinity (optional)
3. concurrency caps
4. retry budget

## Normalized Agent Message Contract

Main server consumes only normalized agent message contracts from worker.

1. provider-native agent output is never parsed at main-server layer
2. incoming worker payloads are validated against normalized `SessionOutputEvent` schema
3. rejected payloads are logged as contract violations and not forwarded to clients
4. persisted session logs and streamed `SESSION_OUTPUT` events use the same normalized schema

## Database Support

Main server supports both PostgreSQL and SQLite by deployment mode.

1. `SINGLE_INSTANCE`: SQLite recommended.
2. `SCALE`: PostgreSQL required and recommended.
3. Both backends use the same logical schema and migration policy.

## Authentication and Authorization

1. endpoint-auth profile per workspace
2. bearer token validation for shared deployments
3. workspace-scoped authorization checks for every RPC

## Configuration

| Key | Required | Description |
|---|---|---|
| `DELIDEV_DEPLOYMENT_MODE` | Y | `SINGLE_INSTANCE` or `SCALE` |
| `DELIDEV_HTTP_ADDR` | Y | Connect RPC bind address |
| `DELIDEV_DATABASE_URL` | Y | SQLite DSN (`SINGLE_INSTANCE`) or PostgreSQL DSN (`SCALE`) |
| `DELIDEV_REDIS_URL` | N | Redis connection URL (required only in `SCALE`) |
| `DELIDEV_REDIS_STREAM_PREFIX` | N | Redis key prefix for workspace streams (`SCALE` only) |
| `DELIDEV_WORKER_RPC_TIMEOUT` | N | Worker call timeout |
| `DELIDEV_PR_POLL_INTERVAL_SEC` | N | PR polling interval |
| `DELIDEV_AUTH_ISSUER_URL` | N | OIDC issuer |
| `DELIDEV_AUTH_AUDIENCE` | N | expected token audience |

Deployment examples:

1. `SINGLE_INSTANCE`:
- `DELIDEV_DEPLOYMENT_MODE=SINGLE_INSTANCE`
- `DELIDEV_DATABASE_URL=sqlite:///Users/<user>/.delidev/main-server.db`

2. `SCALE`:
- `DELIDEV_DEPLOYMENT_MODE=SCALE`
- `DELIDEV_DATABASE_URL=postgres://localhost:5432/delidev`
- `DELIDEV_REDIS_URL=redis://localhost:6379/0`

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
