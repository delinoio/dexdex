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
3. generated patch summary
4. action badges

## 5. Handle Action Badges

When user action is needed, UnitTask shows badges such as:

1. Review Requested
2. Plan Approval Required
3. CI Failed

Badge colors depend on workspace badge settings.

## 6. Use Plan Mode

If the session enters plan wait:

1. review proposed plan
2. choose `Approve`, `Revise`, or `Reject`
3. optionally add revise feedback

## 7. Manage PRs

Open PR Management to:

1. see tracked PR status
2. review comments and CI outcomes
3. run `Fix with Agent`
4. enable or disable auto-fix policy

## 8. Use Review Assist

Open Review Assist to:

1. inspect prioritized review and CI items
2. open linked code context
3. resolve or dismiss items
4. trigger remediation subtasks

## 9. Notifications

DeliDev sends:

1. in-app notifications
2. Web Notification API notifications when permission is granted

Manage notification behavior in Settings.

## 10. Product Rules

1. DeliDev uses workspace connectivity
2. all business communication is Connect RPC-based
3. direct local folder execution is not supported
4. work is executed through task-specific worktrees
