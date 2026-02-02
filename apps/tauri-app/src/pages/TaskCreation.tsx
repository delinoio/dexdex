import { useState } from "react";
import { useNavigate, Link } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Textarea } from "@/components/ui/Textarea";
import { useRepositoryGroups } from "@/hooks/useRepositoryGroups";
import { useRepositories } from "@/hooks/useRepositories";
import { useCreateUnitTask, useCreateCompositeTask } from "@/hooks/useTasks";
import { AiAgentType } from "@/api/types";

export function TaskCreation() {
  const [repositoryGroupId, setRepositoryGroupId] = useState("");
  const [prompt, setPrompt] = useState("");
  const [title, setTitle] = useState("");
  const [branchName, setBranchName] = useState("");
  const [agentType, setAgentType] = useState<AiAgentType>(AiAgentType.ClaudeCode);
  const [isComposite, setIsComposite] = useState(false);
  const navigate = useNavigate();

  const { data: groupsData } = useRepositoryGroups({});
  const { data: repositoriesData } = useRepositories({});
  const createUnitTask = useCreateUnitTask();
  const createCompositeTask = useCreateCompositeTask();

  const groups = groupsData?.groups ?? [];
  const repositories = repositoriesData?.repositories ?? [];

  const selectedGroup = groups.find((g) => g.id === repositoryGroupId);
  const groupRepositories = selectedGroup
    ? repositories.filter((repo) => selectedGroup.repositoryIds.includes(repo.id))
    : [];

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!repositoryGroupId || !prompt) return;

    try {
      if (isComposite) {
        const task = await createCompositeTask.mutateAsync({
          repositoryGroupId,
          prompt,
          title: title || undefined,
          executionAgentType: agentType,
        });
        navigate(`/composite-tasks/${task.id}`);
      } else {
        const task = await createUnitTask.mutateAsync({
          repositoryGroupId,
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

  const isPending = createUnitTask.isPending || createCompositeTask.isPending;

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <h1 className="text-2xl font-bold">Create Task</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <form onSubmit={handleSubmit} className="mx-auto max-w-2xl space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Repository Group</CardTitle>
              <CardDescription>
                Select a repository group for this task.
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-3">
              <Select
                value={repositoryGroupId}
                onChange={(e) => setRepositoryGroupId(e.target.value)}
                required
              >
                <option value="">Select a repository group...</option>
                {groups.map((group) => (
                  <option key={group.id} value={group.id}>
                    {group.name || "Unnamed Group"} ({group.repositoryIds.length}{" "}
                    {group.repositoryIds.length === 1 ? "repo" : "repos"})
                  </option>
                ))}
              </Select>

              {selectedGroup && groupRepositories.length > 0 && (
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

              {groups.length === 0 && (
                <div className="text-center">
                  <p className="text-sm text-[hsl(var(--muted-foreground))]">
                    No repository groups available.
                  </p>
                  <Link
                    to="/repository-groups"
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
                    Create a repository group
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
              <div className="space-y-2">
                <label className="text-sm font-medium">Agent Type</label>
                <Select
                  value={agentType}
                  onChange={(e) => setAgentType(e.target.value as AiAgentType)}
                >
                  <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                  <option value={AiAgentType.OpenCode}>OpenCode</option>
                  <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                  <option value={AiAgentType.CodexCli}>Codex CLI</option>
                  <option value={AiAgentType.Aider}>Aider</option>
                  <option value={AiAgentType.Amp}>Amp</option>
                </Select>
              </div>

              <div className="flex items-center space-x-2">
                <input
                  type="checkbox"
                  id="composite"
                  checked={isComposite}
                  onChange={(e) => setIsComposite(e.target.checked)}
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
            <Button type="submit" disabled={!repositoryGroupId || !prompt || isPending}>
              {isPending ? "Creating..." : "Create Task"}
            </Button>
          </div>
        </form>
      </div>
    </div>
  );
}
