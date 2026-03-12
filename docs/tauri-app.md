# Tauri App (Desktop + Mobile)

DexDex client runs in Tauri across desktop and mobile platforms.
The app is Connect RPC-first.

## Core Rule

Business communication uses Connect RPC as the primary path.
Tauri-specific APIs are only for platform integration.
Client consumes normalized coding-agent message contracts only.
The app does not use direct client-to-worker business APIs.

## Supported Platforms

1. Desktop: macOS, Windows, Linux
2. Mobile: iOS, Android

## Capability Rollout

Desktop provides the full authoring workflow.
Mobile uses phased rollout on the same contracts:

1. baseline: task monitoring, log viewing, plan decisions, and stop actions
2. expansion: broader remediation and review interaction flows

## Client Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Tauri App                                                  │
│                                                            │
│  React UI Layer                                            │
│   ├── Workspace shell                                      │
│   ├── UnitTask and SubTask views                           │
│   ├── PR management, review assist, and inline comments    │
│   └── Settings and notifications                           │
│                                                            │
│  Data Layer                                                │
│   ├── `@connectrpc/connect-query` RPC hooks                │
│   ├── Stream subscriber                                    │
│   └── React Query cache store (`@tanstack/react-query`)    │
│                                                            │
│  Tauri Bridge                                              │
│   ├── keychain wrappers                                    │
│   ├── file picker                                          │
│   ├── deep link handler                                    │
│   └── window lifecycle                                     │
└────────────────────────────────────────────────────────────┘
```

## Tab State Management

The client maintains workspace-scoped tab state in the UI store.

1. each opened item route is represented as a tab entry
2. tab order and active tab are persisted per workspace
3. draft form state is preserved while switching tabs
4. tab badge state tracks running/action-required/unread updates

## Workspace UX Model

The client exposes workspace switching.

Each workspace has:

1. endpoint URL
2. auth profile
3. workspace type (`LOCAL_ENDPOINT` or `REMOTE_ENDPOINT`)

A local workspace points to a locally running server endpoint.

## Event Streaming Client

The app maintains a stream subscription per active workspace:

1. connect to `EventStreamService.StreamWorkspaceEvents`
2. keep last applied sequence
3. reconnect with `from_sequence`
4. apply idempotent event reducers
5. merge `INLINE_COMMENT_UPDATED` events into review diff state
6. render `SESSION_OUTPUT` payloads from normalized `SessionOutputEvent` schema only

## Web Data Access Pattern

The web client uses `@connectrpc/connect-query` with React Query patterns.

1. all unary business RPCs are consumed via generated connect-query hooks
2. query keys are workspace-scoped to avoid cross-workspace cache leaks
3. mutations use React Query invalidation/update patterns for consistency
4. components do not call ad-hoc `fetch` for business APIs
5. stream events update or invalidate React Query caches for near-real-time views

## Notifications

Notification dispatch uses Web Notification API from the web layer.

Rules:

1. request permission during app startup from web layer
2. avoid duplicate notifications by sequence
3. keep in-app notification center authoritative

## Plan Mode UX Responsibilities

When a session enters plan wait state:

1. show plan proposal panel
2. allow `Approve`, `Revise`, `Reject`
3. submit decision through Task API
4. continue streaming results in the same SubTask timeline

## Inline Comment UX Responsibilities

The client handles inline comments in UnitTask and PR review diff views.

1. render line-level comment anchors from `ListInlineComments`
2. create comments with `ReviewCommentService.CreateInlineComment`
3. allow edit, resolve, reopen, and delete actions through review comment APIs
4. apply `INLINE_COMMENT_UPDATED` stream events to keep threads synchronized
5. keep inline comment drafts stable while switching tabs

## Multiline Submit Handling

The client implements unified multiline keyboard handling in the web layer:

1. `Enter` inserts newline in multiline input controls
2. `Cmd+Enter` submits the associated form
3. behavior is consistent across UnitTask, SubTask feedback, plan revise, review inputs, and inline comment composer
4. `Cmd+Enter` handling is stable regardless of current IME language mode

## Shortcut Registry and Scope

The client maintains a centralized shortcut registry.

1. global shortcuts are active across the app shell
2. screen-scoped shortcuts activate only when that screen is focused
3. shortcut collisions are resolved by scope priority (modal > screen > global)
4. every primary item action exposed in UI has an associated shortcut entry
5. each primary screen (Workspace Home, UnitTask Detail, PR Management, PR Review Assist, Settings, Notifications) registers its own shortcut set
6. shortcut matching uses `KeyboardEvent.code` + modifiers, not locale-dependent character output
7. shortcut execution is independent of Korean/English input mode switching
8. stop actions for in-progress UnitTask and SubTask are included in screen shortcut sets
9. tab management actions (`Cmd+T`, `Cmd+W`, `Cmd+Shift+[`, `Cmd+Shift+]`) are handled at app-shell scope
10. context-sensitive shortcuts (such as `Cmd+Enter`) are resolved by focused element role

## Approved Diff Create PR Action

When a user approves AI diff in UnitTask detail:

1. UI renders `Create PR` button
2. button triggers `TaskService.CreateSubTask`
3. request uses `type = PR_CREATE` and prompt `Create A PR`
4. resulting SubTask and AgentSession are streamed in existing task timeline
5. PR creation uses generated real commit chain from the SubTask

## Stop Running Task Actions

Client exposes immediate stop actions:

1. `CancelUnitTask` for running UnitTask
2. `CancelSubTask` for running SubTask
3. stream updates keep cancellation state synchronized in UI

## Offline and Recovery Behavior

1. temporary network loss: show degraded state and auto-retry stream
2. stream reconnection: fetch missed events by sequence
3. command retries: safe retry only for idempotent requests

## Settings Owned by Client

1. workspace list and active workspace pointer
2. badge color mapping editor
3. PR auto-fix preferences
4. notification permission and display preferences
5. appearance mode (Light, Dark, System)
6. shortcut discoverability and keymap preferences

Staged settings with security guardrails:

1. agent credential import/bridge flows
2. worker environment profile management with scoped secret handling

## Testing Focus

1. stream resume correctness by sequence
2. workspace switch isolation
3. notification deduplication
4. plan-mode decision loop behavior
5. mobile layout parity for task and PR remediation screens
