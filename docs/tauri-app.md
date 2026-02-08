# Tauri App

The DeliDev client is built with Tauri, providing a cross-platform desktop and mobile application for orchestrating AI coding agents.

## Platforms

| Platform | Mode Support | Notes |
|----------|--------------|-------|
| **Desktop** | Local + Remote | Windows, macOS, Linux |
| **Mobile** | Remote only | iOS, Android |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri App                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Frontend (WebView)                     │   │
│  │                                                          │   │
│  │   React + TypeScript + TailwindCSS + shadcn/ui          │   │
│  │                                                          │   │
│  │   ┌─────────────────┐  ┌─────────────────────────────┐  │   │
│  │   │  react-query    │  │    Zustand State            │  │   │
│  │   │  (Server State) │  │    (Client State)           │  │   │
│  │   └────────┬────────┘  └─────────────────────────────┘  │   │
│  │            │                                             │   │
│  │            ▼                                             │   │
│  │   ┌─────────────────────────────────────────────────┐   │   │
│  │   │              API Layer                           │   │   │
│  │   │  - Mode detection (local vs remote)              │   │   │
│  │   │  - Tauri invoke (local) or Connect RPC (remote)  │   │   │
│  │   └─────────────────────────────────────────────────┘   │   │
│  └────────────────────────┬────────────────────────────────┘   │
│                           │                                     │
│                           │ Tauri Commands                      │
│                           ▼                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Rust Backend                           │   │
│  │                                                          │   │
│  │   ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │   │
│  │   │  Commands   │  │  Services   │  │ Single Process  │ │   │
│  │   │  (API)      │  │             │  │ Runtime         │ │   │
│  │   └─────────────┘  └─────────────┘  └─────────────────┘ │   │
│  │                                                          │   │
│  │   ┌─────────────────┐  ┌─────────────────────────────┐  │   │
│  │   │  Keychain       │  │    Notification             │  │   │
│  │   │  (Secrets)      │  │    Service                  │  │   │
│  │   └─────────────────┘  └─────────────────────────────┘  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## Single Process Mode

Desktop apps can run in single-process mode, embedding both Server and Worker:

### Implementation

```
apps/tauri-app/src-tauri/src/single_process/
├── mod.rs              # SingleProcessRuntime orchestration
├── runtime.rs          # SingleProcessRuntime implementation (task store, executor)
├── executor.rs         # LocalExecutor wrapping TaskExecutor from coding_agents
└── tty_handler.rs      # Re-exports TtyInputRequestManager from coding_agents
```

The core execution logic is in `crates/coding_agents/src/executor/`:
```
crates/coding_agents/src/executor/
├── mod.rs              # Module exports
├── emitter.rs          # EventEmitter trait and event types
├── task_executor.rs    # TaskExecutor with platform-agnostic execution
└── tty_manager.rs      # TtyInputRequestManager and EventEmitterTtyHandler
```

The Tauri app implements `EventEmitter` via `TauriEventEmitter` to emit events through Tauri's event system.

### Task Execution Flow

When a unit task is created in local mode:

1. **Task Creation**: `create_unit_task` command creates task and agent session in SQLite
2. **Worktree Setup**: `LocalExecutor` creates git worktree from cached repository
3. **Agent Execution**: Claude Code (or other agent) spawned with `--print --verbose --output-format stream-json`
4. **Event Streaming** (purely event-driven, no polling):
   - Stdout/stderr parsed into `NormalizedEvent` types
   - Events emitted via Tauri `agent-output` channel for real-time display
   - `useTaskLogs` hook fetches historical logs once on mount, then streams via events
   - Events incrementally persisted to `agent_session.output_log` (every 10 events)
   - Final persistence on task completion ensures all events are stored
5. **Completion**: Task status updated to `InReview` when agent finishes
   - `task-status-changed` event emitted, `useTaskStatusEvents` invalidates react-query caches

> **Note**: Interactive TTY input (e.g., `AskUserQuestion` prompts) is not currently supported. Agents run in non-interactive mode with stdin closed.

### Behavior

| Aspect | Single Process Mode | Remote Mode |
|--------|---------------------|-------------|
| RPC | Direct function calls | Connect RPC over HTTP |
| Database | SQLite | PostgreSQL (on server) |
| Worker | Embedded | Remote Worker Server |
| Auth | Disabled | JWT + OIDC |
| Secrets | Direct keychain access | Sent to server |
| Network | No network required | Requires connection |

### Mode Detection

```typescript
// Frontend detects mode from Tauri
const mode = await invoke<'local' | 'remote'>('get_mode');

// API calls route based on mode
if (mode === 'local') {
  // Use Tauri invoke
  return invoke('task.createUnit', params);
} else {
  // Use Connect RPC
  return taskService.createUnit(params);
}
```

## Frontend Structure

```
apps/tauri-app/src/
├── api/
│   ├── client-config.ts    # Mode configuration
│   ├── ClientProvider.tsx  # React context for client state
│   ├── hooks.ts            # react-query hooks
│   └── rpc.ts              # Connect RPC client
├── components/
│   ├── ui/                 # shadcn/ui components
│   ├── chat/               # Chat interface components
│   │   ├── ChatWindow.tsx  # Main chat window (modal overlay)
│   │   ├── ChatInput.tsx   # Text input with send button
│   │   └── MessageList.tsx # Message history display
│   ├── dashboard/          # Dashboard components
│   ├── task/               # Task-related components
│   ├── review/             # Review interface (InlineComment, DiffViewer)
│   └── settings/           # Settings components
├── hooks/
│   ├── useKeyboardShortcuts.ts
│   ├── useNotificationClickHandler.ts
│   ├── useNotificationEvents.ts # Populates notification center from Tauri events
│   ├── useReviewComments.ts     # Inline comment state management
│   ├── useTaskLogs.ts           # Event-driven task log streaming (no polling)
│   ├── useTaskStatusEvents.ts   # Invalidates react-query caches on task events
│   ├── useTasks.ts
│   ├── useTtyInput.ts           # TTY input request handling
│   └── ...
├── pages/
│   ├── Dashboard.tsx
│   ├── UnitTaskDetail.tsx
│   ├── CompositeTaskDetail.tsx
│   ├── Settings.tsx
│   └── ...
├── stores/
│   ├── chatStore.ts        # Chat state (messages, isOpen)
│   ├── uiStore.ts          # UI state (tabs, dialogs)
│   └── ...                 # Other Zustand stores
└── App.tsx
```

### API Layer

The API layer abstracts communication, supporting both modes:

```typescript
// api/hooks.ts
export function useCreateUnitTask() {
  const { mode, serverUrl } = useClientConfig();

  return useMutation({
    mutationFn: async (params: CreateUnitTaskParams) => {
      if (mode === 'local') {
        return invoke<UnitTask>('create_unit_task', params);
      } else {
        return rpcClient.call('task.createUnit', params);
      }
    },
  });
}
```

### State Management

| State Type | Tool | Example |
|------------|------|---------|
| Server State | react-query | Tasks, repositories, settings |
| UI State | Zustand | Selected tab, collapsed panels |
| Form State | React Hook Form | Task creation, settings forms |

## Keychain Access

The app accesses the native keychain for secret storage:

### Supported Platforms

| Platform | Backend |
|----------|---------|
| macOS | Keychain Services (security-framework) |
| Windows | Windows Credential Manager |
| Linux | Secret Service (libsecret/KWallet) |

### Tauri Commands

```rust
#[tauri::command]
async fn get_secret(key: String) -> Result<Option<String>, String>;

#[tauri::command]
async fn set_secret(key: String, value: String) -> Result<(), String>;

#[tauri::command]
async fn delete_secret(key: String) -> Result<(), String>;

#[tauri::command]
async fn list_secrets() -> Result<Vec<String>, String>;
```

### Secret Keys

| Key | Description |
|-----|-------------|
| `CLAUDE_CODE_OAUTH_TOKEN` | Claude Code OAuth token |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GOOGLE_AI_API_KEY` | Google AI API key |
| `GITHUB_TOKEN` | GitHub access token |

### Secret Flow (Remote Mode)

```
1. User starts task
         ▼
2. Frontend calls sendSecrets command
         ▼
3. Tauri reads secrets from keychain
         ▼
4. Secrets sent to Main Server via RPC
         ▼
5. Main Server caches secrets (in-memory)
         ▼
6. Worker retrieves secrets when task starts
         ▼
7. Worker injects secrets as env vars
         ▼
8. Main Server clears secrets on task completion
```

## Notification System

Desktop notifications alert users when AI agents need attention.

### Notification Triggers

| Event | Notification |
|-------|--------------|
| TTY Input Request | "Agent is asking a question" |
| Task Review Ready | "Task ready for review" |
| Plan Approval | "Plan ready for approval" |
| Task Failure | "Task failed" |

### Platform Implementation

| Platform | Implementation |
|----------|----------------|
| Windows | `tauri-winrt-notification` with click handler |
| Linux | `notify-rust` with D-Bus action support |
| macOS | AppleScript (native delegate TODO) |

### Click Handling

```rust
// Backend emits event when notification clicked
app.emit("notification-clicked", NotificationPayload {
    task_type: "unit_task",
    task_id: "...",
});

// Frontend handles navigation
useEffect(() => {
    listen("notification-clicked", (event) => {
        const { task_type, task_id } = event.payload;
        if (task_type === "unit_task") {
            navigate(`/unit-tasks/${task_id}`);
        } else if (task_type === "composite_task") {
            navigate(`/composite-tasks/${task_id}`);
        }
    });
}, []);
```

## Global Hotkey

The app registers a global hotkey for quick access:

### Default Hotkey

| Platform | Hotkey |
|----------|--------|
| macOS | `Option+Z` |
| Windows/Linux | `Alt+Z` |

### Configuration

Users can customize the hotkey in settings (`~/.delidev/config.toml`):

```toml
[hotkey]
openChat = "Option+Z"
```

### Behavior

When hotkey is pressed:
1. App window is brought to focus (or opened if minimized)
2. Chat interface is shown
3. Input is focused for immediate typing

## Mobile Considerations

### Remote Mode Only

Mobile apps only support remote mode because:
- No Docker runtime on mobile
- Limited file system access
- Battery and resource constraints
- Git operations require full file system access

### Mobile Features

| Feature | Availability |
|---------|--------------|
| View tasks | Yes |
| Create tasks | Yes |
| Review code | Yes (read-only diff view) |
| Approve/Reject | Yes |
| TTY Input Response | Yes |
| Repository management | Limited (can view, not add) |
| Settings | Yes |

### Platform-Specific

| Feature | iOS | Android |
|---------|-----|---------|
| Keychain | Keychain Services | Android Keystore |
| Notifications | APNs | FCM |
| Biometric Auth | Face ID / Touch ID | Fingerprint / Face |

## Development

### Environment Variables

| Variable | Description |
|----------|-------------|
| `PUBLIC_DEFAULT_MODE` | Default mode: `local` or `remote` |
| `PUBLIC_REMOTE_SERVER_URL` | Remote server URL |
| `PUBLIC_SKIP_MODE_SELECTION` | Skip mode selection screen |

### Scripts

```bash
# Default dev mode (shows mode selection)
pnpm dev

# Local mode (skip selection)
pnpm dev:local

# Remote mode (requires server URL)
PUBLIC_REMOTE_SERVER_URL=http://localhost:54871 pnpm dev:remote
```

### Build

```bash
# Desktop
pnpm tauri build

# iOS
pnpm tauri ios build

# Android
pnpm tauri android build
```

## Tauri Commands

### Task Management

```rust
#[tauri::command]
async fn create_unit_task(params: CreateUnitTaskParams) -> Result<UnitTask, Error>;

#[tauri::command]
async fn create_composite_task(params: CreateCompositeTaskParams) -> Result<CompositeTask, Error>;

#[tauri::command]
async fn get_task(task_id: String) -> Result<Task, Error>;

#[tauri::command]
async fn list_tasks(filters: TaskFilters) -> Result<Vec<Task>, Error>;

#[tauri::command]
async fn approve_task(task_id: String) -> Result<(), Error>;

#[tauri::command]
async fn reject_task(task_id: String) -> Result<(), Error>;

#[tauri::command]
async fn request_changes(task_id: String, feedback: String) -> Result<(), Error>;

#[tauri::command]
async fn dismiss_approval(task_id: String) -> Result<(), Error>;

#[tauri::command]
async fn create_pr(task_id: String) -> Result<String, Error>;

#[tauri::command]
async fn commit_to_local(task_id: String, local_path: String) -> Result<(), Error>;

#[tauri::command]
async fn delete_task(task_id: String) -> Result<(), Error>;

#[tauri::command]
async fn fix_ci(task_id: String, ci_logs: Option<String>) -> Result<(), Error>;

#[tauri::command]
async fn reflect_reviews(task_id: String, review_comments: Option<String>) -> Result<(), Error>;

#[tauri::command]
async fn get_pr_status(task_id: String) -> Result<PrStatusResponse, Error>;
```

### PR Status Polling

The `get_pr_status` command queries the GitHub API to determine:
- **CI failures**: Whether any check runs or commit statuses have failed on the PR head commit
- **Reviews**: Whether the PR has any non-pending reviews

The frontend polls this endpoint every 30 seconds for tasks in `PrOpen` status. The "Fix CI Failures" button only appears when CI has failed, and the "Reflect PR Reviews" button only appears when reviews exist.

Requires a `GITHUB_TOKEN` in the keychain or environment to access the GitHub API.

### Repository Management

```rust
#[tauri::command]
async fn add_repository(path: String) -> Result<Repository, Error>;

#[tauri::command]
async fn list_repositories() -> Result<Vec<Repository>, Error>;

#[tauri::command]
async fn remove_repository(id: String) -> Result<(), Error>;
```

### Settings

```rust
#[tauri::command]
async fn get_global_settings() -> Result<GlobalSettings, Error>;

#[tauri::command]
async fn update_global_settings(settings: GlobalSettings) -> Result<(), Error>;

#[tauri::command]
async fn get_repository_settings(repo_id: String) -> Result<RepositorySettings, Error>;

#[tauri::command]
async fn update_repository_settings(repo_id: String, settings: RepositorySettings) -> Result<(), Error>;
```

### Secrets

```rust
#[tauri::command]
async fn get_secret(key: String) -> Result<Option<String>, Error>;

#[tauri::command]
async fn set_secret(key: String, value: String) -> Result<(), Error>;

#[tauri::command]
async fn send_secrets(task_id: String) -> Result<(), Error>;
```

### Mode

```rust
#[tauri::command]
async fn get_mode() -> Result<Mode, Error>;

#[tauri::command]
async fn set_mode(mode: Mode, server_url: Option<String>) -> Result<(), Error>;
```
