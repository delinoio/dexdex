# DeliDev User Guide

This guide explains the DeliDev user workflow.

## 1. Create a Workspace

1. open Workspace settings
2. click `Create Workspace`
3. enter name and endpoint URL
4. choose workspace type:
- Local Endpoint Workspace
- Remote Endpoint Workspace
5. save and set active workspace

## 2. Add Repositories

1. open Repositories
2. add repository remote URLs
3. create RepositoryGroup for related repositories
4. order repositories intentionally (first repository becomes primary agent execution repository)

## 3. Create a UnitTask

1. click `New UnitTask`
2. select RepositoryGroup
3. enter title and prompt
4. submit

DeliDev creates the initial SubTask and starts AgentSession execution.

## 4. Monitor Execution

In UnitTask detail:

1. SubTask status timeline
2. AgentSession logs
3. generated commit chain summary
4. action badges

Multiline input tip:

1. use `Cmd+Enter` to submit any multiline form
2. use `Enter` for a new line

## 5. Work with Multiple Tabs

You can open and manage multiple item tabs in parallel:

1. open an item with `Enter` or open in a new tab with `Cmd+Enter`
2. create an empty tab with `Cmd+T`
3. move between tabs with `Cmd+Shift+[` and `Cmd+Shift+]`
4. close current tab with `Cmd+W`

## 6. Stop Running Work

You can stop running work immediately:

1. in UnitTask detail, click `Stop UnitTask` while task is in progress
2. in SubTask timeline, click `Stop SubTask` on a running subtask
3. wait for status to change to `CANCELLED` from stream updates

## 7. Handle Action Badges

When user action is needed, UnitTask shows badges such as:

1. Review Requested
2. Plan Approval Required
3. CI Failed

Badge colors depend on workspace badge settings.

## 8. Use Plan Mode

If the session enters plan wait:

1. review proposed plan
2. choose `Approve`, `Revise`, or `Reject`
3. optionally add revise feedback

## 9. Create PR After Diff Approval

After you approve the AI diff in UnitTask detail:

1. click `Create PR`
2. DeliDev starts SubTask `PR_CREATE`
3. coding agent receives prompt `Create A PR`
4. worker creates real git commits for the result
5. PR appears in PR Management when completed

## 10. Manage PRs

Open PR Management to:

1. see tracked PR status
2. review comments and CI outcomes
3. run `Fix with Agent`
4. enable or disable auto-fix policy

## 11. Use Review Assist

Open Review Assist to:

1. inspect prioritized review and CI items
2. open linked code context
3. resolve or dismiss items
4. trigger remediation subtasks

## 12. Add Inline Comments in Code Review

In UnitTask detail or PR review diff:

1. move to a changed line
2. add inline comment on that line
3. submit with `Cmd+Enter`
4. resolve or reopen comments as review progresses

## 13. Notifications

DeliDev sends:

1. in-app notifications
2. Web Notification API notifications when permission is granted

The app requests notification permission during startup.

Manage notification behavior in Settings.

## 14. Product Rules

1. DeliDev uses workspace connectivity
2. all business communication is Connect RPC-based
3. direct local folder execution is not supported
4. work is executed through task-specific worktrees
5. PR creation and Commit to Local use real generated commit chains

## 15. Keyboard Shortcuts

All screens provide shortcuts for key items and actions.

Common shortcuts:

1. `Cmd+K`: command palette
2. `Cmd+N`: new UnitTask
3. `J` / `K`: move selection in lists
4. `Enter`: open selected item
5. `Cmd+Enter`: submit multiline forms
6. `Cmd+T`: new tab
7. `Cmd+W`: close current tab
8. `Cmd+Shift+[` / `Cmd+Shift+]`: previous and next tab
9. `?`: open shortcut cheat sheet

Each primary screen also has dedicated item/action shortcuts shown in the UI hints.
All shortcuts work regardless of Korean/English input mode.
Some shortcuts are context-sensitive based on focused control (for example `Cmd+Enter`).
