# PR Management

DeliDev includes polling-based PR management for PRs created by DeliDev tasks.

## Scope

1. track PR state changes
2. detect actionable review activity
3. detect CI failures
4. trigger remediation subtasks
5. support manual and automatic fix runs

## Core Entities

1. `PullRequestTracking`
2. `ReviewAssistItem`
3. `SubTask` with type `PR_REVIEW_FIX` or `PR_CI_FIX`

See `docs/entities.md`.

## Polling Loop

1. scheduler selects active PR tracking records
2. poll provider APIs (GitHub/GitLab/etc)
3. normalize state into `PrStatus`
4. detect deltas since last snapshot
5. persist updates and emit stream events

## Actionable Signals

1. review requested changes
2. new unresolved review threads
3. CI failed checks
4. merge conflict indicators

## Manual Remediation Flow

1. UI shows `Fix with Agent`
2. client calls `RunAutoFixNow`
3. server creates remediation SubTask
4. worker executes and streams results
5. PR status is re-polled and reflected

## Automatic Remediation Flow

1. workspace or PR policy enables auto-fix
2. actionable signal detected
3. attempt count checked against max attempts
4. remediation SubTask auto-created
5. attempt counter incremented
6. stream events and notifications emitted

## Retry and Guardrails

1. max attempts per PR tracking record
2. cooldown between automatic runs
3. blocked state if repeated failures exceed cap
4. explicit user action required to resume auto-fix

## UI Requirements

1. PR list includes latest signal summary
2. quick action buttons for manual fix and policy toggle
3. clear display of attempt budget and recent outcomes
4. deep links to task, subtask, and PR

## Logging Requirements

Server logs include:

1. provider polling request and response metadata
2. diff detection result
3. auto-fix decision and reason
4. remediation subtask IDs

## Failure Handling

1. provider API unavailable: keep stale snapshot with warning state
2. permission denied on PR: mark tracking as blocked and notify user
3. repeated remediation failure: disable auto-fix and require manual review
