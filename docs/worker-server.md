# Worker Server (Go)

Worker Server executes SubTasks using AI coding agents in isolated worktree environments.

## Responsibilities

1. prepare and manage git worktrees
2. launch and supervise coding agent sessions
3. normalize provider-native agent output into shared message contracts
4. stream normalized runtime output and events to main server
5. persist session artifacts (commit chains, patch refs, logs, summaries, usage, cost)
6. handle cancellation and retry-safe termination

## Execution Principles

1. worktree-only execution
2. one SubTask execution context per RepositoryGroup
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

1. resolve ordered repositories from workspace and repository group context
2. ensure each repository cache is up to date
3. create task-specific worktree path for every repository in the group
4. choose the first repository worktree as primary execution directory
5. execute agent sessions in the primary directory
6. pass remaining repository worktrees as `--add-dir` (or equivalent agent options)
7. create and persist real git commits in branch history
8. export patch and metadata derived from commits
9. cleanup according to retention policy

Path convention:

- cache: `~/.delidev/repo-cache/<repo-hash>/`
- worktree: `~/.delidev/worktrees/<unit-task-id>/<sub-task-id>/<repo-id>/`

## SubTask and Session Flow

1. worker receives `RunSubTask`
2. worker emits `SUBTASK_UPDATED(IN_PROGRESS)`
3. worker starts AgentSession
4. adapter converts provider-native output into normalized `SessionOutputEvent`
5. normalized session output is streamed as incremental events
6. if plan mode is active, session waits for decision
7. worker emits completion or failure and artifact metadata

Cancellation path:

1. worker receives cancel signal for UnitTask or SubTask.
2. worker terminates active agent processes for affected SubTask immediately.
3. worker emits cancellation lifecycle events.
4. worker returns final `CANCELLED` status to main server.

## Plan Mode Support

When `plan_mode_enabled = true` on SubTask:

1. agent emits plan proposal checkpoints
2. worker pauses execution at decision boundaries
3. main server relays user decision (`APPROVE` / `REVISE` / `REJECT`)
4. worker resumes or finalizes accordingly

## PR Remediation SubTasks

Worker supports remediation subtask types:

1. `PR_CREATE`
2. `PR_REVIEW_FIX`
3. `PR_CI_FIX`

Both run with the same worktree policy and event contract.

## Real Commit Requirement

Worker output must be a real git commit chain.

1. if a SubTask makes code changes, it must create real commits in the worktree branch
2. multiple logical changes should produce multiple commits
3. worker persists ordered commit metadata (`sha`, parents, message, timestamps)
4. patch artifacts are generated from those commits for diff views
5. PR creation and Commit to Local must consume commit-chain metadata, not patch-only output

## Primary Repository Launch Rule

Agent process launch uses the RepositoryGroup ordering rule.

1. first repository in `repositoryIds` is the launch directory
2. all other repositories are attached using `--add-dir` (or equivalent option per agent adapter)
3. adapter command builders must preserve directory order when constructing arguments

## Agent Abstraction

Worker uses adapter interfaces for multiple agents.
Each adapter provides:

1. command construction
2. structured output parsing
3. provider-to-normalized event mapping
4. cancellation and termination handling

## Agent Message Normalization (Required)

Worker is the only component that handles provider-native agent message formats.

1. parse provider-native output in adapter runtime
2. map provider-specific event shapes to normalized `SessionOutputEvent`
3. map provider-specific lifecycle signals to normalized session state updates
4. send only normalized session messages to main server
5. keep provider-native raw payloads worker-local for debug if needed

Main server and Tauri client never parse provider-native output formats.

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

If an agent does not expose a counter, the field is `null`.
Provider-native raw usage payload may be retained only in worker-local debug storage.

## Error Handling

1. repository fetch failure: emit classified provider error
2. session startup failure: mark session and subtask failed
3. tool execution timeout: emit timeout event and cancel session
4. partial output corruption: emit parser warning and continue where possible
5. user cancellation request: stop process immediately and flush cancellation events

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
4. commit chain generation and commit count
5. artifact export
6. usage checkpoints and final cost summary
7. cancellation checkpoints
8. primary repository selection and add-dir argument mapping

## Security Baseline

1. sanitize and validate repository URLs and branch refs
2. never log secret values
3. inject secrets only at session runtime scope
4. clear ephemeral secret material after session end
