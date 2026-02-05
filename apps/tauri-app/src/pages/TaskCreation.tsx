import { useState, useCallback, useEffect, useRef, type KeyboardEvent } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Textarea } from "@/components/ui/Textarea";
import { useRepositoryGroups, useCreateRepositoryGroup } from "@/hooks/useRepositoryGroups";
import { useRepositories } from "@/hooks/useRepositories";
import { useCreateUnitTask, useCreateCompositeTask } from "@/hooks/useTasks";
import { useTaskCreationStore } from "@/stores";
import { AiAgentType, RepositoryGroup, Repository } from "@/api/types";

// Prefix to identify individual repository selections (treated as implicit groups)
const REPO_PREFIX = "__repo__";

export function TaskCreation() {
  // Get persisted preferences from store
  const {
    lastSelection,
    lastAgentType,
    lastPlanningAgentType,
    lastExecutionAgentType,
    lastIsComposite,
    setLastSelection,
    setLastAgentType,
    setLastPlanningAgentType,
    setLastExecutionAgentType,
    setLastIsComposite,
  } = useTaskCreationStore();

  // Selection can be either a group ID or a repository ID prefixed with REPO_PREFIX
  const [selection, setSelection] = useState("");
  const [prompt, setPrompt] = useState("");
  const [title, setTitle] = useState("");
  const [branchName, setBranchName] = useState("");
  const [agentType, setAgentType] = useState<AiAgentType>(lastAgentType);
  const [planningAgentType, setPlanningAgentType] = useState<AiAgentType>(lastPlanningAgentType);
  const [executionAgentType, setExecutionAgentType] = useState<AiAgentType>(lastExecutionAgentType);
  const [isComposite, setIsComposite] = useState(lastIsComposite);
  const navigate = useNavigate();
  const promptRef = useRef<HTMLTextAreaElement>(null);

  const { data: groupsData } = useRepositoryGroups({});
  const { data: repositoriesData } = useRepositories({});
  const createUnitTask = useCreateUnitTask();
  const createCompositeTask = useCreateCompositeTask();
  const createRepositoryGroup = useCreateRepositoryGroup();

  const groups = groupsData?.groups ?? [];
  const repositories = repositoriesData?.repositories ?? [];

  // Restore last selection when data is loaded
  useEffect(() => {
    if (lastSelection && !selection) {
      // Check if the last selection is still valid
      const isRepoSelection = lastSelection.startsWith(REPO_PREFIX);
      if (isRepoSelection) {
        const repoId = lastSelection.slice(REPO_PREFIX.length);
        const repoExists = repositories.some((r) => r.id === repoId);
        if (repoExists) {
          setSelection(lastSelection);
        }
      } else {
        const groupExists = groups.some((g) => g.id === lastSelection);
        if (groupExists) {
          setSelection(lastSelection);
        }
      }
    }
  }, [lastSelection, selection, repositories, groups]);

  // Auto-focus prompt textarea when dialog opens
  useEffect(() => {
    promptRef.current?.focus();
  }, []);

  // Helper function to get display name for a group (name or list of repo names)
  const getGroupDisplayName = useCallback(
    (group: RepositoryGroup, repos: Repository[]): string => {
      if (group.name) {
        return group.name;
      }
      const repoIds = group.repositoryIds ?? [];
      const repoNames = repoIds
        .map((id) => repos.find((r) => r.id === id)?.name)
        .filter((name): name is string => name !== undefined);
      if (repoNames.length > 0) {
        return repoNames.join(", ");
      }
      return "Unnamed Group";
    },
    []
  );

  // Check if an individual repository is selected (vs an existing group)
  const isRepositorySelected = selection.startsWith(REPO_PREFIX);
  const selectedRepositoryId = isRepositorySelected ? selection.slice(REPO_PREFIX.length) : null;
  const selectedRepository = selectedRepositoryId
    ? repositories.find((r) => r.id === selectedRepositoryId)
    : null;

  const selectedGroup = !isRepositorySelected ? groups.find((g) => g.id === selection) : null;
  const groupRepositories = isRepositorySelected && selectedRepository
    ? [selectedRepository]
    : selectedGroup
      ? repositories.filter((repo) => (selectedGroup.repositoryIds ?? []).includes(repo.id))
      : [];

  // Handlers that update local state and persist to store
  const handleSelectionChange = useCallback(
    (value: string) => {
      setSelection(value);
      if (value) {
        setLastSelection(value);
      }
    },
    [setLastSelection]
  );

  const handleAgentTypeChange = useCallback(
    (value: AiAgentType) => {
      setAgentType(value);
      setLastAgentType(value);
    },
    [setLastAgentType]
  );

  const handlePlanningAgentTypeChange = useCallback(
    (value: AiAgentType) => {
      setPlanningAgentType(value);
      setLastPlanningAgentType(value);
    },
    [setLastPlanningAgentType]
  );

  const handleExecutionAgentTypeChange = useCallback(
    (value: AiAgentType) => {
      setExecutionAgentType(value);
      setLastExecutionAgentType(value);
    },
    [setLastExecutionAgentType]
  );

  const handleCompositeChange = useCallback(
    (value: boolean) => {
      setIsComposite(value);
      setLastIsComposite(value);
    },
    [setLastIsComposite]
  );

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!selection || !prompt) return;

    try {
      let effectiveGroupId = selection;

      // If an individual repository is selected, find or create a repository group for it
      if (isRepositorySelected && selectedRepositoryId) {
        // Check if an existing unnamed single-repo group already contains this repository
        const existingGroup = groups.find(
          (g) =>
            !g.name &&
            g.repositoryIds.length === 1 &&
            g.repositoryIds[0] === selectedRepositoryId
        );

        if (existingGroup) {
          effectiveGroupId = existingGroup.id;
        } else {
          const newGroup = await createRepositoryGroup.mutateAsync({
            repositoryIds: [selectedRepositoryId],
            // No name - it will show the repository name as title
          });
          effectiveGroupId = newGroup.id;
        }
      }

      if (isComposite) {
        const task = await createCompositeTask.mutateAsync({
          repositoryGroupId: effectiveGroupId,
          prompt,
          title: title || undefined,
          planningAgentType: planningAgentType,
          executionAgentType: executionAgentType,
        });
        navigate(`/composite-tasks/${task.id}`);
      } else {
        const task = await createUnitTask.mutateAsync({
          repositoryGroupId: effectiveGroupId,
          prompt,
          title: title || undefined,
          branchName: branchName || undefined,
          aiAgentType: agentType,
        });
        navigate(`/unit-tasks/${task.id}`);
      }
    } catch (error) {
      console.error("Failed to create task:", error);
    }
  };

  const isPending = createUnitTask.isPending || createCompositeTask.isPending || createRepositoryGroup.isPending;

  // Handle Cmd+Enter (macOS) / Ctrl+Enter (Windows/Linux) to submit the form
  const handleFormKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
        e.preventDefault();
        if (selection && prompt && !isPending) {
          const formEvent = new Event("submit", { bubbles: true, cancelable: true }) as unknown as React.FormEvent;
          handleSubmit(formEvent);
        }
      }
    },
    [selection, prompt, isPending, handleSubmit]
  );

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <h1 className="text-2xl font-bold">Create Task</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <form onSubmit={handleSubmit} onKeyDown={handleFormKeyDown} className="mx-auto max-w-2xl space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Repository</CardTitle>
              <CardDescription>
                Select a repository or repository group for this task.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <Select
                value={selection}
                onChange={(e) => handleSelectionChange(e.target.value)}
                required
              >
                <option value="">Select a repository...</option>
                {repositories.length > 0 && (
                  <optgroup label="Repositories">
                    {repositories.map((repo) => (
                      <option key={repo.id} value={`${REPO_PREFIX}${repo.id}`}>
                        {repo.name}
                      </option>
                    ))}
                  </optgroup>
                )}
                {groups.length > 0 && (
                  <optgroup label="Repository Groups">
                    {groups.map((group) => (
                      <option key={group.id} value={group.id}>
                        {getGroupDisplayName(group, repositories)} ({group.repositoryIds?.length ?? 0}{" "}
                        {(group.repositoryIds?.length ?? 0) === 1 ? "repo" : "repos"})
                      </option>
                    ))}
                  </optgroup>
                )}
              </Select>

              {(selectedGroup || selectedRepository) && groupRepositories.length > 0 && (
                <div className="rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--muted))] p-3">
                  <p className="mb-2 text-xs font-medium text-[hsl(var(--muted-foreground))]">
                    Repositories in this group:
                  </p>
                  <div className="flex flex-wrap gap-2">
                    {groupRepositories.map((repo) => (
                      <div
                        key={repo.id}
                        className="flex items-center gap-1.5 rounded-md bg-[hsl(var(--background))] px-2 py-1 text-xs"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="12"
                          height="12"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="text-[hsl(var(--muted-foreground))]"
                        >
                          <path d="M4 20h16a2 2 0 0 0 2-2V8a2 2 0 0 0-2-2h-7.93a2 2 0 0 1-1.66-.9l-.82-1.2A2 2 0 0 0 7.93 3H4a2 2 0 0 0-2 2v13c0 1.1.9 2 2 2Z" />
                        </svg>
                        <span>{repo.name}</span>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {repositories.length === 0 && (
                <div className="text-center">
                  <p className="text-sm text-[hsl(var(--muted-foreground))]">
                    No repositories available. Add repositories first to create tasks.
                  </p>
                  <Link
                    to="/repositories"
                    className="mt-1 inline-flex items-center text-sm text-[hsl(var(--primary))] hover:underline"
                  >
                    <svg
                      xmlns="http://www.w3.org/2000/svg"
                      width="14"
                      height="14"
                      viewBox="0 0 24 24"
                      fill="none"
                      stroke="currentColor"
                      strokeWidth="2"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                      className="mr-1"
                    >
                      <path d="M5 12h14" />
                      <path d="M12 5v14" />
                    </svg>
                    Add repositories
                  </Link>
                </div>
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Task Details</CardTitle>
              <CardDescription>
                Describe what you want the AI agent to do.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <label className="text-sm font-medium">Prompt</label>
                <Textarea
                  ref={promptRef}
                  placeholder="Add user authentication to the app..."
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  rows={6}
                  required
                />
                <p className="text-xs text-[hsl(var(--muted-foreground))]">
                  Use @filename to reference specific files
                </p>
              </div>

              <div className="grid gap-4 sm:grid-cols-2">
                <div className="space-y-2">
                  <label className="text-sm font-medium">
                    Title (Optional)
                  </label>
                  <Input
                    placeholder="Add user authentication"
                    value={title}
                    onChange={(e) => setTitle(e.target.value)}
                  />
                </div>

                {!isComposite && (
                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      Branch Name (Optional)
                    </label>
                    <Input
                      placeholder="feature/add-user-auth"
                      value={branchName}
                      onChange={(e) => setBranchName(e.target.value)}
                    />
                  </div>
                )}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Agent Configuration</CardTitle>
              <CardDescription>
                Choose which AI agent to use for this task.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="composite"
                  checked={isComposite}
                  onChange={(e) => handleCompositeChange(e.target.checked)}
                  className="h-4 w-4 rounded border-[hsl(var(--input))]"
                />
                <label htmlFor="composite" className="text-sm font-medium">
                  Composite mode
                </label>
              </div>
              <p className="text-xs text-[hsl(var(--muted-foreground))]">
                {isComposite
                  ? "Creates a CompositeTask with an AI-generated plan"
                  : "Direct single-step execution as a UnitTask"}
              </p>

              {isComposite ? (
                <div className="grid gap-4 sm:grid-cols-2">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Planning Agent</label>
                    <Select
                      value={planningAgentType}
                      onChange={(e) => handlePlanningAgentTypeChange(e.target.value as AiAgentType)}
                    >
                      <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                      <option value={AiAgentType.OpenCode}>OpenCode</option>
                      <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                      <option value={AiAgentType.CodexCli}>Codex CLI</option>
                      <option value={AiAgentType.Aider}>Aider</option>
                      <option value={AiAgentType.Amp}>Amp</option>
                    </Select>
                    <p className="text-xs text-[hsl(var(--muted-foreground))]">
                      Agent for generating the task plan
                    </p>
                  </div>
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Execution Agent</label>
                    <Select
                      value={executionAgentType}
                      onChange={(e) => handleExecutionAgentTypeChange(e.target.value as AiAgentType)}
                    >
                      <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                      <option value={AiAgentType.OpenCode}>OpenCode</option>
                      <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                      <option value={AiAgentType.CodexCli}>Codex CLI</option>
                      <option value={AiAgentType.Aider}>Aider</option>
                      <option value={AiAgentType.Amp}>Amp</option>
                    </Select>
                    <p className="text-xs text-[hsl(var(--muted-foreground))]">
                      Agent for executing the UnitTasks
                    </p>
                  </div>
                </div>
              ) : (
                <div className="space-y-2">
                  <label className="text-sm font-medium">Agent Type</label>
                  <Select
                    value={agentType}
                    onChange={(e) => handleAgentTypeChange(e.target.value as AiAgentType)}
                  >
                    <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                    <option value={AiAgentType.OpenCode}>OpenCode</option>
                    <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                    <option value={AiAgentType.CodexCli}>Codex CLI</option>
                    <option value={AiAgentType.Aider}>Aider</option>
                    <option value={AiAgentType.Amp}>Amp</option>
                  </Select>
                </div>
              )}
            </CardContent>
          </Card>

          <div className="flex justify-end gap-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => navigate(-1)}
            >
              Cancel
            </Button>
            <Button type="submit" disabled={!selection || !prompt || isPending}>
              {isPending ? "Creating..." : "Create Task"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
