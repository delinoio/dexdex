# UI Design (Codex Desktop Inspired)

This document defines DeliDev UI and UX structure.

## UX Goals

1. Fast triage-first interface for tasks and PR events
2. Persistent visibility of AI activity timeline
3. One-click remediation for review and CI issues
4. Consistent desktop and mobile mental model

## Visual Direction

Inspired by Codex Desktop style:

1. workspace-oriented shell
2. dense and readable information layout
3. timeline-driven debugging and review
4. clear action-required emphasis

## Layout Blueprint

Desktop layout:

1. Left rail: workspace and navigation
2. Center pane: task and PR list with filters
3. Right pane: details, timeline, and actions

Mobile layout:

1. top segmented tabs (Tasks, PRs, Review Assist, Notifications)
2. stacked detail drawers and sheets
3. persistent action bar for primary actions

## Primary Screens

1. Workspace Home
2. UnitTask Detail
3. PR Management
4. PR Review Assist
5. Settings
6. Notifications Center

## Workspace Home

Shows actionable work first.

Sections:

1. Action Required queue
2. In Progress queue
3. PR Attention queue
4. Completed recently

Each card includes:

1. title
2. repository/group context
3. status
4. action badges
5. latest subtask or session timestamp

## UnitTask Detail

Contains:

1. summary header
2. status and action badges
3. SubTask timeline
4. AgentSession logs per subtask
5. patch and diff preview
6. plan-mode decision controls when active
7. `Create PR` action shown after AI diff approval
8. commit chain viewer for generated commits

## Multiline Input Submit Shortcut

All multiline inputs support `Cmd+Enter` for form submission.

Applies to:

1. UnitTask creation prompt input
2. SubTask feedback and retry prompt input
3. Plan-mode revise feedback input
4. PR review assist note and comment input

Behavior:

1. `Enter` inserts a newline
2. `Cmd+Enter` submits the current form
3. submit button remains available as an alternative

### Approved Diff PR Action

When a user approves the AI diff in UnitTask detail:

1. show `Create PR` button
2. on click, create SubTask with type `PR_CREATE`
3. send simple prompt `Create A PR` to coding agent
4. stream the SubTask and AgentSession progress in the same timeline
5. render generated real commit list in order
6. update PR tracking state after creation

### Commit Chain Panel

UnitTask detail shows commit chain metadata per SubTask:

1. commit SHA
2. commit title
3. author and timestamp
4. commit order index

`Create PR` and `Commit to Local` both use this commit chain.

## Action Badge System

DeliDev highlights UnitTasks requiring user action with badges.

### Badge Rules

1. one UnitTask can have multiple action badges
2. badges map from `ActionType` to `BadgeColorKey`
3. default mapping exists per workspace
4. users can override mapping in Settings

### Example Mapping

- `REVIEW_REQUESTED` -> `BLUE`
- `PR_CREATION_READY` -> `GREEN`
- `PLAN_APPROVAL_REQUIRED` -> `YELLOW`
- `CI_FAILED` -> `RED`
- `MERGE_CONFLICT` -> `ORANGE`
- `USER_INPUT_REQUIRED` -> `GREEN`

## PR Management Screen

Purpose: polling-driven operations for PRs created by DeliDev tasks.

Columns:

1. PR metadata and state
2. latest review and CI signals
3. auto-fix policy state
4. quick actions

Primary actions:

1. `Fix with Agent`
2. `Enable Auto-Fix`
3. `Disable Auto-Fix`
4. `Open PR`

## PR Review Assist Screen

Displays review guidance items grouped by urgency and type.

Each item includes:

1. signal source (review, CI, risk)
2. summary and details
3. links to code and PR context
4. quick action to create remediation subtask

## Plan Mode UX

When plan mode is active for a subtask:

1. show current proposal in the detail pane
2. show decision controls: `Approve`, `Revise`, `Reject`
3. require explicit decision before execution continues
4. preserve the full plan conversation in session timeline

## Notifications UX

1. bell icon with unread count
2. notification center list with deep links
3. Web Notification API permission prompt flow
4. duplicate prevention by event sequence

## Accessibility Baseline

1. keyboard-first action flow on desktop
2. minimum contrast ratios for badges and statuses
3. semantic heading and landmark structure
4. reduced-motion support for streaming updates

## Responsive Breakpoints

1. `>= 1280`: 3-pane layout
2. `>= 768 and < 1280`: 2-pane layout
3. `< 768`: stacked mobile flow

## Empty and Error States

1. no workspace configured
2. stream disconnected
3. PR provider rate-limited
4. no actionable items

Each state includes direct recovery actions.
