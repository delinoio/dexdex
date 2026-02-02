# DeliDev API Documentation

This document describes the API endpoints available in the DeliDev Main Server.

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Task Management](#task-management)
4. [Session Management](#session-management)
5. [Repository Management](#repository-management)
6. [Workspace Management](#workspace-management)
7. [Todo Items](#todo-items)
8. [Secrets Management](#secrets-management)
9. [Worker Management](#worker-management)
10. [Webhooks](#webhooks)
11. [Error Handling](#error-handling)

---

## Overview

The DeliDev API is a RESTful API that uses JSON for request and response bodies. All endpoints are served over HTTP(S) and follow consistent patterns for error handling and pagination.

### Base URL

- **Local Mode**: `http://localhost:54871`
- **Remote Mode**: Configured via `DELIDEV_SERVER_URL`

### Content Type

All requests and responses use `application/json` content type.

---

## Authentication

In multi-user mode, all API requests (except authentication endpoints) require a valid JWT token in the `Authorization` header.

### Headers

```
Authorization: Bearer <jwt_token>
```

### Endpoints

#### Get Login URL

Returns the OIDC login URL for authentication.

- **URL**: `POST /api/auth/get-login-url`
- **Request Body**:
  ```json
  {
    "redirect_uri": "http://localhost:54871/callback"
  }
  ```
- **Response**:
  ```json
  {
    "url": "https://auth.provider.com/authorize?..."
  }
  ```

#### Handle Callback

Handles the OIDC callback and returns JWT tokens.

- **URL**: `GET /api/auth/callback`
- **Query Parameters**: `code`, `state`
- **Response**:
  ```json
  {
    "access_token": "jwt_token",
    "refresh_token": "refresh_token",
    "expires_in": 86400,
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "name": "User Name"
    }
  }
  ```

#### Refresh Token

Refreshes an expired JWT token.

- **URL**: `POST /api/auth/refresh`
- **Request Body**:
  ```json
  {
    "refresh_token": "refresh_token"
  }
  ```
- **Response**:
  ```json
  {
    "access_token": "new_jwt_token",
    "expires_in": 86400
  }
  ```

#### Get Current User

Returns the currently authenticated user.

- **URL**: `GET /api/auth/me`
- **Response**:
  ```json
  {
    "user": {
      "id": "uuid",
      "email": "user@example.com",
      "name": "User Name"
    }
  }
  ```

#### Logout

Invalidates the current session.

- **URL**: `POST /api/auth/logout`
- **Response**: `204 No Content`

---

## Task Management

### Create Unit Task

Creates a new UnitTask for a single repository operation.

- **URL**: `POST /api/task/create-unit`
- **Request Body**:
  ```json
  {
    "repository_group_id": "uuid",
    "prompt": "Fix the login bug",
    "title": "Bug Fix",
    "branch_name": "fix/login-bug",
    "ai_agent_type": "claude_code",
    "ai_agent_model": "claude-sonnet-4-20250514"
  }
  ```
- **Response**:
  ```json
  {
    "task": {
      "id": "uuid",
      "repository_group_id": "uuid",
      "agent_task_id": "uuid",
      "prompt": "Fix the login bug",
      "title": "Bug Fix",
      "branch_name": "fix/login-bug",
      "linked_pr_url": null,
      "base_commit": null,
      "end_commit": null,
      "auto_fix_task_ids": [],
      "status": "in_progress",
      "created_at": "2026-02-01T00:00:00Z",
      "updated_at": "2026-02-01T00:00:00Z"
    }
  }
  ```

### Create Composite Task

Creates a new CompositeTask that can contain multiple UnitTasks.

- **URL**: `POST /api/task/create-composite`
- **Request Body**:
  ```json
  {
    "repository_group_id": "uuid",
    "prompt": "Implement feature X with tests",
    "title": "Feature X",
    "execution_agent_type": "claude_code"
  }
  ```
- **Response**:
  ```json
  {
    "task": {
      "id": "uuid",
      "repository_group_id": "uuid",
      "planning_task_id": "uuid",
      "prompt": "Implement feature X with tests",
      "title": "Feature X",
      "node_ids": [],
      "status": "planning",
      "execution_agent_type": "claude_code",
      "created_at": "2026-02-01T00:00:00Z",
      "updated_at": "2026-02-01T00:00:00Z"
    }
  }
  ```

### Get Task

Gets a task by ID (either UnitTask or CompositeTask).

- **URL**: `POST /api/task/get`
- **Request Body**:
  ```json
  {
    "task_id": "uuid"
  }
  ```
- **Response** (UnitTask):
  ```json
  {
    "unit_task": { ... }
  }
  ```
- **Response** (CompositeTask):
  ```json
  {
    "composite_task": { ... }
  }
  ```

### List Tasks

Lists tasks with optional filters.

- **URL**: `POST /api/task/list`
- **Request Body**:
  ```json
  {
    "repository_group_id": "uuid",
    "unit_status": "in_progress",
    "composite_status": null,
    "limit": 20,
    "offset": 0
  }
  ```
- **Response**:
  ```json
  {
    "unit_tasks": [...],
    "composite_tasks": [...],
    "total_count": 42
  }
  ```

### Update Task Status

Updates the status of a task.

- **URL**: `POST /api/task/update-status`
- **Request Body**:
  ```json
  {
    "task_id": "uuid",
    "unit_status": "in_review"
  }
  ```
- **Response**: Updated task object

### Delete Task

Deletes a task.

- **URL**: `POST /api/task/delete`
- **Request Body**:
  ```json
  {
    "task_id": "uuid"
  }
  ```
- **Response**: `{}`

### Retry Task

Retries a failed task.

- **URL**: `POST /api/task/retry`
- **Request Body**:
  ```json
  {
    "task_id": "uuid"
  }
  ```
- **Response**: Updated task object

### Approve Task

Approves a task that's in review.

- **URL**: `POST /api/task/approve`
- **Request Body**:
  ```json
  {
    "task_id": "uuid"
  }
  ```
- **Response**: `{}`

### Reject Task

Rejects a task.

- **URL**: `POST /api/task/reject`
- **Request Body**:
  ```json
  {
    "task_id": "uuid",
    "reason": "Not suitable for this approach"
  }
  ```
- **Response**: `{}`

### Request Changes

Requests changes on a task.

- **URL**: `POST /api/task/request-changes`
- **Request Body**:
  ```json
  {
    "task_id": "uuid",
    "feedback": "Please also add unit tests"
  }
  ```
- **Response**: Updated task object with feedback appended to prompt

---

## Session Management

### Get Log

Gets the output log for a session.

- **URL**: `POST /api/session/get-log`
- **Request Body**:
  ```json
  {
    "session_id": "uuid"
  }
  ```
- **Response**:
  ```json
  {
    "log": "Session output text..."
  }
  ```

### Stop Session

Stops a running session.

- **URL**: `POST /api/session/stop`
- **Request Body**:
  ```json
  {
    "session_id": "uuid"
  }
  ```
- **Response**: `{}`

### Submit TTY Input

Submits a response to a TTY input request.

- **URL**: `POST /api/session/submit-tty-input`
- **Request Body**:
  ```json
  {
    "request_id": "uuid",
    "response": "yes"
  }
  ```
- **Response**: `{}`

---

## Repository Management

### Add Repository

Adds a new repository.

- **URL**: `POST /api/repository/add`
- **Request Body**:
  ```json
  {
    "vcs_type": "git",
    "vcs_provider_type": "github",
    "remote_url": "https://github.com/user/repo.git",
    "name": "my-repo",
    "default_branch": "main"
  }
  ```
- **Response**: Repository object

### List Repositories

Lists all repositories.

- **URL**: `POST /api/repository/list`
- **Response**:
  ```json
  {
    "repositories": [...]
  }
  ```

### Get Repository

Gets a repository by ID.

- **URL**: `POST /api/repository/get`
- **Request Body**:
  ```json
  {
    "repository_id": "uuid"
  }
  ```
- **Response**: Repository object

### Remove Repository

Removes a repository.

- **URL**: `POST /api/repository/remove`
- **Request Body**:
  ```json
  {
    "repository_id": "uuid"
  }
  ```
- **Response**: `{}`

### Create Repository Group

Creates a new repository group.

- **URL**: `POST /api/repository-group/create`
- **Request Body**:
  ```json
  {
    "workspace_id": "uuid",
    "name": "My Group",
    "repository_ids": ["uuid1", "uuid2"]
  }
  ```
- **Response**: Repository group object

### List Repository Groups

Lists all repository groups.

- **URL**: `POST /api/repository-group/list`
- **Request Body**:
  ```json
  {
    "workspace_id": "uuid"
  }
  ```
- **Response**:
  ```json
  {
    "groups": [...]
  }
  ```

### Update Repository Group

Updates a repository group.

- **URL**: `POST /api/repository-group/update`
- **Request Body**:
  ```json
  {
    "group_id": "uuid",
    "name": "Updated Name",
    "repository_ids": ["uuid1", "uuid2"]
  }
  ```
- **Response**: Updated repository group object

### Delete Repository Group

Deletes a repository group.

- **URL**: `POST /api/repository-group/delete`
- **Request Body**:
  ```json
  {
    "group_id": "uuid"
  }
  ```
- **Response**: `{}`

---

## Workspace Management

### Create Workspace

Creates a new workspace.

- **URL**: `POST /api/workspace/create`
- **Request Body**:
  ```json
  {
    "name": "My Workspace",
    "description": "A workspace for my projects"
  }
  ```
- **Response**: Workspace object

### List Workspaces

Lists all workspaces.

- **URL**: `POST /api/workspace/list`
- **Response**:
  ```json
  {
    "workspaces": [...]
  }
  ```

### Get Workspace

Gets a workspace by ID.

- **URL**: `POST /api/workspace/get`
- **Request Body**:
  ```json
  {
    "workspace_id": "uuid"
  }
  ```
- **Response**: Workspace object

### Update Workspace

Updates a workspace.

- **URL**: `POST /api/workspace/update`
- **Request Body**:
  ```json
  {
    "workspace_id": "uuid",
    "name": "Updated Name",
    "description": "Updated description"
  }
  ```
- **Response**: Updated workspace object

### Delete Workspace

Deletes a workspace.

- **URL**: `POST /api/workspace/delete`
- **Request Body**:
  ```json
  {
    "workspace_id": "uuid"
  }
  ```
- **Response**: `{}`

---

## Todo Items

### List Todo Items

Lists todo items with filters.

- **URL**: `POST /api/todo/list`
- **Request Body**:
  ```json
  {
    "repository_id": "uuid",
    "status": "pending",
    "limit": 20,
    "offset": 0
  }
  ```
- **Response**:
  ```json
  {
    "items": [...],
    "total_count": 10
  }
  ```

### Get Todo Item

Gets a todo item by ID.

- **URL**: `POST /api/todo/get`
- **Request Body**:
  ```json
  {
    "item_id": "uuid"
  }
  ```
- **Response**: Todo item object

### Update Todo Status

Updates the status of a todo item.

- **URL**: `POST /api/todo/update-status`
- **Request Body**:
  ```json
  {
    "item_id": "uuid",
    "status": "completed"
  }
  ```
- **Response**: Updated todo item

### Dismiss Todo

Dismisses a todo item.

- **URL**: `POST /api/todo/dismiss`
- **Request Body**:
  ```json
  {
    "item_id": "uuid"
  }
  ```
- **Response**: `{}`

---

## Secrets Management

### Send Secrets

Sends secrets from the client to the server for a specific task.

- **URL**: `POST /api/secrets/send`
- **Request Body**:
  ```json
  {
    "task_id": "uuid",
    "secrets": [
      { "key": "ANTHROPIC_API_KEY", "value": "sk-..." }
    ]
  }
  ```
- **Response**: `{}`

### Clear Secrets

Clears cached secrets for a task.

- **URL**: `POST /api/secrets/clear`
- **Request Body**:
  ```json
  {
    "task_id": "uuid"
  }
  ```
- **Response**: `{}`

---

## Worker Management

These endpoints are used internally by worker servers.

### Register Worker

Registers a worker with the main server.

- **URL**: `POST /api/worker/register`
- **Request Body**:
  ```json
  {
    "name": "worker-1",
    "callback_url": "http://worker-1:54872"
  }
  ```
- **Response**:
  ```json
  {
    "worker_id": "uuid"
  }
  ```

### Heartbeat

Sends a heartbeat to indicate the worker is alive.

- **URL**: `POST /api/worker/heartbeat`
- **Request Body**:
  ```json
  {
    "worker_id": "uuid"
  }
  ```
- **Response**: `{}`

### Get Next Task

Gets the next available task for the worker.

- **URL**: `POST /api/worker/get-task`
- **Request Body**:
  ```json
  {
    "worker_id": "uuid"
  }
  ```
- **Response**:
  ```json
  {
    "task_id": "uuid",
    "session_id": "uuid",
    "prompt": "...",
    "repository_info": { ... }
  }
  ```
  or `null` if no task available.

### Report Task Status

Reports the status of a task execution.

- **URL**: `POST /api/worker/report-status`
- **Request Body**:
  ```json
  {
    "worker_id": "uuid",
    "task_id": "uuid",
    "status": "completed",
    "output": "Task output log..."
  }
  ```
- **Response**: `{}`

### Get Secrets

Gets secrets for a task execution.

- **URL**: `POST /api/worker/get-secrets`
- **Request Body**:
  ```json
  {
    "worker_id": "uuid",
    "task_id": "uuid"
  }
  ```
- **Response**:
  ```json
  {
    "secrets": [
      { "key": "ANTHROPIC_API_KEY", "value": "sk-..." }
    ]
  }
  ```

---

## Webhooks

### GitHub Webhook

Handles GitHub webhook events for auto-fix features.

- **URL**: `POST /webhooks/github`
- **Headers**:
  - `X-Hub-Signature-256`: HMAC signature
  - `X-GitHub-Event`: Event type
- **Supported Events**:
  - `pull_request_review_comment`: Triggers auto-fix for review comments
  - `check_run`: Triggers auto-fix for CI failures
- **Response**: `200 OK` or `204 No Content`

---

## Error Handling

### Error Response Format

All errors follow a consistent format:

```json
{
  "error": {
    "code": "NOT_FOUND",
    "message": "Task not found"
  }
}
```

### HTTP Status Codes

| Status | Meaning |
|--------|---------|
| 200 | Success |
| 201 | Created |
| 204 | No Content |
| 400 | Bad Request - Invalid input |
| 401 | Unauthorized - Invalid or missing token |
| 403 | Forbidden - Insufficient permissions |
| 404 | Not Found - Resource doesn't exist |
| 409 | Conflict - Resource already exists |
| 500 | Internal Server Error |

### Error Codes

| Code | Description |
|------|-------------|
| `INVALID_REQUEST` | Request body is malformed or missing required fields |
| `NOT_FOUND` | The requested resource was not found |
| `ALREADY_EXISTS` | A resource with the same ID already exists |
| `UNAUTHORIZED` | Authentication required |
| `FORBIDDEN` | User doesn't have permission |
| `INTERNAL_ERROR` | An unexpected error occurred |

---

## Health Check

### Health Check Endpoint

- **URL**: `GET /health`
- **Response**: `OK` (plain text)

Use this endpoint to verify the server is running and healthy.
