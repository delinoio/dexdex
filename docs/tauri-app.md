# Tauri App (Desktop + Mobile) - To-Be Design

DeliDev client runs in Tauri across desktop and mobile targets.
The app is Connect RPC-first.

## Core Rule

Business communication uses Connect RPC as the primary path.
Tauri-specific APIs are only for platform integration features.

## Supported Targets

1. Desktop: macOS, Windows, Linux
2. Mobile: iOS, Android

Mobile is a first-wave target in this rewrite.

## Client Architecture

```
┌────────────────────────────────────────────────────────────┐
│ Tauri App                                                   │
│                                                            │
│  React UI Layer                                             │
│   ├── Workspace shell                                       │
│   ├── UnitTask + SubTask views                              │
│   ├── PR management and review assist                       │
│   └── Settings + notifications                              │
│                                                            │
│  Data Layer                                                  │
│   ├── Connect RPC clients                                   │
│   ├── Stream subscriber                                     │
│   └── Query/cache store                                     │
│                                                            │
│  Tauri Bridge (minimal business scope)                      │
│   ├── keychain wrappers                                     │
│   ├── file picker                                            │
│   ├── deep link handler                                     │
│   └── window lifecycle                                       │
└────────────────────────────────────────────────────────────┘
```

## Workspace UX Model

The client does not expose "mode" switching.
It exposes workspace switching.

Each workspace has:

1. endpoint URL
2. auth profile
3. workspace type (local endpoint vs remote endpoint)

A local workspace is implemented by pointing to a locally running server endpoint.

## Event Streaming Client

The app maintains a streaming subscription per active workspace:

1. connect to `EventStreamService.StreamWorkspaceEvents`
2. keep last applied sequence
3. reconnect with `from_sequence`
4. apply idempotent event reducers

## Notifications

Notification dispatch uses Web Notification API from the web layer.

Rules:

1. request permission explicitly
2. do not emit duplicate notifications for same event sequence
3. preserve in-app notification center state

## Plan Mode UX Responsibilities

When a session enters plan wait state:

1. show plan proposal panel
2. allow `Approve`, `Revise`, `Reject`
3. submit decision through Task API
4. continue streaming results in the same SubTask timeline

## Offline and Recovery Behavior

1. temporary network loss: show degraded state and auto-retry stream
2. stream reconnection: fetch missed events by sequence
3. command retries: safe retry only for idempotent requests

## Settings Owned by Client

1. workspace list and active workspace pointer
2. badge color mapping editor
3. PR auto-fix preferences UI
4. notification permission and in-app display preferences

## Testing Focus

1. stream resume correctness by sequence
2. workspace switch isolation
3. notification deduplication
4. plan-mode decision loop behavior
5. mobile layout parity for task and PR remediation screens
