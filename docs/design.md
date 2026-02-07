# DeliDev Design Document

DeliDev is a desktop and mobile application for orchestrating AI coding agents with support for both local and remote execution.

## Table of Contents

1. [Architecture](#architecture)
2. [Technology Stack](#technology-stack)
3. [Entities](#entities)
4. [Configuration](#configuration)
5. [Authentication](#authentication)
6. [Secrets Management](#secrets-management)
7. [Workflows](#workflows)
8. [Error Handling](#error-handling)
9. [Related Documents](#related-documents)

---

## Architecture

### Component Roles

DeliDev consists of three main components:

| Component | Role |
|-----------|------|
| **Main Server** | Maintains the task list, provides RPC server, coordinates workers |
| **Worker Server** | Runs AI coding agents (Claude Code, OpenCode, etc.) in Docker sandboxes |
| **Client** | Desktop/Mobile app based on Tauri, provides user interface |

### Distributed Architecture

```
                                ┌─────────────────────────────────┐
                                │         Main Server             │
                                │  (Task Management, RPC Server)  │
                                │                                 │
                                │  ┌─────────────────────────────┐│
                                │  │      PostgreSQL / SQLite    ││
                                │  │      (multi/single mode)    ││
                                │  └─────────────────────────────┘│
                                │                                 │
                                │  JWT Auth (OpenID Connect)      │
                                └─────────────┬───────────────────┘
                                              │
                         Connect RPC over HTTP
                                              │
                ┌─────────────────────────────┼─────────────────────────────┐
                │                             │                             │
                ▼                             ▼                             ▼
    ┌───────────────────┐       ┌───────────────────┐       ┌───────────────────┐
    │   Worker Server   │       │   Worker Server   │       │      Client       │
    │                   │       │                   │       │  (Desktop/Mobile) │
    │  ┌─────────────┐  │       │  ┌─────────────┐  │       │                   │
    │  │Claude Code  │  │       │  │Claude Code  │  │       │  React + Tauri    │
    │  │OpenCode     │  │       │  │OpenCode     │  │       │  react-query      │
    │  │Aider, etc.  │  │       │  │Aider, etc.  │  │       │                   │
    │  └─────────────┘  │       │  └─────────────┘  │       │  Keychain Access  │
    │                   │       │                   │       │  (macOS, etc.)    │
    │  Docker Sandbox   │       │  Docker Sandbox   │       │                   │
    └───────────────────┘       └───────────────────┘       └───────────────────┘
```

**Key Characteristics:**

- **Main Server**: Central hub for task management and worker coordination
- **Worker Servers**: Execute AI agents in isolated Docker containers
- **Client**: Never communicates directly with Worker Server (always through Main Server)

### Single Process Mode

For desktop usage, all components can run in a single process for a seamless local experience:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Single Process Desktop App                         │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────────────┐  ┌─────────────────────┐  ┌─────────────────────┐  │
│  │   Embedded Server   │  │   Embedded Worker   │  │      Client UI      │  │
│  │                     │  │                     │  │   (Tauri WebView)   │  │
│  │                     │  │                     │  │                     │  │
│  │  In-process calls   │◄─┤  In-process calls   │◄─┤   In-process calls  │  │
│  │  (no network)       │  │  (no network)       │  │   (no network)      │  │
│  └─────────────────────┘  └─────────────────────┘  └─────────────────────┘  │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │                          SQLite Database                                 ││
│  └─────────────────────────────────────────────────────────────────────────┘│
│                                                                             │
│  Auth: DISABLED (single user, trusted local execution)                      │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**Key Features:**

- No network overhead - all RPC calls are direct function invocations
- SQLite storage for local data
- Authentication disabled (trusted local execution)
- Secrets read directly from local keychain

### Communication

| Layer | Protocol |
|-------|----------|
| Client ↔ Main Server | Connect RPC over HTTP |
| Frontend State | react-query for data fetching |
| Client ↔ Worker | **Not allowed** - all communication goes through Main Server |

---

## Technology Stack

### Desktop/Mobile Framework

- **Tauri**: Rust-based framework using system WebView
  - Small binary size (~10MB)
  - Low memory footprint
  - Native system integration
  - Cross-platform: Desktop (Windows, macOS, Linux) and Mobile (iOS, Android)

### Backend (Rust)

| Crate | Purpose |
|-------|---------|
| sqlx | Async SQLite/PostgreSQL driver |
| bollard | Docker API client |
| git2 | Git operations |
| reqwest | HTTP client for VCS/RPC APIs |
| serde | Serialization |
| tokio | Async runtime |
| axum | Web server framework |
| jsonwebtoken | JWT authentication |

### Shared Crates

| Crate | Purpose |
|-------|---------|
| `entities` | Core entity definitions and shared input sanitization utilities |
| `coding_agents` | AI agent abstraction, output normalization, Docker sandboxing, task execution |
| `task_store` | Task storage (SQLite, PostgreSQL, in-memory) |
| `rpc_protocol` | Connect RPC protocol definitions (Protobuf) |
| `git_ops` | Git operations, worktree management & repository caching |
| `auth` | JWT authentication & RBAC |
| `secrets` | Cross-platform keychain access |
| `worker_impl` | Local worker implementation with incremental log persistence for single-process mode task execution |

### Frontend (React + TypeScript)

| Package | Purpose |
|---------|---------|
| react | UI framework |
| typescript | Type safety |
| @rspack/core | Build tool (Rust-based) |
| tailwindcss | Utility-first CSS |
| shadcn/ui | Component library |
| zustand | State management |
| @tanstack/react-query | Server state management |

### Coding Agent Normalization

The `coding_agents` crate normalizes output from all AI coding agents:

```
┌─────────────────────────────────────────────────────────────────┐
│                        coding_agents crate                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │Claude Code  │  │  OpenCode   │  │   Aider     │  ...        │
│  │   Parser    │  │   Parser    │  │   Parser    │             │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘             │
│         │                │                │                     │
│         └────────────────┴────────────────┘                     │
│                          │                                      │
│                          ▼                                      │
│         ┌────────────────────────────────────┐                  │
│         │      Normalized Event Types        │                  │
│         │  - ToolUse, ToolResult             │                  │
│         │  - TextOutput, ErrorOutput         │                  │
│         │  - FileChange, CommandExecution    │                  │
│         │  - AskUserQuestion, etc.           │                  │
│         └────────────────────────────────────┘                  │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Benefits:**
- Frontend uses only normalized types
- Easy to add new AI agents by implementing a parser
- Consistent UI rendering regardless of agent type

### Task Execution

The `coding_agents` crate also provides platform-agnostic task execution:

```
┌─────────────────────────────────────────────────────────────────┐
│                    coding_agents::executor                       │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                     TaskExecutor                         │   │
│  │  - Creates git worktrees via RepositoryCache            │   │
│  │  - Runs AI agents with proper configuration             │   │
│  │  - Streams events via EventEmitter trait                │   │
│  │  - Handles TTY input via TtyInputRequestManager         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
│  ┌─────────────────────┐  ┌─────────────────────┐              │
│  │   EventEmitter      │  │ TtyInputRequest     │              │
│  │   (trait)           │  │ Manager             │              │
│  │                     │  │                     │              │
│  │ Platform-agnostic   │  │ Pending request     │              │
│  │ event emission      │  │ tracking & response │              │
│  └─────────────────────┘  └─────────────────────┘              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

**Key Components:**
- `TaskExecutor<E>`: Generic executor parameterized by event emitter
- `EventEmitter` trait: Platform-specific event emission (Tauri, CLI, etc.)
- `TtyInputRequestManager`: Manages pending TTY input requests
- `TaskExecutionConfig`: Configuration for executing a task
- `run_agent_in_worktree()`: Runs an agent in an existing worktree (used for subtasks)

This design allows the same execution logic to be reused across different platforms (desktop app, CLI, server) by implementing the `EventEmitter` trait.

---

## Entities

### VCSType

```
enum VCSType {
  git           // Git
}
```

### VCSProviderType

```
enum VCSProviderType {
  github        // GitHub
  gitlab        // GitLab
  bitbucket     // Bitbucket
}
```

### AIAgentType

```
enum AIAgentType {
  claude_code    // Claude Code - Anthropic's terminal-based agentic coding tool
  open_code      // OpenCode - Open-source Claude Code alternative
  gemini_cli     // Gemini CLI - Google's open-source AI agent
  codex_cli      // Codex CLI - OpenAI's terminal-based coding assistant
  aider          // Aider - Open-source CLI for multi-file changes
  amp            // Amp - Sourcegraph's agentic coding CLI
}
```

### TokenUsage

Token usage statistics from an AI coding agent session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| inputTokens | number | Y | Number of input tokens (excluding cache) |
| outputTokens | number | Y | Number of output tokens generated |
| cacheReadInputTokens | number | Y | Number of tokens read from cache |
| cacheCreationInputTokens | number | Y | Number of tokens written to cache |
| totalCostUsd | number | Y | Total cost in USD for this session |
| durationMs | number | Y | Duration of the session in milliseconds |
| numTurns | number | Y | Number of conversation turns (API round-trips) |

### AgentSession

A single AI coding agent session.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| aiAgentType | AIAgentType | Y | Agent type |
| aiAgentModel | string | N | Model to use |
| tokenUsage | TokenUsage | N | Token usage statistics from the session |

### AgentTask

A collection of AgentSessions. The retryable unit.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| baseRemotes | BaseRemote[] | Y | Git repository information |
| agentSessions | AgentSession[] | Y | Session list |
| aiAgentType | AIAgentType | N | Agent type |
| aiAgentModel | string | N | Model to use |

### UnitTask

A single task unit visible to users.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| repositoryGroupId | string | Y | Associated RepositoryGroup ID |
| agentTask | AgentTask | Y | Associated AgentTask (1:1) |
| branchName | string | N | Custom branch name |
| linkedPrUrl | string | N | Created PR URL |
| baseCommit | string | N | Base commit hash |
| endCommit | string | N | End commit hash |
| gitPatch | string | N | Git patch (unified diff) of changes made by AI agent |
| autoFixTasks | AgentTask[] | Y | Auto-fix attempts |
| status | UnitTaskStatus | Y | Current status |

#### UnitTaskStatus

```
enum UnitTaskStatus {
  in_progress   // AI is working
  in_review     // AI work complete, awaiting human review
  approved      // Human approved
  pr_open       // PR created
  done          // PR merged
  rejected      // Rejected and discarded
  failed        // Task failed with error
  cancelled     // Task was cancelled by user
}
```

### CompositeTask

Task graph-based Agent Orchestrator.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| repositoryGroupId | string | Y | Associated RepositoryGroup ID |
| planningTask | AgentTask | Y | AgentTask for generating PLAN.yaml |
| planYaml | string | N | Raw PLAN.yaml content (persisted after planning) |
| updatePlanFeedback | string | N | User feedback for re-planning (set by Update Plan, cleared after re-planning) |
| tasks | CompositeTaskNode[] | Y | List of task nodes |
| status | CompositeTaskStatus | Y | Current status |
| executionAgentType | AIAgentType | N | Agent type for UnitTasks |

#### CompositeTaskStatus

```
enum CompositeTaskStatus {
  planning           // Generating PLAN.yaml
  pending_approval   // Waiting for user approval
  in_progress        // Tasks are executing
  done               // All tasks completed
  rejected           // User rejected the plan
  failed             // Planning or execution failed
}
```

### CompositeTaskNode

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| unitTask | UnitTask | Y | Associated UnitTask |
| dependsOn | CompositeTaskNode[] | Y | Dependent nodes |

### TodoItem

Tasks that humans should do but AI can assist with.

#### type: "issue_triage"

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| issueUrl | string | Y | Issue URL |
| repositoryId | string | Y | Repository ID |
| issueTitle | string | Y | Issue title |
| suggestedLabels | string[] | N | AI suggested labels |
| suggestedAssignees | string[] | N | AI suggested assignees |

#### type: "pr_review"

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| prUrl | string | Y | PR/MR URL |
| repositoryId | string | Y | Repository ID |
| prTitle | string | Y | PR title |
| changedFilesCount | number | Y | Changed files count |
| aiSummary | string | N | AI analysis summary |

### Repository

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| vcsType | VCSType | Y | Version control system |
| vcsProviderType | VCSProviderType | Y | VCS provider |
| remoteUrl | string | Y | Remote URL |
| name | string | Y | Repository name |
| defaultBranch | string | Y | Default branch |

### Workspace

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| name | string | Y | Workspace name |
| description | string | N | Description |
| createdAt | timestamp | Y | Creation time |
| updatedAt | timestamp | Y | Last update time |

### RepositoryGroup

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| name | string | N | Group name |
| workspaceId | string | Y | Parent workspace ID |
| repositoryIds | string[] | Y | Repository IDs |
| createdAt | timestamp | Y | Creation time |
| updatedAt | timestamp | Y | Last update time |

### TtyInputRequest

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string | Y | Unique identifier |
| taskId | string | Y | Associated UnitTask ID |
| sessionId | string | Y | Agent session ID |
| prompt | string | Y | Question from agent |
| inputType | TtyInputType | Y | Input type |
| options | string[] | N | Available options |
| createdAt | timestamp | Y | Creation time |
| status | TtyInputStatus | Y | Current status |
| response | string | N | User's response |
| respondedAt | timestamp | N | Response time |

---

## Configuration

### Global Settings

Location: `~/.delidev/config.toml`

```toml
[learning]
autoLearnFromReviews = false

[hotkey]
openChat = "Option+Z"

[notification]
enabled = true
approvalRequest = true
userQuestion = true
reviewReady = true

[agent.planning]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[agent.execution]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[agent.chat]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[container]
runtime = "docker"
use_container = true

[composite_task]
auto_approve = false

[concurrency]
# max_concurrent_sessions = 3
```

### Repository Settings

Location: `.delidev/config.toml` (committed to git)

```toml
[branch]
template = "feature/${taskId}-${slug}"

[automation]
autoFixReviewComments = true
autoFixReviewCommentsFilter = "write_access_only"
autoFixCIFailures = true
maxAutoFixAttempts = 3

[learning]
autoLearnFromReviews = false

[composite_task]
auto_approve = true
```

### Configuration Precedence

1. Repository settings take precedence
2. Global settings as fallback
3. Built-in defaults if neither is set

---

## Authentication

### Server Authentication (Remote Mode)

In remote mode, the server requires authentication for all API requests.

#### JWT Authentication

- Tokens issued after successful OIDC authentication
- Includes user ID, email, and name claims
- Default expiration: 24 hours

| Variable | Description | Default |
|----------|-------------|---------|
| `DELIDEV_JWT_SECRET` | Secret key for signing JWTs | (required) |
| `DELIDEV_JWT_EXPIRATION_HOURS` | Token expiration | 24 |
| `DELIDEV_JWT_ISSUER` | JWT issuer claim | "delidev" |

#### OpenID Connect (OIDC)

Supports authentication with any OIDC provider (Google, GitHub, Keycloak, etc.).

| Variable | Description |
|----------|-------------|
| `DELIDEV_OIDC_ISSUER_URL` | OIDC provider issuer URL |
| `DELIDEV_OIDC_CLIENT_ID` | OAuth2 client ID |
| `DELIDEV_OIDC_CLIENT_SECRET` | OAuth2 client secret |
| `DELIDEV_OIDC_REDIRECT_URL` | Redirect URL after authentication |
| `DELIDEV_OIDC_SCOPES` | Comma-separated scopes |

**Security Features:**
- PKCE (S256 challenge method)
- CSRF protection via state parameter
- Database-backed state storage
- Redirect URI validation

#### Single Process Mode

**Authentication is DISABLED** in single process mode. This mode is intended for local desktop usage where all requests are trusted.

### VCS Provider Authentication

Stored in `~/.delidev/credentials.toml`:

```toml
[github]
token = "ghp_xxxxxxxxxxxx"

[gitlab]
token = "glpat-xxxxxxxxxxxx"

[bitbucket]
username = "your-username"
app_password = "xxxxxxxxxxxx"
```

---

## Secrets Management

Secrets are stored in the native system keychain and transported to workers when needed.

### Known Secret Keys

| Key | Description | Used By |
|-----|-------------|---------|
| `CLAUDE_CODE_OAUTH_TOKEN` | Claude Code OAuth token | Claude Code |
| `ANTHROPIC_API_KEY` | Anthropic API key | Claude Code, Amp |
| `OPENAI_API_KEY` | OpenAI API key | OpenCode, Aider, Codex CLI |
| `GOOGLE_AI_API_KEY` | Google AI API key | Gemini CLI |
| `GITHUB_TOKEN` | GitHub access token | All agents |

### Secret Storage

| Platform | Backend |
|----------|---------|
| macOS | Keychain Services |
| Windows | Windows Credential Manager |
| Linux | Secret Service (libsecret/KWallet) |

### Client-to-Worker Transport

```
┌─────────────┐    1. Client reads    ┌─────────────┐
│   Client    │    secrets from       │   Native    │
│   (Tauri)   │◄───────────────────── │  Keychain   │
└──────┬──────┘    local keychain     └─────────────┘
       │
       │ 2. Client sends secrets
       │    via sendSecrets RPC
       ▼
┌─────────────┐
│   Server    │    3. Server stores
│   (Main)    │    secrets temporarily
└──────┬──────┘    (in-memory, per-task)
       │
       │ 4. Server relays secrets
       │    when task starts
       ▼
┌─────────────┐
│   Worker    │    5. Worker injects
│             │    secrets as env vars
└─────────────┘
```

**Security:**
- Secrets encrypted at rest (native keychain)
- TLS for transport (HTTPS/WSS)
- Temporary in-memory storage on server
- Cleared after task completion
- Single-process mode reads directly from local keychain

---

## Workflows

### UnitTask Execution Flow

```
User creates UnitTask (with git_remote_url)
        ▼
Check if repository is cached
        ▼
┌──────────────────────────────────────┐
│ Not cached        │ Cached           │
│ Clone as bare     │ Fetch updates    │
│ repo to cache     │ from remote      │
└────────┬──────────┴────────┬─────────┘
         └────────┬──────────┘
                  ▼
Create git worktree from cached repo
        ▼
Start Docker container (on Worker)
        ▼
Run AI Agent (in_progress)
        ▼
Generate git patch from worktree
        ▼
Store patch in database (git_patch field)
        ▼
AI work done (in_review)
        ▼
Human review ──┬──► Approve (approved)
               ├──► Request changes (subtask → in_review)
               └──► Reject (rejected)

Approved ──────┬──► Create PR (subtask → pr_open)
               ├──► Commit to local repo (subtask → done)
               └──► Dismiss approval (back to in_review)

Note: In local mode, "Request Changes" creates a subtask (like "Create PR"
and "Commit to Local") that applies the reviewer's feedback using the AI
agent. Inline review comments from the diff viewer and an optional extra
comment are combined into the subtask prompt. The Approve button is disabled
when the user has written inline review comments, prompting them to submit
the feedback via "Request Changes" first.

Note: While in `in_progress`, the user can cancel the task at any time,
which transitions the status to `cancelled`. The agent execution is
aborted and any partial work is preserved in the worktree.

On each status transition, `task-status-changed` and `task-completed`
events are emitted so the frontend updates automatically.

### Subtasks

Subtasks are agent sessions that run within an existing unit task's
worktree. They are used for post-approval operations like PR creation,
committing to local, and applying requested changes. Key characteristics:

- A subtask creates a new `AgentSession` under the same `AgentTask`
- It runs in the existing worktree (no new worktree is created)
- The parent task transitions to `in_progress` while the subtask runs
- On success, the task transitions to the target status (e.g. `pr_open`)
- On failure or cancellation, the task reverts to `approved`
- Subtasks are not shown separately in the dashboard - they use the
  same task ID as the parent
- The subtask can be cancelled using the same "Stop Agent" button
- Each session (main execution + subtasks) has its own collapsible
  "Agent Session Log" section in the task detail view. The backend
  returns all sessions grouped via `SessionLogsGroup` in the
  `get_task_logs` response.

```
Approved ──► execute_subtask(prompt, target_status)
                    │
                    ├── Create new AgentSession
                    ├── Transition to in_progress
                    ├── Run agent in existing worktree
                    │
                    ├── Success → target_status (pr_open / done / in_review)
                    ├── Failure → approved (user can retry)
                    └── Cancelled → approved
```

### Change Persistence

Changes made by AI agents are persisted as git patches in the database:
- On task completion, a unified diff is generated from the worktree
- The patch is stored in the `git_patch` field of `UnitTask`
- This allows changes to be persisted without needing write access
  to the repository (the worker server may not have push permission)
- In local mode, the worktree is preserved while the task is in review
  so the user can inspect changes directly on disk
- Worktrees for failed/cancelled tasks are cleaned up immediately
- The frontend parses `git_patch` using `parseUnifiedDiff()` and displays
  it via the `DiffViewer` component when the user clicks "View Diff"
```

### Repository Caching

For better performance, DeliDev caches repositories locally:

1. **Cache Location**: `~/.delidev/repo-cache/<url-hash>/` where `<url-hash>` is a SHA256 hash of the normalized repository URL (first 32 hex characters)
2. **Storage Format**: Bare git repositories (no working directory)
3. **Worktree Location**: `~/.delidev/worktrees/<task-id>-<branch>/`

This approach:
- Avoids repeated full clones for the same repository
- Reduces disk space by using worktrees instead of full clones
- Enables faster task startup by only fetching changes
- Supports multiple concurrent tasks on the same repository

### CompositeTask Execution Flow

```
User creates CompositeTask
        ▼
System creates planningTask (AgentTask) and session
        ▼
Planning agent starts immediately (status: planning)
        ▼
┌────────────────────────────────────────────────────┐
│ Executor generates PLAN-{suffix}.yaml filename      │
│ Planning agent creates the specified file            │
│ - Real-time logs streamed to UI via AgentLogViewer │
│ - Logs persisted incrementally to database         │
└────────────────────────────────────────────────────┘
        ▼
┌────────────────────────────────────────────────────┐
│ On Success:                                        │
│ 1. PLAN.yaml content read from worktree            │
│ 2. Content persisted to database (plan_yaml field) │
│ 3. Planning worktree cleaned up immediately        │
│ 4. Status → pending_approval                       │
├────────────────────────────────────────────────────┤
│ On Failure:                                        │
│ 1. Planning worktree cleaned up                    │
│ 2. Status → failed                                 │
└────────────────────────────────────────────────────┘
        ▼
┌──────────────────────────────────────┐
│ Success         │ Failure            │
│ pending_approval│ failed             │
└────────┬────────┴────────┬───────────┘
         ▼                 │
User reviews plan          │ (User can retry or discard)
  ├─ Approve ──────────────┤
  │   1. Validate plan     │
  │      (cycles, deps,    │
  │       limits ≤100)     │
  │   2. → in_progress     │
  ├─ Update Plan ──────────┼──► Re-plan with feedback
  │   (appends feedback    │    (status → planning)
  │    to prompt, resets   │
  │    to planning status) │
  └─ Reject → rejected     │
         ▼                 │
Status: in_progress        │
         ▼                 │
Executor parses plan_yaml  │
Validates plan again       │
Creates CompositeTaskNode  │
+ UnitTask records         │
(with cleanup on error)    │
         ▼                 │
Start root tasks           │
(no dependencies)          │
         ▼                 │
Monitor task graph         │
(configurable interval):   │
  - Task completes →       │
    start ready dependents │
  - Task fails →           │
    leave dependents       │
         ▼                 │
All tasks done             │
(status: done or failed)   │
```

The planning agent execution is handled by `LocalExecutor::execute_composite_task()`, which:
1. Creates an agent session for the planning task
2. Generates the plan YAML filename (e.g., `PLAN-a1b2c3.yaml`) before building the prompt
3. Spawns a background task for execution
4. Uses `PersistingEventEmitter` for real-time streaming and incremental log persistence
5. On success, reads the plan YAML file (by its known filename) from the worktree and persists to `plan_yaml` field
6. Cleans up the planning worktree immediately after persisting (not kept until task completion)
7. Updates composite task status to `pending_approval` on success or `failed` on error
8. Emits `task-status-changed` and `task-completed` events so the frontend updates automatically



The graph execution after approval is handled by `LocalExecutor::execute_composite_task_graph()`, which:
1. Parses `plan_yaml` into task definitions
2. Validates the plan (cycles, invalid dependencies, duplicate IDs, empty prompts)
3. Enforces resource limits (maximum 100 tasks per plan)
4. Creates `AgentTask`, `UnitTask`, and `CompositeTaskNode` records for each plan task (with automatic cleanup of orphaned records on failure)
5. Sets dependency relationships between nodes
6. Starts executing root tasks (tasks with no dependencies) immediately
7. Spawns a monitoring task that periodically checks for newly ready tasks as dependencies complete (configurable interval, default 3 seconds)
8. When all tasks reach a terminal state, marks the composite task as `done` (if all succeeded) or `failed` (if any failed)

**Note:** The server's `approve_task` endpoint validates the plan and changes status, but does *not* create `CompositeTaskNode` records. Node creation is delegated entirely to the executor to avoid duplicate record creation between server and desktop (Tauri) code paths.


**Update Plan:**

When a composite task is in `pending_approval` or `failed` state, the user can request plan updates via the "Update Plan" button. This:
1. Stores the user's feedback in the `update_plan_feedback` field (the original `prompt` is **not** modified)
2. Keeps the existing `plan_yaml` (so the planning agent can reference it)
3. Creates a new planning `AgentTask`
4. Resets status to `planning`
5. Re-triggers `LocalExecutor::execute_composite_task()` which detects `update_plan_feedback` and uses the existing plan + feedback (instead of the original prompt) to generate a revised plan (falls back to initial planning prompt if either is empty)
6. After re-planning completes (or fails), `update_plan_feedback` is cleared

The `update_plan_with_prompt` Tauri command handles this flow.



**PLAN.yaml Persistence:**
- The raw PLAN.yaml content is stored in the `plan_yaml` field of `CompositeTask`
- This allows the plan to be accessed without the worktree (which is cleaned up immediately)
- The worktree is cleaned up right after the PLAN.yaml is persisted to conserve disk space

### PR Auto-Management

#### Auto-Fix Review Comments

When enabled:
1. PR receives review comment
2. Check author against filter
3. Create AgentTask to address feedback
4. AI applies fix and pushes
5. Repeat up to `maxAutoFixAttempts`

#### Auto-Fix CI Failures

When enabled:
1. CI fails on PR
2. Create AgentTask to fix
3. AI analyzes logs, fixes, pushes
4. Repeat up to `maxAutoFixAttempts`

### TTY Input Proxy

When an AI agent requires user input:

1. Agent outputs TTY input request
2. TTY Proxy Service intercepts and pauses execution
3. Desktop notification sent
4. User responds via web form
5. Response written to agent stdin
6. Execution resumes

### AI Document Learning

When user requests changes on a task:

1. Feedback appended to task prompt
2. AI agent re-runs with learning instructions
3. Agent considers if feedback is generalizable
4. If yes, updates AGENTS.md or CLAUDE.md

### Event-Driven UI Updates

The frontend uses a purely event-driven architecture for real-time updates. No polling is used.

**Task Log Streaming:**
```
Tauri Backend ─── agent-output event ──► useTaskLogs hook
                                              └── Appends to event list in real-time
                                         (Initial fetch on mount for historical logs)
```

**Task Status Updates:**
```
Tauri Backend ─── task-status-changed ──► useTaskStatusEvents hook
              ─── task-completed ────────►    └── Invalidates react-query caches
                                               (task detail, task list, and
                                                composite task nodes queries)
```

**Plan Preview (PendingApproval):**

During `pending_approval`, `CompositeTaskNode` records don't exist in the database yet (they are created on plan approval). The frontend parses the raw `plan_yaml` field using `parsePlanYamlToNodes()` to generate preview nodes for the TaskGraph component and sub-task list. Once the plan is approved and execution begins, the preview is replaced by real database-backed nodes.

### Notification Flow

The notification system provides persistent, actionable notifications for system events:

```
Tauri Backend Events ──► useNotificationEvents hook
                              ├── notificationCenterStore (persistent, localStorage)
                              └── Desktop notification (if window unfocused)

User Interface:
  Sidebar bell icon (unread badge) ──► NotificationPanel (slide-out)
  /notifications route ──────────────► Notifications page (full list)
  'n' keyboard shortcut ─────────────► Toggle NotificationPanel
```

Notification categories: `task_review_ready`, `plan_approval`, `task_failed`, `tty_input_request`, `task_completed`.

Each notification links to the relevant task page. See [Notification System](./notifications.md) for full details.

---

## Error Handling

### Docker Errors

| Error | Resolution |
|-------|------------|
| Docker daemon not running | Start Docker Desktop |
| Image pull failed | Check network, verify image name |
| Container start failed | Check resources, verify ports |

### VCS Provider Errors

| Error | Resolution |
|-------|------------|
| Authentication failed | Update token in credentials.toml |
| Permission denied | Verify token scopes |
| Rate limit exceeded | Wait for reset |
| PR creation failed | Check branch protection rules |

### Network Errors

| Error | Resolution |
|-------|------------|
| Connection timeout | Check internet connection |
| SSL certificate error | Update system certificates |

---

## Security

### Input Validation

All user inputs are validated to prevent security vulnerabilities:

#### Repository URL Validation

| Check | Description |
|-------|-------------|
| Allowed schemes | Only `https://`, `http://`, `git@`, `ssh://` are allowed |
| Dangerous characters | Blocks `$`, `` ` ``, `|`, `;`, `&`, newlines |
| Shell injection | Blocks `$(`, `${` patterns |
| Length limit | Maximum 2048 characters |

#### Branch Name Validation

| Check | Description |
|-------|-------------|
| Path traversal | Blocks `..` sequences |
| Special characters | Blocks `$`, `` ` ``, `|`, `;`, `&`, `~`, `^`, `:`, etc. |
| Git patterns | Blocks `.lock` suffix, leading/trailing dots, `@{` |
| Length limit | Maximum 256 characters |

#### Prompt Validation

| Check | Description |
|-------|-------------|
| Minimum length | At least 1 character |
| Maximum length | 100,000 characters |
| Null bytes | Not allowed |

#### Feedback Validation (Update Plan)

| Check | Description |
|-------|-------------|
| Control characters | Removed (except `\n` and `\t`) via `entities::sanitize_user_input()` |
| Maximum length | 10,000 characters (`entities::MAX_FEEDBACK_LENGTH`) |

#### Title Validation

| Check | Description |
|-------|-------------|
| Maximum length | 500 characters |
| Null bytes | Not allowed |

### Rate Limiting

Task creation is rate-limited to prevent resource exhaustion:

| Limit | Value |
|-------|-------|
| Minimum interval | 500 milliseconds between task creations |
| Max tasks per plan | 100 tasks per composite task plan |

### Path Sanitization

Task IDs and branch names are sanitized when used in file system paths:

| Input | Sanitization |
|-------|--------------|
| Task ID | Only alphanumeric and hyphens allowed |
| Branch name | Slashes and special characters converted to hyphens |

---

## Chat Interface

The chat interface provides a global communication channel with AI agents.

### Components

| Component | Purpose |
|-----------|---------|
| ChatWindow | Modal overlay containing the chat UI |
| MessageList | Scrollable list of chat messages |
| ChatInput | Text input with send button |

### State Management

Chat state is managed via Zustand in `chatStore.ts`:

| State | Type | Description |
|-------|------|-------------|
| isOpen | boolean | Chat window visibility |
| messages | ChatMessage[] | Message history |
| inputValue | string | Current input text |
| isLoading | boolean | AI response pending |

### Message Structure

```typescript
interface ChatMessage {
  id: string;          // UUID (crypto.randomUUID)
  role: MessageRole;   // User or Assistant
  content: string;     // Message text
  timestamp: Date;     // Creation time
}
```

### Global Hotkey

- **macOS**: `Option + Z`
- **Windows/Linux**: `Alt + Z`

Toggles chat window visibility from anywhere in the app.

---

## Related Documents

- [Main Server](./main-server.md) - Main Server details
- [Worker Server](./worker-server.md) - Worker Server details
- [Tauri App](./tauri-app.md) - Desktop/Mobile app details
- [Local vs Remote Mode](./client-local-vs-remote.md) - Mode comparison
- [UI Design](./ui.md) - User interface specifications
- [Notification System](./notifications.md) - Notification center, desktop notifications, and event handling
- [PLAN.yaml Specification](./plan-yaml.md) - Task plan format
