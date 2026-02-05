# DeliDev User Guide

Welcome to DeliDev! This guide will help you get started with using DeliDev to orchestrate AI coding agents for your software development workflow.

## Table of Contents

1. [Introduction](#introduction)
2. [Getting Started](#getting-started)
3. [Core Concepts](#core-concepts)
4. [Using the Desktop App](#using-the-desktop-app)
5. [Creating Tasks](#creating-tasks)
6. [Reviewing AI Work](#reviewing-ai-work)
7. [Managing Repositories](#managing-repositories)
8. [Configuration](#configuration)
9. [Keyboard Shortcuts](#keyboard-shortcuts)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

DeliDev is a desktop and mobile application for orchestrating AI coding agents. It allows you to:

- Create coding tasks and have AI agents complete them
- Review and approve AI-generated code changes
- Manage multiple repositories and workspaces
- Automate PR reviews and CI fixes
- Work in local mode (single user) or remote mode (team)

### Supported AI Agents

DeliDev supports multiple AI coding agents:

| Agent | Description |
|-------|-------------|
| **Claude Code** | Anthropic's terminal-based agentic coding tool |
| **OpenCode** | Open-source Claude Code alternative |
| **Gemini CLI** | Google's AI agent |
| **Codex CLI** | OpenAI's coding assistant |
| **Aider** | Open-source CLI for multi-file changes |
| **Amp** | Sourcegraph's agentic coding CLI |

---

## Getting Started

### Installation

1. **Download** the DeliDev installer for your platform:
   - macOS: `.dmg` file
   - Windows: `.exe` installer
   - Linux: `.AppImage` or `.deb` package

2. **Install** by following the platform-specific instructions.

3. **Launch** DeliDev from your applications menu.

### First Run

On first launch, you'll be guided through:

1. **Mode Selection**: Choose between Local Mode or Remote Mode
2. **VCS Connection**: Connect your GitHub, GitLab, or Bitbucket account
3. **Repository Setup**: Add your first repository

### Setting Up API Keys

DeliDev stores API keys securely in your system keychain. To configure:

1. Open **Settings** (Cmd/Ctrl + ,)
2. Go to the **Secrets** tab
3. Add your API keys:
   - `ANTHROPIC_API_KEY` for Claude Code
   - `OPENAI_API_KEY` for OpenCode, Aider, Codex CLI
   - `GOOGLE_AI_API_KEY` for Gemini CLI
   - `GITHUB_TOKEN` for GitHub operations

---

## Core Concepts

### Workspaces

A **Workspace** is a container for organizing your repositories and tasks. You can have multiple workspaces for different projects or clients.

### Repository Groups

A **Repository Group** is a collection of one or more repositories that work together. Tasks are created for repository groups, allowing multi-repo operations.

### Tasks

DeliDev has two types of tasks:

#### UnitTask

A **UnitTask** is a single coding task that an AI agent completes. It goes through these statuses:

| Status | Description |
|--------|-------------|
| `in_progress` | AI is actively working on the task |
| `in_review` | AI work is complete, awaiting your review |
| `approved` | You approved the changes |
| `pr_open` | A PR has been created |
| `done` | PR was merged |
| `rejected` | Task was rejected and discarded |

#### CompositeTask

A **CompositeTask** contains multiple UnitTasks organized as a dependency graph. It's useful for complex features that need to be broken into steps.

Statuses:
- `planning` - AI is generating a PLAN.yaml
- `pending_approval` - Waiting for you to approve the plan
- `in_progress` - Tasks are being executed
- `done` - All tasks completed
- `rejected` - Plan was rejected

### TodoItems

**TodoItems** are tasks that require human attention but can be AI-assisted:

- **Issue Triage**: New issues that need labeling/assignment
- **PR Review**: Pull requests that need review

---

## Using the Desktop App

### Dashboard

The dashboard shows your tasks in a Kanban board layout:

- **In Progress**: Tasks the AI is currently working on
- **In Review**: Tasks ready for your review
- **PR Open**: Tasks with open pull requests
- **Done**: Completed tasks
- **Rejected**: Tasks you've rejected

Below the board, you'll see **TodoItems** requiring attention.

### Navigation

- **Dashboard**: Main view with task board
- **Tasks**: List view of all tasks
- **Repositories**: Manage your repositories
- **Settings**: Configure DeliDev

### Command Palette

Press `Cmd/Ctrl + K` to open the command palette for quick access to any action.

---

## Creating Tasks

### Creating a UnitTask

1. Click **New Task** or press `Cmd/Ctrl + N`
2. Select a **Repository Group** (or create one)
3. Enter your **Prompt** describing what you want the AI to do
4. Optionally set:
   - **Title**: A short name for the task
   - **Branch Name**: Custom branch name
   - **AI Agent**: Which agent to use
5. Click **Create Task**

### Writing Good Prompts

For best results:

- Be specific about what you want
- Reference specific files with `@` mentions
- Describe the expected outcome
- Include any constraints or preferences

**Good prompt:**
```
Fix the login bug where users are logged out after 5 minutes.
The issue is likely in @src/auth/session.ts. The session
timeout should be 24 hours, not 5 minutes.
```

**Poor prompt:**
```
Fix the login
```

### Creating a CompositeTask

For complex features requiring multiple steps:

1. Click **New Composite Task**
2. Select a **Repository Group**
3. Describe the overall feature
4. The AI will create a PLAN.yaml breaking it into steps
5. Review the plan:
   - **Approve**: Accept the plan and start execution
   - **Update Plan**: Provide additional instructions to refine the plan. The AI will re-generate the plan incorporating your feedback
   - **Reject**: Discard the plan entirely
6. DeliDev executes the approved tasks in dependency order

> **Tip**: If the plan needs adjustments, use **Update Plan** instead of rejecting. You can also update the plan after rejection or failure by clicking **Update Plan** on the task details page.

---

## Reviewing AI Work

### Review Interface

When a task is `in_review`:

1. Click on the task to open the review interface
2. You'll see:
   - **File Tree**: Changed files
   - **Diff Viewer**: Side-by-side or unified view
   - **AI Session Log**: What the AI did

### Review Actions

- **Approve**: Accept the changes as-is
- **Request Changes**: Send feedback for the AI to iterate
- **Reject**: Discard the changes
- **Create PR**: Create a pull request with the changes

### Providing Feedback

When requesting changes:

1. Click **Request Changes**
2. Enter your feedback in the dialog
3. The AI will receive your feedback and make improvements
4. The task returns to `in_progress`

### TTY Input Requests

Sometimes the AI needs your input (e.g., confirming a destructive action). You'll see a notification when this happens.

1. Click the notification or the task
2. Answer the AI's question
3. The AI continues with your answer

---

## Managing Repositories

### Adding a Repository

1. Go to **Repositories**
2. Click **Add Repository**
3. Enter the repository URL (GitHub, GitLab, or Bitbucket)
4. DeliDev will detect the VCS type and default branch

### Repository Groups

Create groups to work with multiple repositories:

1. Go to **Repositories**
2. Click **Create Group**
3. Name the group
4. Select repositories to include

### Repository Settings

Each repository can have custom settings:

1. Open the repository
2. Click **Settings**
3. Configure:
   - Branch naming template
   - Auto-fix settings
   - AI agent preferences

---

## Configuration

### Global Settings

Location: `~/.delidev/config.toml`

```toml
[hotkey]
openChat = "Option+Z"  # Global hotkey to open DeliDev

[notification]
enabled = true
approvalRequest = true
userQuestion = true
reviewReady = true

[agent.execution]
type = "claude_code"
model = "claude-sonnet-4-20250514"

[container]
runtime = "docker"
use_container = true
```

### Repository Settings

Location: `.delidev/config.toml` in your repository

```toml
[branch]
template = "feature/${taskId}-${slug}"

[automation]
autoFixReviewComments = true
autoFixCIFailures = true
maxAutoFixAttempts = 3
```

### VCS Credentials

Location: `~/.delidev/credentials.toml`

```toml
[github]
token = "ghp_xxxxxxxxxxxx"

[gitlab]
token = "glpat-xxxxxxxxxxxx"
```

---

## Keyboard Shortcuts

### Global

| Shortcut | Action |
|----------|--------|
| `Option+Z` / `Alt+Z` | Open DeliDev (global hotkey) |
| `Cmd/Ctrl + N` | New task |
| `Cmd/Ctrl + ,` | Open settings |
| `Cmd/Ctrl + K` | Command palette |
| `Escape` | Close dialog/modal |

### Tab Navigation

| Shortcut | Action |
|----------|--------|
| `Cmd/Ctrl + T` | New tab |
| `Cmd/Ctrl + W` | Close tab |
| `Cmd/Ctrl + Tab` | Next tab |
| `Cmd/Ctrl + Shift + Tab` | Previous tab |

### Review Interface

| Shortcut | Action |
|----------|--------|
| `J` / `K` | Navigate files |
| `Enter` | Open selected file |
| `Cmd/Ctrl + Enter` | Approve task |
| `R` | Request changes |

---

## Troubleshooting

### Common Issues

#### Docker Not Running

**Symptom**: Tasks fail to start

**Solution**:
1. Ensure Docker Desktop is running
2. Verify with `docker ps` in terminal
3. Restart Docker if needed

#### API Key Issues

**Symptom**: "Authentication failed" errors

**Solution**:
1. Go to Settings > Secrets
2. Verify your API keys are correct
3. Check the key hasn't expired

#### Task Stuck in Progress

**Symptom**: Task shows "in progress" but nothing is happening

**Solution**:
1. Check the session log for errors
2. Try stopping and retrying the task
3. Verify Docker container is running

#### Connection Issues (Remote Mode)

**Symptom**: Can't connect to server

**Solution**:
1. Verify server URL in settings
2. Check network connectivity
3. Ensure server is running

### Getting Help

- **Documentation**: Check the [docs folder](/docs)
- **Issues**: Report bugs on GitHub
- **Logs**: Check `~/.delidev/logs/` for detailed logs

### Reset DeliDev

To start fresh:

1. Quit DeliDev
2. Delete `~/.delidev/` directory
3. Relaunch DeliDev

**Warning**: This will remove all local data and settings.

---

## Tips and Best Practices

### Writing Effective Prompts

1. **Be specific**: Include file paths, function names, error messages
2. **Provide context**: Explain why you need the change
3. **Set constraints**: Mention any requirements (e.g., "maintain backward compatibility")
4. **Reference files**: Use `@filename` to reference specific files

### Organizing Work

1. Create separate workspaces for different projects
2. Use repository groups for related repos
3. Name tasks clearly for easy tracking
4. Review and close completed tasks regularly

### Security

1. Never commit `.delidev/` directories to version control
2. Use environment-specific API keys
3. Rotate API keys periodically
4. Review AI changes before approving

---

Happy coding with DeliDev!
