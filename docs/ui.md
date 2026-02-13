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

## Keyboard Shortcut System

Every screen provides appropriate shortcuts for its primary items and actions.

Notation rule:

1. shortcut notation uses macOS style (`Cmd`, `Option`, `Shift`)
2. non-mac platforms map `Cmd` to `Ctrl`
3. shortcut matching uses physical key codes, not localized character output
4. shortcuts work regardless of current language input mode (Korean/English IME)

Global shortcuts:

1. `Cmd+K`: open command palette
2. `Cmd+1`: go to Workspace Home
3. `Cmd+2`: go to PR Management
4. `Cmd+3`: go to PR Review Assist
5. `Cmd+,`: open Settings
6. `Cmd+N`: create UnitTask
7. `Cmd+B`: toggle sidebar
8. `?`: open shortcut cheat sheet
9. `Esc`: close modal, drawer, or inline editor

Screen shortcut coverage rule:

1. each list screen supports next and previous item navigation (`J` / `K`)
2. each selected item supports open action (`Enter`)
3. each primary button action has a dedicated shortcut
4. shortcut hints are shown in tooltip, menu, or action label
5. active shortcuts must still trigger under IME language switching

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

Workspace Home shortcuts:

1. `J` / `K`: move selected card
2. `Enter`: open selected task or PR item
3. `A`: focus Action Required queue
4. `I`: focus In Progress queue
5. `P`: focus PR Attention queue

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
9. stop controls for in-progress UnitTask and SubTask

### Stop Controls

Users can stop running work with minimal friction:

1. show `Stop UnitTask` button when UnitTask is `IN_PROGRESS`
2. show `Stop SubTask` action on each in-progress SubTask row
3. stop action sends cancellation request immediately
4. UI reflects `CANCELLED` state from stream updates

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
4. `Cmd+Enter` submit works regardless of current IME language mode

### Approved Diff PR Action

When a user approves the AI diff in UnitTask detail:

1. show `Create PR` button
2. on click, create SubTask with type `PR_CREATE`
3. send simple prompt `Create A PR` to coding agent
4. stream the SubTask and AgentSession progress in the same timeline
5. render generated real commit list in order
6. update PR tracking state after creation

UnitTask Detail shortcuts:

1. `A`: approve current diff
2. `R`: request changes
3. `Shift+P`: create PR (when approval condition is met)
4. `C`: commit to local (when available)
5. `L`: toggle session log panel
6. `D`: toggle diff panel
7. `[` / `]`: move to previous or next changed file
8. `S`: stop current UnitTask when in progress
9. `Shift+S`: stop selected SubTask when in progress

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

PR Management shortcuts:

1. `J` / `K`: move selected PR row
2. `Enter`: open selected PR detail
3. `F`: run `Fix with Agent`
4. `E`: toggle auto-fix policy for selected PR
5. `O`: open selected PR in provider page
6. `R`: refresh selected PR state

## PR Review Assist Screen

Displays review guidance items grouped by urgency and type.

Each item includes:

1. signal source (review, CI, risk)
2. summary and details
3. links to code and PR context
4. quick action to create remediation subtask

PR Review Assist shortcuts:

1. `J` / `K`: move selected review assist item
2. `Enter`: open selected item detail
3. `F`: create remediation subtask
4. `X`: resolve selected item
5. `Shift+X`: dismiss selected item

## Plan Mode UX

When plan mode is active for a subtask:

1. show current proposal in the detail pane
2. show decision controls: `Approve`, `Revise`, `Reject`
3. require explicit decision before execution continues
4. preserve the full plan conversation in session timeline

Plan Mode shortcuts:

1. `A`: approve plan
2. `V`: open revise input
3. `Shift+X`: reject plan

## Notifications UX

1. bell icon with unread count
2. notification center list with deep links
3. Web Notification API permission prompt flow
4. duplicate prevention by event sequence

Notifications Center shortcuts:

1. `J` / `K`: move selected notification
2. `Enter`: open deep link for selected notification
3. `M`: mark selected notification as read
4. `Shift+M`: mark all visible notifications as read

Settings shortcuts:

1. `/`: focus settings search input
2. `Cmd+S`: save settings form
3. `R`: reset current settings section

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
