# Worker Server (Go) - To-Be Design

Worker Server executes SubTasks using AI coding agents in isolated worktree environments.

## Responsibilities

1. Prepare and manage git worktrees
2. Launch and supervise coding agent sessions
3. Stream runtime output/events to main server
4. Persist session artifacts (patch refs, logs, summaries)
5. Handle cancellation and retry-safe termination

## Execution Principles

1. Worktree-only execution (no direct arbitrary folder mode)
2. One SubTask execution context per worktree
3. One or more AgentSession runs per SubTask
4. Deterministic cleanup policy by outcome

## Runtime Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ Worker Server (Go)                                           │
│                                                              │
│  Job Receiver (Connect RPC client/server)                    │
│    └── SubTask Runner                                        │
│         ├── Worktree Manager                                 │
│         ├── Agent Adapter Layer                              │
│         ├── Session Event Emitter                            │
│         └── Artifact Collector                               │
│                                                              │
│  Local Resources                                              │
│    ├── Repository Cache                                       │
│    ├── Worktrees                                              │
│    └── Temporary Session Data                                 │
└──────────────────────────────────────────────────────────────┘
```

## Worktree Lifecycle

1. Resolve repository from workspace + repository group context.
2. Ensure repository cache is up to date.
3. Create task-specific worktree path.
4. Execute agent session(s) inside that worktree.
5. Export patch/metadata.
6. Cleanup according to retention policy.

Example path convention:

- cache: `~/.delidev/repo-cache/<repo-hash>/`
- worktree: `~/.delidev/worktrees/<unit-task-id>/<sub-task-id>/`

## SubTask and Session Flow

1. Worker receives `RunSubTask` command.
2. Worker emits `SUBTASK_UPDATED(IN_PROGRESS)`.
3. Worker starts `AgentSession`.
4. Session output is streamed as incremental events.
5. If Plan Mode is active, session enters waiting state until decision is submitted.
6. Worker emits completion/failure and artifact metadata.

## Plan Mode Support

When `plan_mode_enabled = true` on SubTask:

1. Agent can emit plan proposal checkpoints.
2. Worker pauses forward execution at decision boundaries.
3. Main server relays user decision (`APPROVE` / `REVISE` / `REJECT`).
4. Worker resumes or finalizes accordingly.

## PR Remediation SubTasks

Worker must support remediation subtask types:

1. `PR_REVIEW_FIX`
2. `PR_CI_FIX`

Both types run in the same worktree policy and emit standard task/session events.

## Agent Abstraction

Worker uses adapter interfaces for multiple agents.
Each adapter must provide:

1. command construction
2. structured output parsing
3. typed event mapping
4. cancellation/termination handling

## Error Handling

1. repository fetch failure: emit failure event with provider error classification
2. session startup failure: mark session failed and subtask failed
3. tool execution timeout: emit timeout event and cancel session
4. partial output corruption: emit parser warning and continue stream where possible

## Configuration (Target)

| Key | Required | Description |
|---|---|---|
| `DELIDEV_WORKER_ID` | Y | Stable worker identifier |
| `DELIDEV_MAIN_SERVER_URL` | Y | Main server endpoint |
| `DELIDEV_WORKTREE_ROOT` | N | Worktree base path |
| `DELIDEV_REPO_CACHE_ROOT` | N | Repository cache root |
| `DELIDEV_MAX_PARALLEL_SUBTASKS` | N | Concurrency cap |
| `DELIDEV_AGENT_EXEC_TIMEOUT_SEC` | N | Session timeout |

## Logging Requirements

Emit structured logs for:

1. worktree create/cleanup
2. session start/stop
3. plan-mode wait/resume
4. artifact export
5. cancellation checkpoints

## Security Baseline

1. sanitize and validate all repository URLs and branch refs
2. never log secret values
3. inject secrets only at session runtime scope
4. clear ephemeral secret material after session end

## Migration Note

Worker no longer supports CompositeTask graph execution.
