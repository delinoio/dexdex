/**
 * API Mock utilities for testing.
 *
 * These utilities help create mock API responses for unit and integration tests.
 */

import { vi } from 'vitest';
import type {
  UnitTask,
  CompositeTask,
  Workspace,
  Repository,
  RepositoryGroup,
  TodoItem,
  GlobalSettings,
  UnitTaskStatus,
  CompositeTaskStatus,
  VcsType,
  VcsProviderType,
  TodoItemType,
  TodoItemStatus,
  AiAgentType,
} from '../../api/types';

/**
 * Creates a mock UnitTask with default values.
 */
export function createMockUnitTask(overrides?: Partial<UnitTask>): UnitTask {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    repositoryGroupId: crypto.randomUUID(),
    agentTaskId: crypto.randomUUID(),
    prompt: 'Fix the bug in the login flow',
    title: 'Bug Fix',
    branchName: 'fix/login-bug',
    linkedPrUrl: undefined,
    baseCommit: undefined,
    endCommit: undefined,
    autoFixTaskIds: [],
    status: 'in_progress' as UnitTaskStatus,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock CompositeTask with default values.
 */
export function createMockCompositeTask(overrides?: Partial<CompositeTask>): CompositeTask {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    repositoryGroupId: crypto.randomUUID(),
    planningTaskId: crypto.randomUUID(),
    prompt: 'Implement feature X',
    title: 'Feature X',
    nodeIds: [],
    status: 'planning' as CompositeTaskStatus,
    executionAgentType: 'claude_code' as AiAgentType,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock Workspace with default values.
 */
export function createMockWorkspace(overrides?: Partial<Workspace>): Workspace {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    name: 'My Workspace',
    description: 'A test workspace',
    kind: 'local' as const,
    userId: undefined,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock Repository with default values.
 */
export function createMockRepository(overrides?: Partial<Repository>): Repository {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    workspaceId: crypto.randomUUID(),
    name: 'my-repo',
    remoteUrl: 'https://github.com/user/my-repo.git',
    defaultBranch: 'main',
    vcsType: 'git' as VcsType,
    vcsProviderType: 'github' as VcsProviderType,
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock RepositoryGroup with default values.
 */
export function createMockRepositoryGroup(overrides?: Partial<RepositoryGroup>): RepositoryGroup {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    workspaceId: crypto.randomUUID(),
    name: 'My Group',
    repositoryIds: [],
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock TodoItem with default values.
 */
export function createMockTodoItem(overrides?: Partial<TodoItem>): TodoItem {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    itemType: 'issue_triage' as TodoItemType,
    status: 'pending' as TodoItemStatus,
    repositoryId: crypto.randomUUID(),
    issueTriage: {
      issueUrl: 'https://github.com/user/repo/issues/1',
      issueTitle: 'Test Issue',
      suggestedLabels: [],
      suggestedAssignees: [],
    },
    createdAt: now,
    updatedAt: now,
    ...overrides,
  };
}

/**
 * Creates a mock GlobalSettings object with default values.
 */
export function createMockGlobalSettings(overrides?: Partial<GlobalSettings>): GlobalSettings {
  return {
    hotkey: 'Option+Z',
    notificationsEnabled: true,
    defaultAgentType: 'claude_code',
    defaultAgentModel: 'claude-sonnet-4-20250514',
    ...overrides,
  };
}

/**
 * Mock Tauri invoke responses.
 *
 * This can be used with vi.mock to mock @tauri-apps/api/core.
 */
export const mockTauriResponses = {
  // Task commands
  list_tasks: {
    unitTasks: [createMockUnitTask()],
    compositeTasks: [createMockCompositeTask()],
    totalCount: 2,
  },
  get_task: createMockUnitTask(),
  create_unit_task: { task: createMockUnitTask() },
  create_composite_task: { task: createMockCompositeTask() },
  approve_task: {},
  reject_task: {},
  request_changes: { task: createMockUnitTask() },

  // Workspace commands
  list_workspaces: { workspaces: [createMockWorkspace()] },
  get_workspace: { workspace: createMockWorkspace() },
  create_workspace: { workspace: createMockWorkspace() },
  update_workspace: { workspace: createMockWorkspace() },
  delete_workspace: {},

  // Repository commands
  list_repositories: { repositories: [createMockRepository()] },
  get_repository: { repository: createMockRepository() },
  add_repository: { repository: createMockRepository() },
  remove_repository: {},

  // Repository group commands
  list_repository_groups: { groups: [createMockRepositoryGroup()] },
  create_repository_group: { group: createMockRepositoryGroup() },
  update_repository_group: { group: createMockRepositoryGroup() },
  delete_repository_group: {},

  // Todo commands
  list_todo_items: { items: [createMockTodoItem()], totalCount: 1 },
  get_todo_item: { item: createMockTodoItem() },
  update_todo_status: { item: createMockTodoItem() },
  dismiss_todo: {},

  // Settings commands
  get_global_settings: {
    settings: createMockGlobalSettings(),
  },
  update_global_settings: { settings: {} },
  get_repository_settings: { settings: {} },
  update_repository_settings: { settings: {} },

  // Mode commands
  get_mode: { mode: 'local' },
  set_mode: {},
};

/**
 * Creates a mock invoke function for testing.
 */
export function createMockInvoke() {
  return vi.fn().mockImplementation((command: string, _args?: unknown) => {
    if (command in mockTauriResponses) {
      return Promise.resolve(mockTauriResponses[command as keyof typeof mockTauriResponses]);
    }
    return Promise.reject(new Error(`Unknown command: ${command}`));
  });
}

/**
 * Helper to create error responses for testing error handling.
 */
export function createErrorResponse(message: string): Error {
  return new Error(message);
}

/**
 * Helper to create a rejected promise for error testing.
 */
export function createRejectedPromise<T = never>(message: string): Promise<T> {
  return Promise.reject(createErrorResponse(message));
}
