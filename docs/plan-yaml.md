# Plan Mode Interaction Spec

This document defines how DexDex supports plan-mode capable coding agents.

## Scope

Plan mode applies at SubTask execution level.

## Key Concepts

1. plan proposal checkpoints are emitted during an AgentSession
2. session can pause waiting for explicit user decision
3. user decision is submitted via Connect RPC
4. session resumes or terminates based on decision

## States

### SubTaskStatus used in plan flow

1. `WAITING_FOR_PLAN_APPROVAL`
2. `IN_PROGRESS`
3. `COMPLETED`
4. `FAILED`
5. `CANCELLED`

### AgentSessionStatus used in plan flow

1. `RUNNING`
2. `WAITING_FOR_INPUT`

## Decision Actions

User submits one decision:

1. `APPROVE`
- continue execution with current plan

2. `REVISE`
- continue execution after feedback is applied

3. `REJECT`
- terminate current execution path and return control to user

## RPC Contract

Use `TaskService.SubmitPlanDecision`.

Request fields:

1. `sub_task_id`
2. `decision`
3. optional `feedback`

## Event Stream Contract

Plan-mode relevant event payloads:

1. plan proposal emitted in `SESSION_OUTPUT`
2. subtask status transition in `SUBTASK_UPDATED`
3. session wait and resume transition in `SESSION_STATE_CHANGED`

## UI Behavior

When subtask is waiting for plan approval:

1. show plan proposal panel
2. lock conflicting destructive actions
3. enable approve, revise, and reject controls
4. show audit trail of decisions
5. in revise feedback multiline input, `Cmd+Enter` submits the revise request
6. provide keyboard shortcuts for decisions (`A` approve, `V` revise, `Shift+X` reject)
7. decision shortcuts must work regardless of current IME language mode

## Audit and Persistence

Persist:

1. decision type
2. feedback text if provided
3. decision timestamp
4. acting user ID
5. linked session ID and subtask ID

## Failure Modes

1. decision timeout policy exceeded: subtask marked failed or cancelled by policy
2. invalid decision state transition: return `FAILED_PRECONDITION`
3. stream disconnect during waiting state: recover via reconnect and state fetch
