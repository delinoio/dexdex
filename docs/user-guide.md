# DeliDev User Guide (To-Be)

This guide explains the target user workflow for the rewritten DeliDev.

## 1. Create a Workspace

1. Open Workspace settings.
2. Click `Create Workspace`.
3. Enter name and endpoint URL.
4. Choose workspace type:
- Local Endpoint Workspace
- Remote Endpoint Workspace
5. Save and set as active.

## 2. Add Repositories

1. Open Repositories.
2. Add repository remote URLs.
3. Create RepositoryGroup for related repositories.

## 3. Create a UnitTask

1. Click `New UnitTask`.
2. Select RepositoryGroup.
3. Enter title and prompt.
4. Submit.

DeliDev creates the initial SubTask and starts AgentSession execution.

## 4. Monitor Execution

In UnitTask detail you can track:

1. SubTask status timeline
2. AgentSession logs
3. generated patch summary
4. action badges

## 5. Handle Action Badges

When user action is needed, UnitTask shows badges such as:

1. Review Requested
2. Plan Approval Required
3. CI Failed

Badge colors depend on your workspace badge settings.

## 6. Use Plan Mode (When Agent Supports It)

If the session enters plan wait:

1. review proposed plan
2. choose `Approve`, `Revise`, or `Reject`
3. optionally add feedback for revise

## 7. Manage PRs

Open PR Management screen to:

1. see tracked PR status
2. review new comments and CI outcomes
3. run `Fix with Agent`
4. enable or disable auto-fix policy

## 8. Review Assist

Open Review Assist screen to:

1. inspect prioritized review/CI items
2. open linked code context
3. resolve or dismiss items
4. trigger remediation subtasks quickly

## 9. Notifications

DeliDev sends:

1. in-app notifications
2. Web Notification API notifications (if permission granted)

You can manage notification behavior in Settings.

## 10. Important Product Rules

1. DeliDev uses workspace connectivity, not mode switching.
2. All business communication is Connect RPC-based.
3. Direct local folder execution is not supported.
4. Work is executed through task-specific worktrees.
