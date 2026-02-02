/**
 * API Mock utilities for testing.
 *
 * These utilities help create mock API responses for unit and integration tests.
 */

import type {
  UnitTask,
  CompositeTask,
  Workspace,
  Repository,
  RepositoryGroup,
  TodoItem,
} from '../../api/types';

/**
 * Creates a mock UnitTask with default values.
 */
export function createMockUnitTask(overrides?: Partial<UnitTask>): UnitTask {
  const now = new Date().toISOString();
  return {
    id: crypto.randomUUID(),
    repository_group_id: crypto.randomUUID(),
    agent_task_id: crypto.randomUUID(),
    prompt: 'Fix the bug in the login flow',
    title: 'Bug Fix',
    branch_name: 'fix/login-bug',
    linked_pr_url: null,
    base_commit: null,
    end_commit: null,
    auto_fix_task_ids: [],
    status: 'in_progress',
    created_at: now,
    updated_at: now,
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
    repository_group_id: crypto.randomUUID(),
    planning_task_id: crypto.randomUUID(),
    prompt: 'Implement feature X',
    title: 'Feature X',
    node_ids: [],
    status: 'planning',
    execution_agent_type: 'claude_code',
    created_at: now,
    updated_at: now,
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
    user_id: null,
    created_at: now,
    updated_at: now,
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
    workspace_id: crypto.randomUUID(),
    name: 'my-repo',
    remote_url: 'https://github.com/user/my-repo.git',
    default_branch: 'main',
    vcs_type: 'git',
    vcs_provider_type: 'github',
    created_at: now,
    updated_at: now,
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
    workspace_id: crypto.randomUUID(),
    name: 'My Group',
    repository_ids: [],
    created_at: now,
    updated_at: now,
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
    item_type: 'issue_triage',
    source: 'github',
    status: 'pending',
    repository_id: crypto.randomUUID(),
    data: {
      issue_url: 'https://github.com/user/repo/issues/1',
      issue_title: 'Test Issue',
    },
    created_at: now,
    updated_at: now,
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
    unit_tasks: [createMockUnitTask()],
    composite_tasks: [createMockCompositeTask()],
    total_count: 2,
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
  list_todo_items: { items: [createMockTodoItem()], total_count: 1 },
  get_todo_item: { item: createMockTodoItem() },
  update_todo_status: { item: createMockTodoItem() },
  dismiss_todo: {},

  // Settings commands
  get_global_settings: {
    settings: {
      learning: { auto_learn_from_reviews: false },
      hotkey: { open_chat: 'Option+Z' },
      notification: { enabled: true },
    },
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
