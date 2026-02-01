// Tauri API client wrapper
import { invoke } from "@tauri-apps/api/core";
import type {
  AddRepositoryParams,
  CompositeTask,
  CreateCompositeTaskParams,
  CreateUnitTaskParams,
  GlobalSettings,
  ListRepositoriesParams,
  ListRepositoriesResult,
  ListTasksParams,
  ListTasksResult,
  Repository,
  RepositorySettings,
  TaskResponse,
  UnitTask,
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
