# UI Design

DeliDev provides a desktop and mobile application for orchestrating AI coding agents.

## Table of Contents

1. [Mode Selection](#mode-selection)
2. [Onboarding](#onboarding)
3. [Dashboard](#dashboard)
4. [Chat Interface](#chat-interface)
5. [Task Creation](#task-creation)
6. [Task Detail Pages](#task-detail-pages)
7. [Review Interface](#review-interface)
8. [Settings Interface](#settings-interface)
9. [Repository Management](#repository-management)
10. [Keyboard Shortcuts](#keyboard-shortcuts)
11. [Multi-Tab Interface](#multi-tab-interface)
12. [Desktop Notifications](#desktop-notifications)
13. [Theme / Dark Mode](#theme--dark-mode)

---

## Mode Selection

Mode selection screen shown on first start to choose between Local Mode and Remote Mode.

**Note**: Mobile apps skip this screen and go directly to server URL entry (Remote Mode only).

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        Welcome to DeliDev                                    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Choose how you want to run DeliDev                                        │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ [Monitor Icon]                                                      │   │
│  │                                                                     │   │
│  │ Local Mode                                                          │   │
│  │ Run everything locally on your machine. All processing happens     │   │
│  │ on your computer with no external server required.                  │   │
│  │                                                                     │   │
│  │ • Full privacy - your code never leaves your machine                │   │
│  │ • No network latency                                                │   │
│  │ • Works offline (requires local AI setup)                           │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ [Server Icon]                                                       │   │
│  │                                                                     │   │
│  │ Remote Mode                                                         │   │
│  │ Connect to a remote DeliDev server for task execution and          │   │
│  │ coordination.                                                       │   │
│  │                                                                     │   │
│  │ • Centralized task management                                       │   │
│  │ • Team collaboration support                                        │   │
│  │ • Offload computation to server                                     │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ── Server URL Input (shown when Remote Mode selected) ──                  │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Server URL                              [ https://...           ]   │   │
│  │ Enter the URL of your DeliDev server                                │   │
│  │                                        [Test Connection]            │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│  You can change this setting later in Settings                             │
│                                                          [Continue →]      │
└────────────────────────────────────────────────────────────────────────────┘
```

### Mode Selection Features

| Feature | Description |
|---------|-------------|
| Local Mode | Runs server, worker, and client in single process |
| Remote Mode | Connects to a remote Main Server |
| Connection Test | Validates server URL before proceeding |
| Dev Mode Behavior | In development, mode selection shown on every start |
| Persistence | Mode choice saved for subsequent starts |

---

## Onboarding

First-time setup wizard shown after mode selection.

### Step 1: VCS Provider Connection

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        Welcome to DeliDev                                    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Connect your VCS Provider                                     Step 1 of 2 │
│  ─────────────────────────────────────────                                 │
│                                                                            │
│  Select a provider and enter your access token.                            │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Provider                           [ GitHub               ▼]      │   │
│  ├────────────────────────────────────────────────────────────────────┤   │
│  │ Personal Access Token              [ ghp_...               ]      │   │
│  │                                                                    │   │
│  │ Required scopes: repo, read:user, workflow                        │   │
│  │ [Create token on GitHub →]                                        │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ [✓] Connection successful                                          │   │
│  │ Authenticated as: @username                                        │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                    [Skip]       [Next →]   │
└────────────────────────────────────────────────────────────────────────────┘
```

### Step 2: Add First Repository

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        Welcome to DeliDev                                    │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Add Your First Repository                                     Step 2 of 2 │
│  ─────────────────────────────────────────                                 │
│                                                                            │
│  Enter a repository URL to get started.                                    │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Repository URL                                                     │   │
│  │ [ https://github.com/user/my-app                              ]    │   │
│  │                                        [Validate]                  │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ [✓] Repository found                                               │   │
│  │                                                                    │   │
│  │ Name: my-app                                                       │   │
│  │ Default Branch: main                                               │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                              [← Back]      [Get Started]   │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Dashboard

Main view showing task status in a Kanban-style layout.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              Dashboard                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌────────┐│
│  │ In-Progress │ │  In-Review  │ │   PR-Open   │ │    Done     │ │Rejected││
│  ├─────────────┤ ├─────────────┤ ├─────────────┤ ├─────────────┤ ├────────┤│
│  │             │ │             │ │             │ │             │ │        ││
│  │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │        ││
│  │ │ Task 1  │ │ │ │ Task 3  │ │ │ │ Task 5  │ │ │ │ Task 7  │ │ │        ││
│  │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │        ││
│  │             │ │             │ │             │ │             │ │        ││
│  │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │ ┌─────────┐ │ │        ││
│  │ │ Task 2  │ │ │ │ Task 4  │ │ │ │ Task 6  │ │ │ │ Task 8  │ │ │        ││
│  │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │ └─────────┘ │ │        ││
│  │             │ │             │ │             │ │             │ │        ││
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ └────────┘│
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                            TodoItem List                                     │
│  ┌─────────────────────────────────────────────────────────────────────────┐│
│  │ Issue Triage                                                            ││
│  │ ┌────────────────────────────────────────────────────────────────────┐  ││
│  │ │ [bug] App crashes on startup  │  Suggested: bug, high-priority     │  ││
│  │ └────────────────────────────────────────────────────────────────────┘  ││
│  │                                                                         ││
│  │ PR Review                                                               ││
│  │ ┌────────────────────────────────────────────────────────────────────┐  ││
│  │ │ feat: Add dark mode  │  12 files changed  │  AI: Adds theme toggle │  ││
│  │ └────────────────────────────────────────────────────────────────────┘  ││
│  └─────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Kanban Columns

| Column | Description |
|--------|-------------|
| In-Progress | AI is currently working on the task |
| In-Review | AI work complete, awaiting human review |
| PR-Open | PR created on VCS provider |
| Done | PR merged, task complete |
| Rejected | Task rejected and discarded |

### Task Card

Each task card shows:
- Task title/description
- Repository name
- Current status indicator
- Progress (for CompositeTask: X/Y nodes complete)
- Quick actions

---

## Chat Interface

Accessible via global hotkey (default: `Option+Z` / `Alt+Z`).

```
┌────────────────────────────────────────────┐
│                   Chat                      │
├────────────────────────────────────────────┤
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │ User: Create a new feature to add    │  │
│  │ user authentication                   │  │
│  └──────────────────────────────────────┘  │
│                                            │
│  ┌──────────────────────────────────────┐  │
│  │ Assistant: I'll create a             │  │
│  │ CompositeTask for this. The plan     │  │
│  │ includes:                            │  │
│  │ 1. Database schema for users         │  │
│  │ 2. Auth API endpoints                │  │
│  │ 3. Login/signup UI                   │  │
│  │                                      │  │
│  │ [Update Plan] [Approve] [Reject]     │  │
│  └──────────────────────────────────────┘  │
│                                            │
├────────────────────────────────────────────┤
│  ┌──────────────────────────────────────┐  │
│  │ Type a message...          [mic] [>] │  │
│  └──────────────────────────────────────┘  │
└────────────────────────────────────────────┘
```

### Features

- **Text Input**: Type messages to interact with AI
- **Voice Input**: Microphone for voice commands
- **Local AI Agent Execution**: Runs directly in working directory (no Docker)
- **Full Control**: Create tasks, review, manage repos via chat

---

## Task Creation

```
┌────────────────────────────────────────────────────────────────────────────┐
│                           Create Task                                        │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Repository Group: [ Full Stack App                    ▼]                  │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ 📁 frontend-app  📁 backend-api  📁 shared-libs                   │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Add user authentication to the app                                 │   │
│  │                                                                    │   │
│  │ Focus on @src/auth/login.ts and @src/db/schema.ts                  │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Title & Branch (Optional)                                         │   │
│  │ ┌──────────────────────────────┐ ┌──────────────────────────────┐ │   │
│  │ │ Task Title                   │ │ Branch Name                  │ │   │
│  │ │ [ Add user authentication ] │ │ [ feature/add-user-auth    ] │ │   │
│  │ └──────────────────────────────┘ └──────────────────────────────┘ │   │
│  │ Leave empty for AI-generated suggestions.                         │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Agent: [ Claude Code                                    ▼]                │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ [✓] Composite mode                                                 │   │
│  │     Creates a CompositeTask with AI-generated plan                 │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                [Cancel]    [Create Task]   │
└────────────────────────────────────────────────────────────────────────────┘
```

### File Mention (@)

Type `@` to reference files:

| Feature | Description |
|---------|-------------|
| Autocomplete | Shows file/folder suggestions |
| Fuzzy Search | Matches partial names |
| Multiple Files | Multiple `@` mentions allowed |

### Composite Mode

| State | Task Type | Description |
|-------|-----------|-------------|
| Checked | CompositeTask | AI generates a plan (PLAN.yaml) |
| Unchecked | UnitTask | Direct single-step execution |

---

## Task Detail Pages

### UnitTask Detail

**URL**: `/unit-tasks/{id}`

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          UnitTask Details                                    │
├────────────────────────────────────────────────────────────────────────────┤
│  Task: Add user authentication                                               │
│  Status: [In Review]                    Repository: my-project               │
│  Created: 2024-01-15 10:30              Branch: feature/auth                 │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ AI Agent Request                                              [!]   │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                     │   │
│  │  The AI agent is requesting approval:                               │   │
│  │                                                                     │   │
│  │  "I've completed the authentication implementation.                 │   │
│  │   Should I proceed with creating the PR?"                           │   │
│  │                                                                     │   │
│  │                              [Deny]    [Approve]                    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Agent Session Log                                                         │
│  ─────────────────────────────────────────                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ [10:30:15] Starting agent session...                                │   │
│  │ [10:30:20] Analyzing codebase structure                             │   │
│  │ [10:35:42] Creating auth module                                     │   │
│  │ [10:40:18] Writing tests                                            │   │
│  │ [10:45:30] Requesting user approval...                              │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│  [View Diff]        [Request Changes]        [Reject]                      │
└────────────────────────────────────────────────────────────────────────────┘
```

### CompositeTask Detail

**URL**: `/composite-tasks/{id}`

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        CompositeTask Details                                 │
├────────────────────────────────────────────────────────────────────────────┤
│  Task: Build e-commerce checkout system                                      │
│  Status: [Pending Approval]             Repository: my-shop                  │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │ Plan Approval Required                                        [!]   │   │
│  ├─────────────────────────────────────────────────────────────────────┤   │
│  │                                                                     │   │
│  │  The AI has generated a plan for this task.                         │   │
│  │  Please review and approve to proceed.                              │   │
│  │                                                                     │   │
│  │  [Update Plan]              [Reject]    [Approve Plan]    │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Task Graph                                                                │
│  ─────────────────────────────────────────                                 │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                                                                     │   │
│  │   ┌─────────────┐                                                   │   │
│  │   │ setup-db    │──────┐                                            │   │
│  │   │ [Pending]   │      │                                            │   │
│  │   └─────────────┘      │    ┌──────────────┐    ┌─────────────┐     │   │
│  │                        ├───►│ api-endpoints│───►│  frontend   │     │   │
│  │   ┌─────────────┐      │    │  [Pending]   │    │  [Pending]  │     │   │
│  │   │ setup-auth  │──────┘    └──────────────┘    └─────────────┘     │   │
│  │   │ [Pending]   │                                                   │   │
│  │   └─────────────┘                                                   │   │
│  │                                                                     │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Sub-Tasks                                                                 │
│  ┌────────────────────────────────────────────────────────────────────┐    │
│  │ 1. setup-db       │ Set up database schema      │ [Pending]   [→]  │    │
│  │ 2. setup-auth     │ Set up authentication       │ [Pending]   [→]  │    │
│  │ 3. api-endpoints  │ Implement API endpoints     │ [Pending]   [→]  │    │
│  │ 4. frontend       │ Implement frontend          │ [Pending]   [→]  │    │
│  └────────────────────────────────────────────────────────────────────┘    │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│  Progress: 0/4 tasks complete                                              │
└────────────────────────────────────────────────────────────────────────────┘
```

### Task Graph Visualization

Rendered using `@xyflow/react` in `TaskGraph.tsx`:

| Feature | Description |
|---------|-------------|
| Nodes | Custom nodes with title, prompt preview, and status |
| Edges | Animated arrows showing dependencies |
| Status Colors | Color-coded by task status (see below) |
| Zoom Controls | Zoom in/out, fit view |
| MiniMap | Overview for larger graphs with status-colored nodes |
| Auto Layout | Automatic node positioning based on dependency levels |

**Node Status Colors:**

| Status | Color | Description |
|--------|-------|-------------|
| Pending | Gray | Task not yet started |
| In Progress | Blue | AI is working on the task |
| In Review | Blue | Task awaiting human review |
| Done | Green | Task completed successfully |
| Approved | Green | Task approved |
| PR Open | Green | Pull request created |
| Rejected | Red | Task rejected |

---

## Review Interface

Built-in diff viewer for reviewing AI-generated code.

```
┌────────────────────────────────────────────────────────────────────────────┐
│                          Code Review                                        │
├────────────────────────────────────────────────────────────────────────────┤
│  Task: Add user authentication                                              │
│  Branch: feature/auth                                                       │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Files Changed (5)                    │  src/auth/login.ts                 │
│  ┌─────────────────────────────────┐  │  ────────────────────────────────── │
│  │ [✓] src/auth/login.ts    ✓     │  │   1  + import { hash } from 'bcrypt'│
│  │ [ ] src/auth/signup.ts   (1)   │  │   2  +                              │
│  │ [✓] src/db/schema.ts     ✓     │  │   3  + export async function login( │
│  │ [ ] src/routes/auth.ts         │  │   4  +   email: string,             │
│  │ [ ] tests/auth.test.ts         │  │   5  +   password: string           │
│  └─────────────────────────────────┘  │   6  + ) {                          │
│                                       │   7  +   const user = await findUser│
│  2/5 viewed                          │   ...                                │
│                                       │                                     │
│                                       │  [Mark as viewed] [Open in Editor]  │
├────────────────────────────────────────────────────────────────────────────┤
│  Comments on this file (1):                                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐│
│  │ Line 7: Consider adding rate limiting here              [Edit] [Delete]││
│  └────────────────────────────────────────────────────────────────────────┘│
│  [+ Add comment]                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│  [Submit Review]  [Request Changes]  [Reject]  [Commit]  [Create PR]       │
└────────────────────────────────────────────────────────────────────────────┘
```

### Features

| Feature | Description | Implementation |
|---------|-------------|----------------|
| File Tree | List of changed files with viewed status | `DiffFileList` component |
| Diff Viewer | Side-by-side or unified diff | `DiffViewer` component |
| Inline Comments | Add comments on specific lines | `InlineComment`, `CommentInputForm`, `LineComments` components |
| Viewed Tracking | Mark files as reviewed | `DiffViewer` with `isViewed` prop |

### Agent Log Viewer

The Agent Session Log displays real-time streaming output from AI coding agents. It uses dedicated components to render tool events with rich formatting instead of raw JSON.

| Component | File | Description |
|-----------|------|-------------|
| `AgentLogViewer` | `components/task/AgentLogViewer.tsx` | Main log viewer with auto-scroll and TTY input |
| `ToolUseContent` | `components/task/ToolEventComponents.tsx` | Renders tool use events with tool-specific formatting |
| `ToolResultContent` | `components/task/ToolEventComponents.tsx` | Renders tool results with tool-specific formatting |

#### Supported Tool Formats

| Tool | Display Format |
|------|----------------|
| Read | File icon with path, offset/limit info |
| Write | File icon with path, collapsible content preview |
| Edit | Pencil icon with path, diff-style old/new strings |
| Bash | Terminal icon with command, description |
| Glob | Folder icon with pattern and path |
| Grep | Search icon with pattern, path, glob filter |
| Task | Agent type badge, description, collapsible prompt |
| WebFetch | URL link with prompt |
| WebSearch | Search icon with query |
| TodoWrite | Todo list with status indicators |
| AskUserQuestion | Question with header badge and options |

Unknown tools fall back to JSON display.

### Implementation Components

The review interface is built from the following components:

| Component | File | Description |
|-----------|------|-------------|
| `DiffViewer` | `components/review/DiffViewer.tsx` | Main diff display with inline commenting support |
| `DiffFileList` | `components/review/DiffViewer.tsx` | Sidebar file list with status indicators |
| `InlineComment` | `components/review/InlineComment.tsx` | Single comment display with edit/delete |
| `CommentInputForm` | `components/review/InlineComment.tsx` | Form for adding new comments |
| `LineComments` | `components/review/InlineComment.tsx` | Container for multiple comments on a line |
| `useReviewComments` | `hooks/useReviewComments.ts` | Hook for comment CRUD operations |

### Actions

| Action | Description |
|--------|-------------|
| Submit Review | Open review submission dialog |
| Commit | Merge changes to repository |
| Create PR | Create PR on VCS provider |
| Request Changes | Send feedback for AI rework |
| Reject | Discard the task |

---

## Settings Interface

```
┌────────────────────────────────────────────────────────────────────────────┐
│                            Settings                                         │
├────────────────────────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌──────────────────┐  ┌──────────────────┐               │
│  │   Global    │  │    Workspace     │  │   Connection     │               │
│  └─────────────┘  └──────────────────┘  └──────────────────┘               │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Global Settings (~/.delidev/config.toml)                                  │
│  ─────────────────────────────────────────                                 │
│                                                                            │
│  Appearance                                                                │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Theme          [☀ Light] [🌙 Dark] [🖥 System]                    │   │
│  │ Currently using dark theme (based on system preference)           │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Hotkey                                                                    │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Open Chat                                     [ Option+Z       ]   │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Agent - Planning                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Agent Type                              [ Claude Code       ▼]    │   │
│  │ AI Model                                [ claude-sonnet-4   ▼]    │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Agent - Execution                                                         │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Agent Type                              [ Claude Code       ▼]    │   │
│  │ AI Model                                [ claude-sonnet-4   ▼]    │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
├────────────────────────────────────────────────────────────────────────────┤
│                                                [Cancel]         [Save]     │
└────────────────────────────────────────────────────────────────────────────┘
```

### Tabs

| Tab | Description |
|-----|-------------|
| Global | User-wide settings (`~/.delidev/config.toml`) |
| Workspace | Repository-specific (`.delidev/config.toml`) |
| Connection | Mode selection and server URL |

### Connection Tab (New)

```
┌────────────────────────────────────────────────────────────────────────────┐
│  Connection Settings                                                       │
│  ─────────────────────────────────────────                                 │
│                                                                            │
│  Mode: [●] Local Mode  [ ] Remote Mode                                     │
│                                                                            │
│  ── Remote Mode Settings (shown when Remote selected) ──                   │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ Server URL                              [ https://...           ]   │   │
│  │                                        [Test Connection]            │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│  Note: Changing mode requires restarting the application.                  │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Repository Management

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        Repository Management                                │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Registered Repositories                                                   │
│  ─────────────────────────────────────────                                 │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ 📁 my-project          │ github.com/user/my-project    │ [✕]      │   │
│  ├────────────────────────────────────────────────────────────────────┤   │
│  │ 📁 another-repo        │ github.com/user/another-repo  │ [✕]      │   │
│  ├────────────────────────────────────────────────────────────────────┤   │
│  │ 📁 frontend-app        │ github.com/user/frontend-app  │ [✕]      │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
│                                           [+ Add Repositories]             │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Repository Groups

**URL**: `/repository-groups`

```
┌────────────────────────────────────────────────────────────────────────────┐
│                        Repository Groups                                     │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  Create groups of repositories for multi-repository tasks.                 │
│                                                           [+ Create Group]  │
│                                                                            │
│  ┌────────────────────────────────────────────────────────────────────┐   │
│  │ 🗂 Full Stack App      │ 3 repositories     │ [Edit] [Manage] [✕] │   │
│  │                                                                    │   │
│  │ 📁 frontend-app  📁 backend-api  📁 shared-libs                   │   │
│  └────────────────────────────────────────────────────────────────────┘   │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## Keyboard Shortcuts

All keyboard shortcuts are keyboard layout-aware. When using a non-English keyboard layout (e.g., Korean, Russian), shortcuts are matched by the physical key position using `KeyboardEvent.code` as a fallback. This means pressing the physical 'C' key will trigger the 'c' shortcut regardless of the active input method.

### Global Shortcuts

| Shortcut | Action |
|----------|--------|
| `Option+Z` (macOS) / `Alt+Z` (Win/Linux) | Open Chat |

### Application Shortcuts

| Shortcut | Action |
|----------|--------|
| `?` | Show Keyboard Shortcuts |
| `c` | Create Task |
| `Cmd+N` / `Ctrl+N` | New Task |
| `Cmd+,` / `Ctrl+,` | Settings |
| `Cmd+K` / `Ctrl+K` | Command Palette |
| `Cmd+1` / `Ctrl+1` | Dashboard |
| `Escape` | Close Dialog |

### Form Shortcuts

All multiline inputs (textareas) support `Cmd+Enter` (macOS) / `Ctrl+Enter` (Windows/Linux) to submit. This applies to:

| Form / Input | Location | Submit Action |
|--------------|----------|---------------|
| Task Creation | `/create-task` | Create Task |
| Repository Group Dialog | Repository Groups page | Create/Save Group |
| Onboarding Wizard | Step 1: Next, Step 2: Get Started |
| Chat Input | Chat window | Send Message |
| Inline Comment Edit | Review interface | Save Comment |
| New Comment Form | Review interface | Add Comment |
| Update Plan | Composite task detail | Update Plan |

### Review Interface

| Shortcut | Action |
|----------|--------|
| `J` / `K` | Navigate Files |
| `Enter` | Open File |
| `Cmd+Enter` / `Ctrl+Enter` | Approve |

### Task Detail

| Shortcut | Action |
|----------|--------|
| `A` | Approve |
| `D` | Deny |
| `L` | Toggle Log |
| `S` | Stop Execution |

### Tab Navigation

| Shortcut | Action |
|----------|--------|
| `Cmd+T` / `Ctrl+T` | New Tab |
| `Cmd+W` / `Ctrl+W` | Close Tab |
| `Cmd+Tab` / `Ctrl+Tab` | Next Tab |
| `Cmd+1-9` / `Ctrl+1-9` | Switch Tab |

---

## Multi-Tab Interface

The tab bar appears above the main content area when there are multiple tabs open. It is hidden on mobile devices.

```
┌────────────────────────────────────────────────────────────────────────────┐
│  [Dashboard]  [Task: Add auth ×]  [Task: Fix bug ×]                        │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  (Tab content displayed here)                                              │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

### Implementation

| Component | Location | Description |
|-----------|----------|-------------|
| `TabBar` | `components/layout/TabBar.tsx` | Horizontal tab bar UI with close buttons |
| `useTabNavigation` | `hooks/useTabNavigation.ts` | Syncs router with tab state |
| `useTabTitle` | `hooks/useTabNavigation.ts` | Updates active tab title |

### Features

| Feature | Description |
|---------|-------------|
| Ctrl/Cmd+Click | Open in new tab (via `handleLinkClick` from `useTabNavigation`) |
| Middle Click | Close tab |
| Tab Title | Auto-updated based on route |
| Close Button | Appears on hover for closable tabs |
| Hidden on Single Tab | Tab bar hidden when only one tab exists |
| Desktop Only | Tab bar hidden on mobile viewports |

### State Management

Tab state is managed in `stores/uiStore.ts`:

| Function | Description |
|----------|-------------|
| `addTab` | Creates new tab, returns ID |
| `removeTab` | Closes tab, handles active tab selection |
| `setActiveTab` | Switches to specified tab |
| `updateTabTitle` | Updates tab title |
| `updateTabPath` | Updates tab path |
| `updateTab` | Updates multiple tab properties |

---

## Desktop Notifications

### Triggers

| Event | Notification |
|-------|--------------|
| TTY Input Request | "Agent is asking a question" |
| Task Review Ready | "Task ready for review" |
| Plan Approval | "Plan ready for approval" |
| Task Failure | "Task failed" |

### Click Behavior

1. App window focused
2. Navigate to task detail page

### Platform Support

| Platform | Implementation |
|----------|----------------|
| Windows | tauri-winrt-notification |
| Linux | notify-rust |
| macOS | AppleScript |

---

## Theme / Dark Mode

DeliDev supports light, dark, and system-following theme modes. The theme is toggled via the sidebar button or the Settings > Global > Appearance card.

### Theme Modes

| Mode | Description |
|------|-------------|
| Light | Forces light theme |
| Dark | Forces dark theme |
| System | Follows the operating system's `prefers-color-scheme` setting |

### Implementation

| Component | File | Description |
|-----------|------|-------------|
| `themeStore` | `stores/themeStore.ts` | Zustand store with `ThemeMode` enum, persisted to localStorage |
| `useTheme` | `hooks/useTheme.ts` | Hook that applies `.dark` class to `<html>` and listens for system preference changes |
| Sidebar toggle | `components/layout/Sidebar.tsx` | Quick-access button that cycles Light → Dark → System |
| Settings card | `pages/Settings.tsx` | Appearance card with explicit Light/Dark/System buttons |

### CSS Architecture

- CSS variables defined in `index.css` under `:root` (light) and `.dark` (dark) selectors
- All components use `hsl(var(--variable))` for colors
- Tailwind CSS v4 `dark:` variant configured via `@custom-variant dark (&:where(.dark, .dark *))` for class-based dark mode
- Theme preference persisted in localStorage via Zustand's `persist` middleware (key: `delidev-theme-store`)
