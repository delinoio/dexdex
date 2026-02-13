# DeliDev Design

DeliDev uses a Connect RPC-first architecture with Tauri clients and Go servers.
This document is the primary architecture reference.

## Product Goals

1. Use Tauri as the desktop and mobile app container.
2. Use Go for `main-server` and `worker-server`.
3. Use `Workspace` as the primary connectivity and scope concept.
4. Use UnitTask-centric workflows with nested SubTask and AgentSession history.
5. Make PR management and PR review assist first-class workflows.
6. Provide real-time event streaming for UI updates and automation.
7. Support iOS and Android as first-wave platforms.

## Non-Goals

1. Direct local folder execution without worktree.
2. Tauri-invoke-first business contracts.
3. Native OS notification plugin as the primary notification channel.

## Top-Level Architecture

```
                        Connect RPC + Event Streams
┌───────────────────────┐   https://api endpoint   ┌───────────────────────┐
│ Tauri Client          │ <----------------------> │ Main Server (Go)      │
│ (Desktop / iOS /      │                          │ - RPC API             │
│  Android)             │                          │ - Workspace/Task/PR   │
│ - React UI            │                          │ - Event broker        │
│ - Web Notification API│                          │ - Auth & policy       │
└───────────┬───────────┘                          └───────────┬───────────┘
            │                                                   │
            │                                                   │ Connect RPC
            │                                                   │
            │                                         ┌─────────▼──────────┐
            │                                         │ Worker Server (Go) │
            │                                         │ - Worktree exec    │
            │                                         │ - Agent sessions   │
            │                                         │ - Log stream       │
            │                                         └────────────────────┘
```

## Monorepo Structure

- `apps/main-server/` (Go)
- `apps/worker-server/` (Go)
- `apps/tauri-app/` (Tauri + React)
- single root `go.mod` for server apps

## Connect RPC First Rule

All business data and control flows use Connect RPC.

Tauri-native bindings are used only for platform integration:

1. window lifecycle and tray integration
2. secure local storage helpers
3. file picker and OS capabilities
4. deep links

Business operations (task, repository, workspace, PR, review, streaming, settings) are not Tauri-only contracts.

## Workspace Model

DeliDev uses workspace switching.
Each workspace points to a main server endpoint.

### Workspace Types

1. Local Endpoint Workspace
- endpoint runs on the same device (for example `http://127.0.0.1:4621`)
- uses the same RPC and streaming contracts as remote endpoints

2. Remote Endpoint Workspace
- endpoint runs on a network-hosted server
- uses the same RPC and streaming contracts

## Data Model Overview

Detailed model is maintained in `docs/entities.md`.

Core entities:

1. Workspace
2. Repository
3. RepositoryGroup
4. UnitTask
5. SubTask (child of UnitTask)
6. AgentSession (child of SubTask)
7. PullRequestTracking
8. ReviewAssistItem
9. BadgeTheme and ActionBadge mapping
10. Notification

## Task Execution Model

### UnitTask

UnitTask is the top-level user-visible work item.

### SubTask

SubTask is a UnitTask child entity used for initial implementation, retries, review feedback, and PR follow-up fixes.

### AgentSession

Each SubTask can run one or more AI coding agent sessions.

```
UnitTask
  ├── SubTask #1 (initial implementation)
  │     ├── AgentSession #1
  │     └── AgentSession #2 (retry)
  └── SubTask #2 (fix CI failure)
        └── AgentSession #3
```

## Worktree-Only Policy

DeliDev does not support editing directly against arbitrary local folders.

All code execution paths must:

1. resolve repository through workspace-scoped repository settings
2. materialize task-specific git worktree
3. execute agent operations in that worktree
4. persist patch and commit metadata
5. cleanup or archive worktree by retention policy

## PR Management

PR management is part of the standard lifecycle:

1. DeliDev tracks PRs created by DeliDev tasks.
2. Pollers fetch PR state, review comments, and CI status.
3. On actionable events (review requested changes, CI failure), UI shows `Fix with Agent`.
4. If auto-run is enabled, DeliDev starts remediation SubTask automatically.

See `docs/pr-management.md`.

## PR Review Assist

Review assist features include:

1. changed file prioritization
2. AI summaries and risk markers
3. review checklist and suggested questions
4. unresolved thread and CI signal aggregation

## Plan Mode Support

For coding agents with plan mode:

1. show plan proposal state
2. support explicit approve/revise/reject actions
3. stream plan updates and rationale
4. attach plan decisions to SubTask and AgentSession records

See `docs/plan-yaml.md`.

## Event Streaming

DeliDev uses event streaming for low-latency UI updates and automation triggers.
Main server uses Redis to propagate and replay events.

Event families:

1. task state
2. subtask lifecycle
3. session output and state
4. PR tracking state
5. review assist updates
6. notification triggers

See `docs/event-streaming.md`.

## Notification Architecture

Notification delivery uses Web Notification API.

- primary channel: notification API in Tauri webview context
- in-app notification center stores authoritative state

See `docs/notifications.md`.

## Mobile Strategy

iOS and Android are first-wave platforms.

Design implications:

1. core workflows must be API-driven and mobile-safe
2. no desktop-only business logic path
3. notification and streaming strategy must work with mobile runtime constraints
4. workspace onboarding and PR remediation actions must be touch-friendly

## Observability and Logging

Server-side structured logging is required for:

1. workspace routing
2. task and subtask state transitions
3. agent session start/stop/failure
4. PR polling snapshots and decision points
5. event stream delivery health

Client-side logging is required for:

1. stream disconnect and reconnect
2. notification permission and dispatch outcomes
3. user-triggered remediation actions

## Security Baseline

1. Connect RPC over TLS for non-localhost endpoints
2. token-based auth for shared remote workspaces
3. scoped secret usage with minimal runtime lifetime
4. strict validation for repository URLs, branch names, prompts, and review payloads

## Related Docs

1. `docs/entities.md`
2. `docs/api.md`
3. `docs/main-server.md`
4. `docs/worker-server.md`
5. `docs/tauri-app.md`
6. `docs/ui.md`
7. `docs/pr-management.md`
8. `docs/event-streaming.md`
9. `docs/workspace-connectivity.md`
10. `docs/notifications.md`
11. `docs/plan-yaml.md`
