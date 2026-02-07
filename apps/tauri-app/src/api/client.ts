// Tauri API client wrapper
import { invoke } from "@tauri-apps/api/core";
import type {
  AddRepositoryParams,
  CompositeTask,
  CompositeTaskNodesResult,
  CreateCompositeTaskParams,
  CreateRepositoryGroupParams,
  CreateUnitTaskParams,
  CreateWorkspaceParams,
  GlobalSettings,
  ListRepositoriesParams,
  ListRepositoriesResult,
  ListRepositoryGroupsParams,
  ListRepositoryGroupsResult,
  ListTasksParams,
  ListTasksResult,
  ListWorkspacesParams,
  ListWorkspacesResult,
  Repository,
  RepositoryGroup,
  RepositorySettings,
  RespondTtyInputParams,
  TaskLogsResponse,
  TaskResponse,
  UnitTask,
  UpdateRepositoryGroupParams,
  UpdateWorkspaceParams,
  Workspace,
} from "./types";

// Mode commands

export async function getMode(): Promise<string> {
  return invoke<string>("get_mode");
}

export async function setMode(
  mode: "local" | "remote",
  serverUrl?: string
): Promise<void> {
  return invoke<void>("set_mode", { mode, serverUrl });
}

// Task commands

export async function createUnitTask(
  params: CreateUnitTaskParams
): Promise<UnitTask> {
  return invoke<UnitTask>("create_unit_task", { params });
}

export async function createCompositeTask(
  params: CreateCompositeTaskParams
): Promise<CompositeTask> {
  return invoke<CompositeTask>("create_composite_task", { params });
}

export async function getTask(taskId: string): Promise<TaskResponse> {
  return invoke<TaskResponse>("get_task", { taskId });
}

export async function listTasks(params: ListTasksParams): Promise<ListTasksResult> {
  return invoke<ListTasksResult>("list_tasks", { params });
}

export async function approveTask(taskId: string): Promise<void> {
  return invoke<void>("approve_task", { taskId });
}

export async function rejectTask(
  taskId: string,
  reason?: string
): Promise<void> {
  return invoke<void>("reject_task", { taskId, reason });
}

export async function requestChanges(
  taskId: string,
  feedback: string
): Promise<void> {
  return invoke<void>("request_changes", { taskId, feedback });
}

export async function cancelTask(taskId: string): Promise<void> {
  return invoke<void>("cancel_task", { taskId });
}

export async function updatePlanWithPrompt(
  taskId: string,
  prompt: string
): Promise<void> {
  return invoke<void>("update_plan_with_prompt", { taskId, prompt });
}

export async function getCompositeTaskNodes(
  compositeTaskId: string
): Promise<CompositeTaskNodesResult> {
  return invoke<CompositeTaskNodesResult>("get_composite_task_nodes", {
    compositeTaskId,
  });
}

export async function getTaskLogs(
  agentTaskId: string,
  afterEventId?: number
): Promise<TaskLogsResponse> {
  return invoke<TaskLogsResponse>("get_task_logs", { agentTaskId, afterEventId });
}

export async function respondTtyInput(params: RespondTtyInputParams): Promise<void> {
  return invoke<void>("respond_tty_input", { params });
}

export async function getWorktreePath(taskId: string): Promise<string | null> {
  return invoke<string | null>("get_worktree_path", { taskId });
}

// Repository commands

export async function addRepository(
  params: AddRepositoryParams
): Promise<Repository> {
  return invoke<Repository>("add_repository", { params });
}

export async function listRepositories(
  params: ListRepositoriesParams
): Promise<ListRepositoriesResult> {
  return invoke<ListRepositoriesResult>("list_repositories", { params });
}

export async function removeRepository(repositoryId: string): Promise<void> {
  return invoke<void>("remove_repository", { repositoryId });
}

// Repository Group commands

export async function createRepositoryGroup(
  params: CreateRepositoryGroupParams
): Promise<RepositoryGroup> {
  return invoke<RepositoryGroup>("create_repository_group", { params });
}

export async function listRepositoryGroups(
  params: ListRepositoryGroupsParams = {}
): Promise<ListRepositoryGroupsResult> {
  return invoke<ListRepositoryGroupsResult>("list_repository_groups", { params });
}

export async function getRepositoryGroup(groupId: string): Promise<RepositoryGroup> {
  return invoke<RepositoryGroup>("get_repository_group", { groupId });
}

export async function updateRepositoryGroup(
  groupId: string,
  params: UpdateRepositoryGroupParams
): Promise<RepositoryGroup> {
  return invoke<RepositoryGroup>("update_repository_group", { groupId, params });
}

export async function deleteRepositoryGroup(groupId: string): Promise<void> {
  return invoke<void>("delete_repository_group", { groupId });
}

// Settings commands

export async function getGlobalSettings(): Promise<GlobalSettings> {
  return invoke<GlobalSettings>("get_global_settings");
}

export async function updateGlobalSettings(
  settings: Partial<GlobalSettings>
): Promise<GlobalSettings> {
  return invoke<GlobalSettings>("update_global_settings", { settings });
}

export async function getRepositorySettings(
  repositoryId: string
): Promise<RepositorySettings> {
  return invoke<RepositorySettings>("get_repository_settings", { repositoryId });
}

export async function updateRepositorySettings(
  repositoryId: string,
  settings: Partial<RepositorySettings>
): Promise<RepositorySettings> {
  return invoke<RepositorySettings>("update_repository_settings", {
    repositoryId,
    settings,
  });
}

// Secrets commands

export async function getSecret(key: string): Promise<string | null> {
  return invoke<string | null>("get_secret", { key });
}

export async function setSecret(key: string, value: string): Promise<void> {
  return invoke<void>("set_secret", { key, value });
}

export async function deleteSecret(key: string): Promise<void> {
  return invoke<void>("delete_secret", { key });
}

export async function listSecrets(): Promise<string[]> {
  return invoke<string[]>("list_secrets");
}

export async function sendSecrets(secrets: Record<string, string>): Promise<void> {
  return invoke<void>("send_secrets", { secrets });
}

// Workspace commands

export async function createWorkspace(
  params: CreateWorkspaceParams
): Promise<Workspace> {
  return invoke<Workspace>("create_workspace", { params });
}

export async function listWorkspaces(
  params: ListWorkspacesParams = {}
): Promise<ListWorkspacesResult> {
  return invoke<ListWorkspacesResult>("list_workspaces", { params });
}

export async function getWorkspace(workspaceId: string): Promise<Workspace> {
  return invoke<Workspace>("get_workspace", { workspaceId });
}

export async function updateWorkspace(
  workspaceId: string,
  params: UpdateWorkspaceParams
): Promise<Workspace> {
  return invoke<Workspace>("update_workspace", { workspaceId, params });
}

export async function deleteWorkspace(workspaceId: string): Promise<void> {
  return invoke<void>("delete_workspace", { workspaceId });
}

export async function getDefaultWorkspaceId(): Promise<string> {
  return invoke<string>("get_default_workspace_id");
}
