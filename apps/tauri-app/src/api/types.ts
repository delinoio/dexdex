// Types that mirror the Rust entity definitions

// Enums

export enum VcsType {
  Unspecified = "unspecified",
  Git = "git",
}

export enum VcsProviderType {
  Unspecified = "unspecified",
  Github = "github",
  Gitlab = "gitlab",
  Bitbucket = "bitbucket",
}

export enum AiAgentType {
  Unspecified = "unspecified",
  ClaudeCode = "claude_code",
  OpenCode = "open_code",
  GeminiCli = "gemini_cli",
  CodexCli = "codex_cli",
  Aider = "aider",
  Amp = "amp",
}

export enum UnitTaskStatus {
  Unspecified = "unspecified",
  InProgress = "in_progress",
  InReview = "in_review",
  Approved = "approved",
  PrOpen = "pr_open",
  Done = "done",
  Rejected = "rejected",
}

export enum CompositeTaskStatus {
  Unspecified = "unspecified",
  Planning = "planning",
  PendingApproval = "pending_approval",
  InProgress = "in_progress",
  Done = "done",
  Rejected = "rejected",
}

export enum TtyInputType {
  Unspecified = "unspecified",
  Text = "text",
  Select = "select",
  Confirm = "confirm",
  Password = "password",
}

export enum TtyInputStatus {
  Unspecified = "unspecified",
  Pending = "pending",
  Responded = "responded",
  Timeout = "timeout",
  Cancelled = "cancelled",
}

export enum TodoItemType {
  Unspecified = "unspecified",
  IssueTriage = "issue_triage",
  PrReview = "pr_review",
}

export enum TodoItemStatus {
  Unspecified = "unspecified",
  Pending = "pending",
  InProgress = "in_progress",
  Completed = "completed",
  Dismissed = "dismissed",
}

// Entities

export interface BaseRemote {
  gitRemoteUrl: string;
  gitBranchName: string;
}

export interface AgentSession {
  id: string;
  agentTaskId: string;
  aiAgentType: AiAgentType;
  aiAgentModel?: string;
  startedAt?: string;
  completedAt?: string;
  outputLog?: string;
  createdAt: string;
}

export interface AgentTask {
  id: string;
  baseRemotes: BaseRemote[];
  agentSessions: AgentSession[];
  aiAgentType?: AiAgentType;
  aiAgentModel?: string;
  createdAt: string;
}

export interface UnitTask {
  id: string;
  repositoryGroupId: string;
  agentTaskId: string;
  prompt: string;
  title?: string;
  branchName?: string;
  linkedPrUrl?: string;
  baseCommit?: string;
  endCommit?: string;
  autoFixTaskIds: string[];
  status: UnitTaskStatus;
  createdAt: string;
  updatedAt: string;
}

export interface CompositeTaskNode {
  id: string;
  compositeTaskId: string;
  unitTaskId: string;
  dependsOnIds: string[];
  createdAt: string;
}

export interface CompositeTaskNodeWithUnitTask {
  node: CompositeTaskNode;
  unitTask: UnitTask;
}

export interface CompositeTaskNodesResult {
  nodes: CompositeTaskNodeWithUnitTask[];
}

export interface CompositeTask {
  id: string;
  repositoryGroupId: string;
  planningTaskId: string;
  prompt: string;
  title?: string;
  nodeIds: string[];
  status: CompositeTaskStatus;
  executionAgentType?: AiAgentType;
  createdAt: string;
  updatedAt: string;
}

export interface Repository {
  id: string;
  workspaceId: string;
  name: string;
  remoteUrl: string;
  defaultBranch: string;
  vcsType: VcsType;
  vcsProviderType: VcsProviderType;
  createdAt: string;
  updatedAt: string;
}

export interface RepositoryGroup {
  id: string;
  workspaceId: string;
  name?: string;
  repositoryIds: string[];
  createdAt: string;
  updatedAt: string;
}

export interface Workspace {
  id: string;
  name: string;
  description?: string;
  userId?: string;
  createdAt: string;
  updatedAt: string;
}

export interface User {
  id: string;
  email: string;
  name?: string;
  createdAt: string;
  updatedAt: string;
}

export interface TtyInputRequest {
  id: string;
  taskId: string;
  sessionId: string;
  prompt: string;
  inputType: TtyInputType;
  options: string[];
  status: TtyInputStatus;
  response?: string;
  createdAt: string;
  respondedAt?: string;
}

export interface IssueTriageData {
  issueUrl: string;
  issueTitle: string;
  suggestedLabels: string[];
  suggestedAssignees: string[];
}

export interface PrReviewData {
  prUrl: string;
  prTitle: string;
  changedFilesCount: number;
  aiSummary?: string;
}

export interface TodoItem {
  id: string;
  itemType: TodoItemType;
  status: TodoItemStatus;
  repositoryId: string;
  issueTriage?: IssueTriageData;
  prReview?: PrReviewData;
  createdAt: string;
  updatedAt: string;
}

// App settings types

export interface GlobalSettings {
  mode: "local" | "remote";
  serverUrl?: string;
  hotkey: {
    openChat: string;
  };
  notification: {
    enabled: boolean;
    approvalRequest: boolean;
    userQuestion: boolean;
    reviewReady: boolean;
  };
  agent: {
    planning: {
      type: AiAgentType;
      model: string;
    };
    execution: {
      type: AiAgentType;
      model: string;
    };
    chat: {
      type: AiAgentType;
      model: string;
    };
  };
}

export interface RepositorySettings {
  branch: {
    template: string;
  };
  automation: {
    autoFixReviewComments: boolean;
    autoFixReviewCommentsFilter: string;
    autoFixCIFailures: boolean;
    maxAutoFixAttempts: number;
  };
  compositeTask: {
    autoApprove: boolean;
  };
}

// API Request/Response types

export interface CreateUnitTaskParams {
  repositoryGroupId: string;
  prompt: string;
  title?: string;
  branchName?: string;
  aiAgentType?: string;
  aiAgentModel?: string;
}

export interface CreateCompositeTaskParams {
  repositoryGroupId: string;
  prompt: string;
  title?: string;
  executionAgentType?: string;
  planningAgentType?: string;
}

export interface ListTasksParams {
  repositoryGroupId?: string;
  unitStatus?: string;
  compositeStatus?: string;
  limit?: number;
  offset?: number;
}

export interface ListTasksResult {
  unitTasks: UnitTask[];
  compositeTasks: CompositeTask[];
  totalCount: number;
}

export interface TaskResponse {
  unitTask?: UnitTask;
  compositeTask?: CompositeTask;
}

export interface AddRepositoryParams {
  workspaceId?: string;
  remoteUrl: string;
  name?: string;
  defaultBranch?: string;
}

export interface ListRepositoriesParams {
  workspaceId?: string;
  limit?: number;
  offset?: number;
}

export interface ListRepositoriesResult {
  repositories: Repository[];
  totalCount: number;
}

export interface CreateWorkspaceParams {
  name: string;
  description?: string;
}

export interface UpdateWorkspaceParams {
  name?: string;
  description?: string;
}

export interface ListWorkspacesParams {
  limit?: number;
  offset?: number;
}

export interface ListWorkspacesResult {
  workspaces: Workspace[];
  totalCount: number;
}

// Repository Group Request/Response types

export interface CreateRepositoryGroupParams {
  workspaceId?: string;
  name?: string;
  repositoryIds: string[];
}

export interface ListRepositoryGroupsParams {
  workspaceId?: string;
  limit?: number;
  offset?: number;
}

export interface ListRepositoryGroupsResult {
  groups: RepositoryGroup[];
  totalCount: number;
}

export interface UpdateRepositoryGroupParams {
  name?: string;
  repositoryIds: string[];
}
