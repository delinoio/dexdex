// Types mirroring the new Rust entity definitions

export type UnitTaskStatus =
  | 'queued'
  | 'in_progress'
  | 'action_required'
  | 'blocked'
  | 'completed'
  | 'failed'
  | 'cancelled';

export type SubTaskType =
  | 'initial_implementation'
  | 'request_changes'
  | 'pr_create'
  | 'pr_review_fix'
  | 'pr_ci_fix'
  | 'manual_retry';

export type SubTaskStatus =
  | 'queued'
  | 'in_progress'
  | 'waiting_for_plan_approval'
  | 'waiting_for_user_input'
  | 'completed'
  | 'failed'
  | 'cancelled';

export type AgentSessionStatus =
  | 'starting'
  | 'running'
  | 'waiting_for_input'
  | 'completed'
  | 'failed'
  | 'cancelled';

export type SessionOutputKind =
  | 'text'
  | 'plan_update'
  | 'tool_call'
  | 'tool_result'
  | 'progress'
  | 'warning'
  | 'error';

export type ActionType =
  | 'review_requested'
  | 'pr_creation_ready'
  | 'plan_approval_required'
  | 'ci_failed'
  | 'merge_conflict'
  | 'security_alert'
  | 'user_input_required';

export type PrStatus =
  | 'open'
  | 'approved'
  | 'changes_requested'
  | 'merged'
  | 'closed'
  | 'ci_failed';

export type BadgeColorKey =
  | 'blue'
  | 'green'
  | 'yellow'
  | 'orange'
  | 'red'
  | 'gray';

export type StreamEventType =
  | 'task_updated'
  | 'subtask_updated'
  | 'session_output'
  | 'session_state_changed'
  | 'pr_updated'
  | 'review_assist_updated'
  | 'inline_comment_updated'
  | 'notification_created';

export type NotificationType =
  | 'task_action_required'
  | 'plan_action_required'
  | 'pr_review_activity'
  | 'pr_ci_failure'
  | 'agent_session_failed';

export interface GeneratedCommit {
  sha: string;
  parentShas: string[];
  title: string;
  body?: string;
  authoredAt: string;
}

export interface UnitTask {
  id: string;
  workspaceId: string;
  repositoryGroupId: string;
  title: string;
  prompt: string;
  branchName?: string;
  status: UnitTaskStatus;
  actionTypes: ActionType[];
  prTrackingIds: string[];
  latestCommitSha?: string;
  generatedCommitCount: number;
  latestPatchRef?: string;
  createdAt: string;
  updatedAt: string;
}

export interface SubTask {
  id: string;
  unitTaskId: string;
  taskType: SubTaskType;
  prompt: string;
  status: SubTaskStatus;
  planModeEnabled: boolean;
  targetActionType?: ActionType;
  baseCommitSha?: string;
  headCommitSha?: string;
  generatedCommits: GeneratedCommit[];
  createdAt: string;
  updatedAt: string;
}

export interface TokenUsageMetrics {
  provider: string;
  model: string;
  inputTokens: number;
  outputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  totalTokens: number;
  totalCostUsd: number;
}

export interface AgentSession {
  id: string;
  subTaskId: string;
  agentType: string;
  model?: string;
  status: AgentSessionStatus;
  tokenUsage?: TokenUsageMetrics;
  startedAt?: string;
  completedAt?: string;
  createdAt: string;
}

export interface SessionOutputEvent {
  id: string;
  sessionId: string;
  sequence: number;
  kind: SessionOutputKind;
  message: string;
  attributes: Record<string, string>;
  emittedAt: string;
}

export interface PullRequestTracking {
  id: string;
  unitTaskId: string;
  provider: string;
  repositoryId: string;
  prNumber: number;
  prUrl: string;
  status: PrStatus;
  lastPolledAt?: string;
  autoFixEnabled: boolean;
  maxAutoFixAttempts: number;
  autoFixAttemptsUsed: number;
  createdAt: string;
  updatedAt: string;
}

export interface ReviewAssistItem {
  id: string;
  unitTaskId: string;
  prTrackingId: string;
  sourceType: string;
  title: string;
  details?: string;
  status: string;
  createdAt: string;
  updatedAt: string;
}

export interface ReviewInlineComment {
  id: string;
  unitTaskId: string;
  subTaskId?: string;
  filePath: string;
  side: string;
  lineNumber: number;
  body: string;
  status: string;
  authorUserId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface BadgeTheme {
  id: string;
  workspaceId: string;
  actionType: ActionType;
  colorKey: BadgeColorKey;
  createdAt: string;
  updatedAt: string;
}

export interface Notification {
  id: string;
  workspaceId: string;
  notificationType: NotificationType;
  title: string;
  body: string;
  deepLink?: string;
  readAt?: string;
  createdAt: string;
}

export interface Workspace {
  id: string;
  name: string;
  endpointUrl?: string;
  authProfileId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Repository {
  id: string;
  workspaceId: string;
  remoteUrl: string;
  defaultBranch: string;
  vcsProvider: string;
  createdAt: string;
  updatedAt: string;
}

export interface RepositoryGroup {
  id: string;
  workspaceId: string;
  name: string;
  repositoryIds: string[];
  createdAt: string;
  updatedAt: string;
}

export interface StreamEvent {
  eventType: StreamEventType;
  workspaceId: string;
  payload: unknown;
}

// API Request/Response types

export interface ListTasksRequest {
  workspaceId?: string;
  repositoryGroupId?: string;
  status?: UnitTaskStatus;
  limit?: number;
  offset?: number;
}

export interface ListTasksResponse {
  tasks: UnitTask[];
  totalCount: number;
}

export interface GetTaskRequest {
  taskId: string;
}

export interface GetTaskResponse {
  task: UnitTask;
}

export interface CreateTaskRequest {
  workspaceId: string;
  repositoryGroupId: string;
  title: string;
  prompt: string;
  branchName?: string;
}

export interface CreateTaskResponse {
  task: UnitTask;
}

export interface DeleteTaskRequest {
  taskId: string;
}

export interface StopTaskRequest {
  taskId: string;
}

export interface ApproveSubTaskRequest {
  subTaskId: string;
}

export interface RequestChangesRequest {
  subTaskId: string;
  feedback: string;
}

export interface ApprovePlanRequest {
  subTaskId: string;
}

export interface RevisePlanRequest {
  subTaskId: string;
  feedback: string;
}

export interface CreatePrRequest {
  taskId: string;
}

export interface RetrySubTaskRequest {
  subTaskId: string;
}

export interface ListSubTasksRequest {
  unitTaskId: string;
}

export interface ListSubTasksResponse {
  subTasks: SubTask[];
}

export interface GetSubTaskRequest {
  subTaskId: string;
}

export interface GetSubTaskResponse {
  subTask: SubTask;
}

export interface ListAgentSessionsRequest {
  subTaskId: string;
}

export interface ListAgentSessionsResponse {
  sessions: AgentSession[];
}

export interface ListSessionOutputsRequest {
  sessionId: string;
  afterSequence?: number;
}

export interface ListSessionOutputsResponse {
  events: SessionOutputEvent[];
}

export interface ListNotificationsRequest {
  workspaceId: string;
  unreadOnly?: boolean;
  limit?: number;
  offset?: number;
}

export interface ListNotificationsResponse {
  notifications: Notification[];
  totalCount: number;
}

export interface MarkNotificationReadRequest {
  notificationId: string;
}

export interface MarkAllNotificationsReadRequest {
  workspaceId: string;
}

export interface ListWorkspacesRequest {
  limit?: number;
  offset?: number;
}

export interface ListWorkspacesResponse {
  workspaces: Workspace[];
  totalCount: number;
}

export interface GetWorkspaceRequest {
  workspaceId: string;
}

export interface GetWorkspaceResponse {
  workspace: Workspace;
}

export interface CreateWorkspaceRequest {
  name: string;
  endpointUrl?: string;
}

export interface UpdateWorkspaceRequest {
  workspaceId: string;
  name?: string;
  endpointUrl?: string;
}

export interface DeleteWorkspaceRequest {
  workspaceId: string;
}

export interface ListRepositoriesRequest {
  workspaceId?: string;
  limit?: number;
  offset?: number;
}

export interface ListRepositoriesResponse {
  repositories: Repository[];
  totalCount: number;
}

export interface ListRepositoryGroupsRequest {
  workspaceId?: string;
  limit?: number;
  offset?: number;
}

export interface ListRepositoryGroupsResponse {
  groups: RepositoryGroup[];
  totalCount: number;
}

export interface ListPrTrackingsRequest {
  unitTaskId: string;
}

export interface ListPrTrackingsResponse {
  prTrackings: PullRequestTracking[];
}
