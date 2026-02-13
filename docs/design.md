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

## Database Strategy

Main server supports PostgreSQL and SQLite.

1. PostgreSQL is the recommended default database.
2. SQLite is supported for local endpoint deployments.

## Connect RPC First Rule

All business data and control flows use Connect RPC.
Web client data access uses `@connectrpc/connect-query` with React Query patterns.

Tauri-native bindings are used only for platform integration:

1. window lifecycle and tray integration
2. secure local storage helpers
3. file picker and OS capabilities
4. deep links

Business operations (task, repository, workspace, PR, review, streaming, settings) are not Tauri-only contracts.

Web client rule:

1. unary RPCs are consumed through `@connectrpc/connect-query` query and mutation hooks
2. caching, refetch, and invalidation follow React Query (`@tanstack/react-query`) patterns
3. ad-hoc `fetch` or component-local RPC calls are not used for business data flows

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
9. ReviewInlineComment
10. BadgeTheme and ActionBadge mapping
11. Notification

## Task Execution Model

### UnitTask

UnitTask is the top-level user-visible work item.
Execution scope is one RepositoryGroup.

### SubTask

SubTask is a UnitTask child entity used for initial implementation, retries, review feedback, PR creation, and PR follow-up fixes.
SubTask is also used for small operational tasks triggered by UI actions.

### AgentSession

Each SubTask can run one or more AI coding agent sessions.

```
UnitTask
  ├── SubTask #1 (initial implementation)
  │     ├── AgentSession #1
  │     └── AgentSession #2 (retry)
  ├── SubTask #2 (create PR)
  │     └── AgentSession #3
  └── SubTask #3 (fix CI failure)
        └── AgentSession #4
```

## Agent Message Normalization Boundary

Worker server is the normalization boundary for all coding-agent outputs.

1. agent adapters parse provider-native outputs (Claude Code, Codex, OpenCode, and others) inside worker runtime
2. worker emits only normalized session messages and normalized session state events to main server
3. main server stores and relays only normalized agent messages
4. Tauri client renders and reacts only to normalized message contracts
5. provider-native raw payloads are not part of main-server or client contracts

## Commit Chain Invariant

Worker-produced code changes must be represented as real git commits.

1. Every SubTask that changes code produces one or more real commits in the task worktree branch.
2. Multi-step changes should be split into multiple commits, not squashed into one patch-only result.
3. Commit order is preserved and stored as SubTask commit chain metadata.
4. PR creation and Commit to Local use this commit chain as the source of truth.
5. Patch artifacts are derived from commits for diff viewing and are not authoritative.

## Worktree-Only Policy

DeliDev does not support editing directly against arbitrary local folders.

All code execution paths must:

1. resolve repository through workspace-scoped repository settings
2. materialize task-specific git worktrees for each repository in the target RepositoryGroup
3. execute agent operations from the first repository worktree and attach other repository worktrees via `--add-dir` or equivalent option
4. persist real git commit chain and commit metadata
5. cleanup or archive worktree by retention policy

## RepositoryGroup Execution Rule

RepositoryGroup is the execution unit for agent runs.

1. Worker creates one worktree per repository in the group.
2. Repository order is preserved from `repositoryIds`.
3. Agent process starts in the first repository worktree.
4. Additional repositories are passed as extra directories using `--add-dir` (or agent-equivalent flags).

## PR Management

PR management is part of the standard lifecycle:

1. DeliDev tracks PRs created by DeliDev tasks.
2. When a user approves AI diff on a UnitTask, UI shows a `Create PR` button.
3. Clicking `Create PR` creates a SubTask with type `PR_CREATE` and prompt `Create A PR`.
4. PR creation uses the SubTask real commit chain, not a synthetic patch-only payload.
5. Commit to Local also applies the same commit chain into the destination repository.
6. Pollers fetch PR state, review comments, and CI status.
7. On actionable events (review requested changes, CI failure), UI shows `Fix with Agent`.
8. If auto-run is enabled, DeliDev starts remediation SubTask automatically.

See `docs/pr-management.md`.

## PR Review Assist

Review assist features include:

1. changed file prioritization
2. AI summaries and risk markers
3. review checklist and suggested questions
4. unresolved thread and CI signal aggregation
5. line-level inline comments in code review diff

## Inline Comment Requirement

Code review UI provides inline comments anchored to diff lines.

1. users can add inline comments on specific file and line positions
2. inline comments can be resolved and reopened through review workflow
3. inline comment updates are streamed in real time
4. inline comments are used as input context for `Request Changes` and related SubTasks

## Stop Running Work

Users can easily stop running work at both UnitTask and SubTask levels.

1. in-progress UnitTask provides a direct stop action.
2. in-progress SubTask provides a direct stop action.
3. stop requests propagate immediately to worker session runners.
4. cancelled items transition to `CANCELLED` and emit stream updates.

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
6. inline comment updates
7. notification triggers

See `docs/event-streaming.md`.

## Notification Architecture

Notification delivery uses Web Notification API.

- primary channel: notification API in Tauri webview context
- in-app notification center stores authoritative state
- permission request is initiated during app startup

See `docs/notifications.md`.

## Mobile Strategy

iOS and Android are first-wave platforms.

Design implications:

1. core workflows must be API-driven and mobile-safe
2. no desktop-only business logic path
3. notification and streaming strategy must work with mobile runtime constraints
4. workspace onboarding and PR remediation actions must be touch-friendly

## Multi-Tab UI Invariant

The client provides multi-tab UI so users can work on multiple items concurrently.

1. detail views and workflow pages open as tabs in the app shell
2. tab state is persisted per workspace
3. tab switching does not discard running context or unsaved draft input
4. tab state indicators surface running/action-required/unread-update signals

## Keyboard Input Rule

All multiline form inputs must support `Cmd+Enter` submit behavior.
This is a product-wide interaction rule and applies across task, plan, review, and PR workflows.
Shortcut handling must be independent of current language input mode (Korean/English IME).

## Screen Shortcut Invariant

Every screen includes appropriate keyboard shortcuts for its primary items and actions.

1. list navigation and item-open shortcuts are required on list/detail screens
2. primary action buttons require direct keyboard shortcuts
3. shortcuts must be discoverable in UI labels, tooltips, or a shortcut cheat sheet
4. all primary screens are covered: Workspace Home, UnitTask Detail, PR Management, PR Review Assist, Settings, Notifications Center
5. shortcut matching uses physical key codes and modifiers so behavior is stable across input language modes
6. tab management shortcuts are required (`new`, `close`, `previous`, `next`)

## Observability and Logging

Server-side structured logging is required for:

1. workspace routing
2. task and subtask state transitions
3. agent session start/stop/failure
4. agent token usage and cost snapshots
5. PR polling snapshots and decision points
6. event stream delivery health

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
