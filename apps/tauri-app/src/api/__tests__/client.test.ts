import { describe, it, expect, vi, beforeEach } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import {
  getMode,
  setMode,
  createUnitTask,
  createCompositeTask,
  getTask,
  listTasks,
  approveTask,
  rejectTask,
  requestChanges,
  addRepository,
  listRepositories,
  removeRepository,
  getGlobalSettings,
  updateGlobalSettings,
  getRepositorySettings,
  updateRepositorySettings,
  getSecret,
  setSecret,
  deleteSecret,
  listSecrets,
  sendSecrets,
  createWorkspace,
  listWorkspaces,
  getWorkspace,
  updateWorkspace,
  deleteWorkspace,
  getDefaultWorkspaceId,
} from "../client";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

describe("API Client", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Mode commands", () => {
    it("getMode calls invoke with correct command", async () => {
      vi.mocked(invoke).mockResolvedValue("local");

      const result = await getMode();

      expect(invoke).toHaveBeenCalledWith("get_mode");
      expect(result).toBe("local");
    });

    it("setMode calls invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await setMode("remote", "https://example.com");

      expect(invoke).toHaveBeenCalledWith("set_mode", {
        mode: "remote",
        serverUrl: "https://example.com",
      });
    });

    it("setMode works without serverUrl", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await setMode("local");

      expect(invoke).toHaveBeenCalledWith("set_mode", {
        mode: "local",
        serverUrl: undefined,
      });
    });
  });

  describe("Task commands", () => {
    it("createUnitTask calls invoke with correct parameters", async () => {
      const params = { repositoryGroupId: "repo-1", prompt: "Test" };
      const mockTask = { id: "task-1", ...params };
      vi.mocked(invoke).mockResolvedValue(mockTask);

      const result = await createUnitTask(params);

      expect(invoke).toHaveBeenCalledWith("create_unit_task", { params });
      expect(result).toEqual(mockTask);
    });

    it("createCompositeTask calls invoke with correct parameters", async () => {
      const params = { repositoryGroupId: "repo-1", prompt: "Test composite" };
      const mockTask = { id: "composite-1", ...params };
      vi.mocked(invoke).mockResolvedValue(mockTask);

      const result = await createCompositeTask(params);

      expect(invoke).toHaveBeenCalledWith("create_composite_task", { params });
      expect(result).toEqual(mockTask);
    });

    it("getTask calls invoke with correct taskId", async () => {
      const mockResponse = { unitTask: { id: "task-1" } };
      vi.mocked(invoke).mockResolvedValue(mockResponse);

      const result = await getTask("task-1");

      expect(invoke).toHaveBeenCalledWith("get_task", { taskId: "task-1" });
      expect(result).toEqual(mockResponse);
    });

    it("listTasks calls invoke with correct parameters", async () => {
      const params = { limit: 10, offset: 0 };
      const mockResult = { unitTasks: [], compositeTasks: [], totalCount: 0 };
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await listTasks(params);

      expect(invoke).toHaveBeenCalledWith("list_tasks", { params });
      expect(result).toEqual(mockResult);
    });

    it("approveTask calls invoke with correct taskId", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await approveTask("task-1");

      expect(invoke).toHaveBeenCalledWith("approve_task", { taskId: "task-1" });
    });

    it("rejectTask calls invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await rejectTask("task-1", "Not approved");

      expect(invoke).toHaveBeenCalledWith("reject_task", {
        taskId: "task-1",
        reason: "Not approved",
      });
    });

    it("requestChanges calls invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await requestChanges("task-1", "Please fix this");

      expect(invoke).toHaveBeenCalledWith("request_changes", {
        taskId: "task-1",
        feedback: "Please fix this",
      });
    });
  });

  describe("Repository commands", () => {
    it("addRepository calls invoke with correct parameters", async () => {
      const params = { remoteUrl: "https://github.com/test/repo" };
      const mockRepo = { id: "repo-1", ...params };
      vi.mocked(invoke).mockResolvedValue(mockRepo);

      const result = await addRepository(params);

      expect(invoke).toHaveBeenCalledWith("add_repository", { params });
      expect(result).toEqual(mockRepo);
    });

    it("listRepositories calls invoke with correct parameters", async () => {
      const params = { workspaceId: "ws-1" };
      const mockResult = { repositories: [], totalCount: 0 };
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await listRepositories(params);

      expect(invoke).toHaveBeenCalledWith("list_repositories", { params });
      expect(result).toEqual(mockResult);
    });

    it("removeRepository calls invoke with correct repositoryId", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await removeRepository("repo-1");

      expect(invoke).toHaveBeenCalledWith("remove_repository", { repositoryId: "repo-1" });
    });
  });

  describe("Settings commands", () => {
    it("getGlobalSettings calls invoke correctly", async () => {
      const mockSettings = { mode: "local" };
      vi.mocked(invoke).mockResolvedValue(mockSettings);

      const result = await getGlobalSettings();

      expect(invoke).toHaveBeenCalledWith("get_global_settings");
      expect(result).toEqual(mockSettings);
    });

    it("updateGlobalSettings calls invoke with correct parameters", async () => {
      const settings = { mode: "remote" as const };
      const mockResult = { ...settings };
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await updateGlobalSettings(settings);

      expect(invoke).toHaveBeenCalledWith("update_global_settings", { settings });
      expect(result).toEqual(mockResult);
    });

    it("getRepositorySettings calls invoke with correct repositoryId", async () => {
      const mockSettings = { branch: { template: "feature/{task}" } };
      vi.mocked(invoke).mockResolvedValue(mockSettings);

      const result = await getRepositorySettings("repo-1");

      expect(invoke).toHaveBeenCalledWith("get_repository_settings", { repositoryId: "repo-1" });
      expect(result).toEqual(mockSettings);
    });

    it("updateRepositorySettings calls invoke with correct parameters", async () => {
      const settings = { automation: { autoFixReviewComments: true } };
      vi.mocked(invoke).mockResolvedValue(settings);

      const result = await updateRepositorySettings("repo-1", settings);

      expect(invoke).toHaveBeenCalledWith("update_repository_settings", {
        repositoryId: "repo-1",
        settings,
      });
      expect(result).toEqual(settings);
    });
  });

  describe("Secrets commands", () => {
    it("getSecret calls invoke with correct key", async () => {
      vi.mocked(invoke).mockResolvedValue("secret-value");

      const result = await getSecret("API_KEY");

      expect(invoke).toHaveBeenCalledWith("get_secret", { key: "API_KEY" });
      expect(result).toBe("secret-value");
    });

    it("setSecret calls invoke with correct parameters", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await setSecret("API_KEY", "secret-value");

      expect(invoke).toHaveBeenCalledWith("set_secret", {
        key: "API_KEY",
        value: "secret-value",
      });
    });

    it("deleteSecret calls invoke with correct key", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await deleteSecret("API_KEY");

      expect(invoke).toHaveBeenCalledWith("delete_secret", { key: "API_KEY" });
    });

    it("listSecrets calls invoke correctly", async () => {
      vi.mocked(invoke).mockResolvedValue(["KEY1", "KEY2"]);

      const result = await listSecrets();

      expect(invoke).toHaveBeenCalledWith("list_secrets");
      expect(result).toEqual(["KEY1", "KEY2"]);
    });

    it("sendSecrets calls invoke with correct parameters", async () => {
      const secrets = { KEY1: "value1", KEY2: "value2" };
      vi.mocked(invoke).mockResolvedValue(undefined);

      await sendSecrets(secrets);

      expect(invoke).toHaveBeenCalledWith("send_secrets", { secrets });
    });
  });

  describe("Workspace commands", () => {
    it("createWorkspace calls invoke with correct parameters", async () => {
      const params = { name: "Test Workspace" };
      const mockWorkspace = { id: "ws-1", ...params };
      vi.mocked(invoke).mockResolvedValue(mockWorkspace);

      const result = await createWorkspace(params);

      expect(invoke).toHaveBeenCalledWith("create_workspace", { params });
      expect(result).toEqual(mockWorkspace);
    });

    it("listWorkspaces calls invoke with correct parameters", async () => {
      const params = { limit: 10 };
      const mockResult = { workspaces: [], totalCount: 0 };
      vi.mocked(invoke).mockResolvedValue(mockResult);

      const result = await listWorkspaces(params);

      expect(invoke).toHaveBeenCalledWith("list_workspaces", { params });
      expect(result).toEqual(mockResult);
    });

    it("listWorkspaces uses empty object by default", async () => {
      const mockResult = { workspaces: [], totalCount: 0 };
      vi.mocked(invoke).mockResolvedValue(mockResult);

      await listWorkspaces();

      expect(invoke).toHaveBeenCalledWith("list_workspaces", { params: {} });
    });

    it("getWorkspace calls invoke with correct workspaceId", async () => {
      const mockWorkspace = { id: "ws-1", name: "Test" };
      vi.mocked(invoke).mockResolvedValue(mockWorkspace);

      const result = await getWorkspace("ws-1");

      expect(invoke).toHaveBeenCalledWith("get_workspace", { workspaceId: "ws-1" });
      expect(result).toEqual(mockWorkspace);
    });

    it("updateWorkspace calls invoke with correct parameters", async () => {
      const params = { name: "Updated Name" };
      const mockWorkspace = { id: "ws-1", ...params };
      vi.mocked(invoke).mockResolvedValue(mockWorkspace);

      const result = await updateWorkspace("ws-1", params);

      expect(invoke).toHaveBeenCalledWith("update_workspace", {
        workspaceId: "ws-1",
        params,
      });
      expect(result).toEqual(mockWorkspace);
    });

    it("deleteWorkspace calls invoke with correct workspaceId", async () => {
      vi.mocked(invoke).mockResolvedValue(undefined);

      await deleteWorkspace("ws-1");

      expect(invoke).toHaveBeenCalledWith("delete_workspace", { workspaceId: "ws-1" });
    });

    it("getDefaultWorkspaceId calls invoke correctly", async () => {
      vi.mocked(invoke).mockResolvedValue("default-ws-id");

      const result = await getDefaultWorkspaceId();

      expect(invoke).toHaveBeenCalledWith("get_default_workspace_id");
      expect(result).toBe("default-ws-id");
    });
  });
});
