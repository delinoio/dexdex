# Plan Mode Interaction Spec (To-Be)

This document defines how DeliDev supports plan-mode capable coding agents.

## Scope

Plan mode applies at SubTask execution level.
It does not introduce CompositeTask-style orchestration entities.

## Key Concepts

1. Plan proposal checkpoints are emitted during an AgentSession.
2. Session can pause waiting for explicit user decision.
3. User decision is submitted via Connect RPC.
4. Session resumes or terminates based on decision.

## States

### SubTaskStatus additions used in plan flow

1. `WAITING_FOR_PLAN_APPROVAL`
2. `IN_PROGRESS`
3. `COMPLETED`
4. `FAILED`
5. `CANCELLED`

### AgentSessionStatus additions used in plan flow

1. `RUNNING`
2. `WAITING_FOR_INPUT`

## Decision Actions

User can submit one decision:

1. `APPROVE`
- continue execution with current plan

2. `REVISE`
- continue execution after applying feedback

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
3. session wait/resume transition in `SESSION_STATE_CHANGED`

## UI Behavior

When subtask is waiting for plan approval:

1. show plan proposal panel
2. lock conflicting destructive actions
3. enable approve/revise/reject controls
4. show audit trail of decisions

## Audit and Persistence

Persist:

1. decision type
2. feedback text (if provided)
3. decision timestamp
4. acting user ID
5. linked session ID and subtask ID

## Failure Modes

1. decision timeout policy exceeded -> subtask marked failed or cancelled by policy
2. invalid decision state transition -> `FAILED_PRECONDITION`
3. stream disconnect during waiting state -> recoverable via reconnect and state fetch
