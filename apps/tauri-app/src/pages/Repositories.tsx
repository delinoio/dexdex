import { useState } from "react";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/Dialog";
import { useRepositories, useAddRepository, useRemoveRepository } from "@/hooks/useRepositories";

export function Repositories() {
  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [repoUrl, setRepoUrl] = useState("");
  const [repoName, setRepoName] = useState("");

  const { data, isLoading, error } = useRepositories({});
  const addRepository = useAddRepository();
  const removeRepository = useRemoveRepository();

  const handleAddRepository = async () => {
    if (!repoUrl) return;

    try {
      await addRepository.mutateAsync({
        remoteUrl: repoUrl,
        name: repoName || undefined,
      });
      setIsAddDialogOpen(false);
      setRepoUrl("");
      setRepoName("");
    } catch (error) {
      console.error("Failed to add repository:", error);
    }
  };

  const handleRemoveRepository = async (id: string) => {
    try {
      await removeRepository.mutateAsync(id);
    } catch (error) {
      console.error("Failed to remove repository:", error);
    }
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-[hsl(var(--destructive))]">
            Failed to load repositories
          </p>
          <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
            {error instanceof Error ? error.message : "Unknown error"}
          </p>
        </div>
      </div>
    );
  }

  const repositories = data?.repositories ?? [];

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-6 py-4">
        <div>
          <h1 className="text-2xl font-bold">Repository Management</h1>
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            {data?.totalCount ?? 0} registered repositories
          </p>
        </div>
        <Button onClick={() => setIsAddDialogOpen(true)}>
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="mr-2"
          >
            <path d="M5 12h14" />
            <path d="M12 5v14" />
          </svg>
          Add Repository
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Registered Repositories</CardTitle>
              <CardDescription>
                Repositories available for task creation
              </CardDescription>
            </CardHeader>
            <CardContent>
              {repositories.length === 0 ? (
                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                  No repositories registered yet.
                </p>
              ) : (
                <div className="space-y-2">
                  {repositories.map((repo) => (
                    <div
                      key={repo.id}
                      className="flex items-center justify-between rounded-md border border-[hsl(var(--border))] p-3"
                    >
                      <div className="flex items-center gap-3">
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
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
                        <div>
                          <p className="text-sm font-medium">{repo.name}</p>
                          <p className="text-xs text-[hsl(var(--muted-foreground))]">
                            {repo.remoteUrl}
                          </p>
                        </div>
                      </div>
                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemoveRepository(repo.id)}
                        disabled={removeRepository.isPending}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="16"
                          height="16"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                        >
                          <path d="M18 6 6 18" />
                          <path d="m6 6 12 12" />
                        </svg>
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      <Dialog open={isAddDialogOpen} onOpenChange={setIsAddDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Add Repository</DialogTitle>
            <DialogDescription>
              Enter the repository URL to add it to DexDex.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <label className="text-sm font-medium">Repository URL</label>
              <Input
                placeholder="https://github.com/user/repo"
                value={repoUrl}
                onChange={(e) => setRepoUrl(e.target.value)}
              />
            </div>
            <div className="space-y-2">
              <label className="text-sm font-medium">Name (Optional)</label>
              <Input
                placeholder="my-repo"
                value={repoName}
                onChange={(e) => setRepoName(e.target.value)}
              />
              <p className="text-xs text-[hsl(var(--muted-foreground))]">
                Leave empty to use the repository name from the URL
              </p>
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setIsAddDialogOpen(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleAddRepository}
              disabled={!repoUrl || addRepository.isPending}
            >
              {addRepository.isPending ? "Adding..." : "Add Repository"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
