# Local vs Remote Workspaces

DeliDev supports two kinds of workspaces: **Local** and **Remote**. Each workspace has its own `kind` field that determines how operations on that workspace's contents (repositories, tasks) are routed. A single app instance can have both local and remote workspaces simultaneously. Mobile clients only support remote workspaces.

## Workspace Kind Comparison

| Aspect | Local Workspace | Remote Workspace |
|--------|----------------|-----------------|
| **Architecture** | Single process | Distributed |
| **Server** | Embedded | Remote Main Server |
| **Worker** | Embedded | Remote Worker Server(s) |
| **Database** | SQLite (local) | PostgreSQL (remote) |
| **Authentication** | Disabled | JWT + OIDC |
| **Network** | Not required | Required |
| **Secrets** | Direct keychain | Sent via RPC |
| **Docker** | Local machine | Worker machine |
| **Collaboration** | Single user | Multi-user |

## Platform Support

| Platform | Local Workspace | Remote Workspace |
|----------|----------------|-----------------|
| Desktop (Windows) | Yes | Yes |
| Desktop (macOS) | Yes | Yes |
| Desktop (Linux) | Yes | Yes |
| Mobile (iOS) | No | Yes |
| Mobile (Android) | No | Yes |

## Local Workspaces

### When to Use

- **Solo development**: Working alone on projects
- **Offline work**: No internet connection available
- **Privacy**: Code stays on local machine
- **Low latency**: No network round trips
- **Simple setup**: No server configuration needed

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Desktop App (Single Process)                  │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │   Embedded Server   │  │   Embedded Worker   │              │
│  │                     │  │                     │              │
│  │  - Task store       │  │  - AI agent exec    │              │
│  │  - Local SQLite     │  │  - Docker mgmt      │              │
│  │  - No auth          │  │  - Git worktrees    │              │
│  └─────────────────────┘  └─────────────────────┘              │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Frontend (WebView)                    │   │
│  │  Tauri invoke() ──► Direct function calls                │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Local Resources                         │   │
│  │  SQLite │ Keychain │ Docker │ Git Repos                  │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
User creates task
        ▼
Frontend calls Tauri command
        ▼
Embedded Server stores in SQLite
        ▼
Embedded Worker picks up task
        ▼
Worker reads secrets from local keychain
        ▼
Worker runs Docker container locally
        ▼
AI agent executes
        ▼
Results stored in SQLite
        ▼
Frontend updates via Tauri events
```

### Characteristics

| Feature | Behavior |
|---------|----------|
| Startup | Fast, no connection needed |
| Auth | None (trusted local user) |
| Data | SQLite in app data directory |
| Secrets | Read directly from OS keychain |
| Docker | Uses local Docker/Podman |
| Git | Direct access to local repos |
| Concurrency | Single worker (configurable) |

## Remote Workspaces

### When to Use

- **Team collaboration**: Multiple users sharing tasks
- **Resource offloading**: Execute on powerful servers
- **Mobile access**: View and manage tasks from phone
- **Centralized management**: Single source of truth
- **Scalability**: Multiple workers for parallel execution

### Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Client (Desktop/Mobile)                     │
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Frontend (WebView)                    │   │
│  │  react-query ──► Connect RPC ──► Main Server            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────┐                                       │
│  │  Local Keychain     │  (Secrets sent to server)             │
│  └─────────────────────┘                                       │
│                                                                 │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                    Connect RPC over HTTPS
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────┐
│                         Main Server                              │
│                                                                 │
│  ┌───────────┐  ┌───────────┐  ┌───────────────────────────┐   │
│  │  Auth     │  │  Task     │  │  Worker                   │   │
│  │  Module   │  │  Store    │  │  Registry                 │   │
│  └───────────┘  └───────────┘  └───────────────────────────┘   │
│                                                                 │
│  PostgreSQL Database                                            │
│                                                                 │
└───────────────────────────────┬─────────────────────────────────┘
                                │
                                ▼
    ┌───────────────────────────────────────────────────────────┐
    │                    Worker Server(s)                        │
    │                                                            │
    │  ┌────────────┐  ┌────────────┐  ┌────────────┐           │
    │  │  Worker 1  │  │  Worker 2  │  │  Worker N  │           │
    │  │  (Docker)  │  │  (Docker)  │  │  (Docker)  │           │
    │  └────────────┘  └────────────┘  └────────────┘           │
    │                                                            │
    └───────────────────────────────────────────────────────────┘
```

### Data Flow

```
User creates task
        ▼
Frontend sends Connect RPC request
        ▼
Main Server authenticates (JWT)
        ▼
Main Server stores in PostgreSQL
        ▼
Main Server notifies available Worker
        ▼
Client sends secrets to Main Server
        ▼
Main Server relays secrets to Worker
        ▼
Worker runs Docker container
        ▼
AI agent executes
        ▼
Worker reports status to Main Server
        ▼
Main Server broadcasts to clients
        ▼
Frontend updates via WebSocket events
```

### Characteristics

| Feature | Behavior |
|---------|----------|
| Startup | Requires connection to server |
| Auth | JWT token from OIDC login |
| Data | PostgreSQL on Main Server |
| Secrets | Sent from client to server to worker |
| Docker | Runs on Worker Server machines |
| Git | Workers clone/access repos |
| Concurrency | Multiple workers in parallel |

## Workspace Creation

### Desktop

Users choose workspace kind on first launch (or when adding a new workspace):

```
┌────────────────────────────────────────────────────────────────┐
│                     Welcome to DeliDev                           │
│              Choose how to set up your workspace                 │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  [Monitor Icon]  Local Workspace                          │  │
│  │                                                          │  │
│  │  Run everything locally on your machine.                 │  │
│  │  • Full privacy - code stays on your machine             │  │
│  │  • No network latency                                    │  │
│  │  • Works offline                                         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │  [Server Icon]  Remote Workspace                          │  │
│  │                                                          │  │
│  │  Connect to a remote DeliDev server.                     │  │
│  │  • Team collaboration                                    │  │
│  │  • Offload computation to server                         │  │
│  │  • Access from multiple devices                          │  │
│  │                                                          │  │
│  │  Server URL: [ https://delidev.example.com       ]       │  │
│  │                                   [Test Connection]       │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│  You can add more workspaces later                             │
│                                            [Continue →]        │
└────────────────────────────────────────────────────────────────┘
```

A single app instance can have multiple workspaces of different kinds. Users can switch between workspaces via the workspace selector in the sidebar.

### Mobile

Mobile apps automatically create remote workspaces:

```
┌────────────────────────────────────────────────────────────────┐
│                     Connect to Server                           │
│                                                                │
│  Enter your DeliDev server URL:                                │
│                                                                │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ https://delidev.example.com                              │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                │
│                    [Test Connection]                           │
│                                                                │
│                                            [Continue →]        │
└────────────────────────────────────────────────────────────────┘
```

## Why Mobile Only Supports Remote Workspaces

Mobile platforms have fundamental limitations that prevent local workspaces:

### Technical Limitations

| Limitation | Impact |
|------------|--------|
| **No Docker** | Cannot run containerized AI agents |
| **File System** | Limited/sandboxed access to files |
| **Background Execution** | OS kills long-running tasks |
| **Resource Constraints** | CPU, memory, battery limits |
| **Git Operations** | Require full file system access |

### User Experience

| Factor | Local (if possible) | Remote |
|--------|---------------------|--------|
| Battery | Heavy drain | Minimal impact |
| Storage | Large repos consume space | Only metadata |
| Heat | CPU-intensive | Offloaded |
| Responsiveness | Degraded during execution | Always responsive |

### Practical Usage

Mobile is best suited for:
- Monitoring task progress
- Reviewing code changes
- Approving/rejecting tasks
- Responding to agent questions
- Quick task creation

Heavy lifting happens on remote servers:
- AI agent execution
- Docker container management
- Git operations
- Large file handling

## Workspace Management

### Adding Workspaces

Users can add new workspaces at any time via the workspace management UI. Each workspace can be either local or remote, allowing a mix of both.

### Workspace Routing

All workspace metadata (name, kind, server_url, etc.) is stored locally in SQLite. When performing operations on a workspace's contents (repositories, tasks), the workspace's `kind` field determines routing:

- **Local workspace**: Operations go to the embedded local runtime (SQLite + local executor)
- **Remote workspace**: Operations are sent via RPC to the workspace's `server_url`

### Data Isolation

Each workspace's data is isolated:

| Workspace Kind | Data Location |
|---------------|---------------|
| Local | Local SQLite database |
| Remote | Remote server's PostgreSQL database |

**Note**: Data is not synced between workspaces. Each workspace operates independently.

## Development

### Environment Variables

```bash
# Force workspace kind (skip selection)
PUBLIC_DEFAULT_WORKSPACE_KIND=local   # or 'remote'
PUBLIC_SKIP_WORKSPACE_SELECTION=true

# Remote workspace settings
PUBLIC_REMOTE_SERVER_URL=http://localhost:54871
```

### Scripts

```bash
# Show workspace selection (default)
pnpm dev

# Force local workspace
pnpm dev:local

# Force remote workspace
PUBLIC_REMOTE_SERVER_URL=http://localhost:54871 pnpm dev:remote
```

## Security Considerations

### Local Workspaces

| Aspect | Status |
|--------|--------|
| Auth | None (trusted local) |
| Secrets | Native keychain |
| Data | Local SQLite |
| Network | None required |
| Isolation | Docker containers |

### Remote Workspaces

| Aspect | Status |
|--------|--------|
| Auth | JWT + OIDC |
| Secrets | Encrypted transport (TLS) |
| Data | PostgreSQL with auth |
| Network | HTTPS/WSS required |
| Isolation | Docker on worker |

### Secret Handling

| Workspace Kind | Secret Flow |
|---------------|-------------|
| Local | Keychain → Environment → Docker |
| Remote | Keychain → RPC (TLS) → Server (memory) → Worker → Docker |

In remote workspaces, secrets are:
- Read from local keychain on client
- Sent via TLS-encrypted RPC
- Cached in server memory (not persisted)
- Cleared after task completion
