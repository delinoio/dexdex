# Tauri App (Desktop + Mobile)

DeliDev client runs in Tauri across desktop and mobile platforms.
The app is Connect RPC-first.

## Core Rule

Business communication uses Connect RPC as the primary path.
Tauri-specific APIs are only for platform integration.

## Supported Platforms

1. Desktop: macOS, Windows, Linux
2. Mobile: iOS, Android

## Client Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Tauri App                                                  │
│                                                            │
│  React UI Layer                                            │
│   ├── Workspace shell                                      │
│   ├── UnitTask and SubTask views                           │
│   ├── PR management and review assist                      │
│   └── Settings and notifications                           │
│                                                            │
│  Data Layer                                                │
│   ├── Connect RPC clients                                  │
│   ├── Stream subscriber                                    │
│   └── Query and cache store                                │
│                                                            │
│  Tauri Bridge                                              │
│   ├── keychain wrappers                                    │
│   ├── file picker                                          │
│   ├── deep link handler                                    │
│   └── window lifecycle                                     │
└────────────────────────────────────────────────────────────┘
```

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

## Notifications

Notification dispatch uses Web Notification API from the web layer.

Rules:

1. request permission explicitly
2. avoid duplicate notifications by sequence
3. keep in-app notification center authoritative

## Plan Mode UX Responsibilities

When a session enters plan wait state:

1. show plan proposal panel
2. allow `Approve`, `Revise`, `Reject`
3. submit decision through Task API
4. continue streaming results in the same SubTask timeline

## Multiline Submit Handling

The client implements unified multiline keyboard handling in the web layer:

1. `Enter` inserts newline in multiline input controls
2. `Cmd+Enter` submits the associated form
3. behavior is consistent across UnitTask, SubTask feedback, plan revise, and review inputs

## Shortcut Registry and Scope

The client maintains a centralized shortcut registry.

1. global shortcuts are active across the app shell
2. screen-scoped shortcuts activate only when that screen is focused
3. shortcut collisions are resolved by scope priority (modal > screen > global)
4. every primary item action exposed in UI has an associated shortcut entry
5. each primary screen (Workspace Home, UnitTask Detail, PR Management, PR Review Assist, Settings, Notifications) registers its own shortcut set

## Approved Diff Create PR Action

When a user approves AI diff in UnitTask detail:

1. UI renders `Create PR` button
2. button triggers `TaskService.CreateSubTask`
3. request uses `type = PR_CREATE` and prompt `Create A PR`
4. resulting SubTask and AgentSession are streamed in existing task timeline
5. PR creation uses generated real commit chain from the SubTask

## Offline and Recovery Behavior

1. temporary network loss: show degraded state and auto-retry stream
2. stream reconnection: fetch missed events by sequence
3. command retries: safe retry only for idempotent requests

## Settings Owned by Client

1. workspace list and active workspace pointer
2. badge color mapping editor
3. PR auto-fix preferences
4. notification permission and display preferences

## Testing Focus

1. stream resume correctness by sequence
2. workspace switch isolation
3. notification deduplication
4. plan-mode decision loop behavior
5. mobile layout parity for task and PR remediation screens
