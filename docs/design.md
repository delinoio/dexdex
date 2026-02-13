# DeliDev Design (To-Be, Rewrite Baseline)

DeliDev is being rebuilt around a Connect RPC-first architecture with Tauri clients and Go servers.
This document defines the target design baseline used by all other docs.

## Status

- Document type: To-be specification for rewrite
- Current implementation may differ
- Source of truth for data model: `docs/entities.md`
- Source of truth for RPC contract: `docs/api.md`

## Product Goals

1. Keep Tauri as the desktop and mobile app container.
2. Rewrite server stack to Go.
3. Remove mode-centric UX language and use `Workspace` as the user-facing concept.
4. Remove `CompositeTask` from the product model.
5. Support UnitTask-centric workflows with nested SubTask and AgentSession history.
6. Make PR management and PR review assist first-class.
7. Provide real-time event streaming for UI updates and automation.
8. Preserve future parity targets for iOS and Android from day one.

## Non-Goals

1. Direct local folder execution without worktree.
2. Tauri-invoke-first API design.
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

## Monorepo Structure (Target)

- `apps/main-server/` (Go)
- `apps/worker-server/` (Go)
- `apps/tauri-app/` (Tauri + React)
- Single root `go.mod` at repository root (shared module for server apps)

## Connect RPC First Rule

All business data and control flows must use Connect RPC.

Tauri-native bindings are allowed only for platform integration needs:

1. window lifecycle and tray integration
2. secure local storage helpers
3. file picker / OS capabilities
4. deep links

Business operations (task, repo, workspace, PR, review, streaming, settings) are not defined as Tauri-only contracts.

## Workspace Model (Replaces Mode Model)

DeliDev no longer presents Local Mode vs Remote Mode.

Instead, users create and switch between Workspaces that point to server endpoints.

### Workspace Types

1. Local Workspace
- Endpoint points to a server running on the same device (for example `http://127.0.0.1:4621`)
- Uses the same Remote protocol path as any other workspace
- "Local" means endpoint locality, not a separate protocol/runtime path

2. Remote Workspace
- Endpoint points to shared network-hosted main server
- Same RPC contracts and streaming contracts

## Data Model Overview

Detailed model is maintained in `docs/entities.md`.

Core entities:

1. Workspace
2. Repository / RepositoryGroup
3. UnitTask
4. SubTask (child of UnitTask)
5. AgentSession (child of SubTask)
6. PullRequestTracking
7. ReviewAssistItem
8. BadgeTheme / ActionBadge

Deprecated entity:

- CompositeTask (removed from active design)

## Task Execution Model

### UnitTask

UnitTask is the top-level user-visible work item.

### SubTask

SubTask is a UnitTask child entity used for retries, review feedback application, PR follow-up fixes, and other bounded actions.

### AgentSession

Each SubTask can run one or more AI coding agent sessions.
This preserves full timeline/history of retries and strategy changes.

```
UnitTask (user visible)
  ├── SubTask #1 (initial implementation)
  │     ├── AgentSession #1
  │     └── AgentSession #2 (retry)
  └── SubTask #2 (fix CI failure)
        └── AgentSession #3
```

## Worktree-Only Policy

DeliDev does not support editing directly against an arbitrary local folder.

All code execution paths must:

1. resolve repository through workspace-scoped repository settings
2. materialize task-specific git worktree
3. execute agent operations in that worktree
4. persist patch/commit metadata
5. cleanup or archive worktree per retention policy

## PR Management (Polling-Based)

PR management is built into the standard lifecycle:

1. DeliDev tracks PRs created by DeliDev tasks.
2. Polling workers fetch PR state, review comments, and CI status.
3. On actionable events (review requested changes, CI failure), UI surfaces "Fix with Agent".
4. If auto-run is enabled in settings, DeliDev starts remediation SubTask automatically.

Detailed flow lives in `docs/pr-management.md`.

## PR Review Assist

DeliDev includes review-assist features for user-owned review activity:

1. changed file prioritization
2. AI summaries and risk markers
3. suggested review checklist and questions
4. unresolved thread and CI signal aggregation

## Plan Mode Support

When using coding agents that expose Plan Mode, DeliDev must support the full plan loop:

1. present plan proposal state
2. allow explicit approve/revise/reject actions
3. stream plan updates and rationale
4. attach final approved plan metadata to SubTask/AgentSession records

Plan-mode protocol details are in `docs/plan-yaml.md`.

## Event Streaming

DeliDev uses event streaming for low-latency UI state updates and automation triggers.

Event families:

1. task state
2. subtask lifecycle
3. session/log stream
4. PR tracking state
5. review assist updates
6. notification triggers

Detailed stream contract is in `docs/event-streaming.md`.

## Notification Architecture

Notification delivery is based on Web Notification API.

- Primary channel: browser-compatible notification API in Tauri webview context
- Native plugin notifications are not the primary path in this design

See `docs/notifications.md`.

## Mobile Strategy (iOS + Android)

iOS and Android are first-wave targets, not post-MVP stretch goals.

Design implications:

1. all core workflows must be API-driven and mobile-safe
2. no desktop-only business logic path
3. notification and streaming strategy must work on mobile WebView/runtime constraints
4. workspace onboarding and PR remediation actions must be usable on touch UIs

## Observability and Logging

Server-side structured logging is required for:

1. workspace routing
2. task/subtask state transitions
3. agent session start/stop/failure
4. PR polling snapshots and decision points
5. event stream delivery health

Client-side telemetry/logging is required for:

1. stream disconnect/reconnect
2. notification permission and dispatch outcomes
3. user-triggered fix actions

## Security Baseline

1. Connect RPC over TLS for non-localhost endpoints
2. token-based auth for shared remote workspaces
3. scoped secret usage with minimal lifetime in worker runtime
4. strict validation for repository URLs, branch names, prompts, and review payloads

## Deprecations

- `CompositeTask` is removed from active product design.
- Legacy references may remain only in migration history notes.

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
