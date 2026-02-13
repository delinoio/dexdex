# Worker Server (Go)

Worker Server executes SubTasks using AI coding agents in isolated worktree environments.

## Responsibilities

1. prepare and manage git worktrees
2. launch and supervise coding agent sessions
3. stream runtime output and events to main server
4. persist session artifacts (patch refs, logs, summaries, usage, cost)
5. handle cancellation and retry-safe termination

## Execution Principles

1. worktree-only execution
2. one SubTask execution context per worktree
3. one or more AgentSession runs per SubTask
4. deterministic cleanup policy by outcome

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
│  Local Resources                                             │
│    ├── Repository Cache                                      │
│    ├── Worktrees                                             │
│    └── Temporary Session Data                                │
└──────────────────────────────────────────────────────────────┘
```

## Worktree Lifecycle

1. resolve repository from workspace and repository group context
2. ensure repository cache is up to date
3. create task-specific worktree path
4. execute agent sessions in that worktree
5. export patch and metadata
6. cleanup according to retention policy

Path convention:

- cache: `~/.delidev/repo-cache/<repo-hash>/`
- worktree: `~/.delidev/worktrees/<unit-task-id>/<sub-task-id>/`

## SubTask and Session Flow

1. worker receives `RunSubTask`
2. worker emits `SUBTASK_UPDATED(IN_PROGRESS)`
3. worker starts AgentSession
4. session output is streamed as incremental events
5. if plan mode is active, session waits for decision
6. worker emits completion or failure and artifact metadata

## Plan Mode Support

When `plan_mode_enabled = true` on SubTask:

1. agent emits plan proposal checkpoints
2. worker pauses execution at decision boundaries
3. main server relays user decision (`APPROVE` / `REVISE` / `REJECT`)
4. worker resumes or finalizes accordingly

## PR Remediation SubTasks

Worker supports remediation subtask types:

1. `PR_REVIEW_FIX`
2. `PR_CI_FIX`

Both run with the same worktree policy and event contract.

## Agent Abstraction

Worker uses adapter interfaces for multiple agents.
Each adapter provides:

1. command construction
2. structured output parsing
3. typed event mapping
4. cancellation and termination handling

## Token Usage and Cost Tracking

Worker tracks token usage and cost for every AgentSession.
This applies to Claude Code, Codex, OpenCode, and any additional adapter.

Requirements:

1. collect provider-reported usage counters per session
2. normalize usage into a shared `tokenUsage` schema
3. compute or ingest `total_cost_usd` from provider pricing metadata
4. emit usage checkpoints during long-running sessions
5. emit final usage and cost summary at session completion
6. send normalized usage and cost data to main server for persistence

Normalized fields:

1. `provider`
2. `model`
3. `inputTokens`
4. `outputTokens`
5. `cacheReadTokens`
6. `cacheWriteTokens`
7. `totalTokens`
8. `totalCostUsd`
9. `pricingVersion`
10. `capturedAt`

If an agent does not expose a counter, the field is `null` and raw usage payload is retained for audit.

## Error Handling

1. repository fetch failure: emit classified provider error
2. session startup failure: mark session and subtask failed
3. tool execution timeout: emit timeout event and cancel session
4. partial output corruption: emit parser warning and continue where possible

## Configuration

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

1. worktree create and cleanup
2. session start and stop
3. plan-mode wait and resume
4. artifact export
5. usage checkpoints and final cost summary
6. cancellation checkpoints

## Security Baseline

1. sanitize and validate repository URLs and branch refs
2. never log secret values
3. inject secrets only at session runtime scope
4. clear ephemeral secret material after session end
