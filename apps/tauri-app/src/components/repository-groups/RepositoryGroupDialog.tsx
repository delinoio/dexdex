import { useState, useEffect } from "react";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/Dialog";
import { useRepositories } from "@/hooks/useRepositories";
import type { RepositoryGroup } from "@/api/types";

interface RepositoryGroupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  group?: RepositoryGroup;
  onSubmit: (name: string | undefined, repositoryIds: string[]) => Promise<void>;
  isPending: boolean;
}

export function RepositoryGroupDialog({
  open,
  onOpenChange,
  group,
  onSubmit,
  isPending,
}: RepositoryGroupDialogProps) {
  const [name, setName] = useState("");
  const [selectedRepoIds, setSelectedRepoIds] = useState<string[]>([]);

  const { data: repositoriesData } = useRepositories({});
  const repositories = repositoriesData?.repositories ?? [];

  const isEdit = !!group;

  useEffect(() => {
    if (open) {
      if (group) {
        setName(group.name ?? "");
        setSelectedRepoIds(group.repositoryIds);
      } else {
        setName("");
        setSelectedRepoIds([]);
      }
    }
  }, [open, group]);

  const handleToggleRepo = (repoId: string) => {
    setSelectedRepoIds((prev) =>
      prev.includes(repoId)
        ? prev.filter((id) => id !== repoId)
        : [...prev, repoId]
    );
  };

  const handleSubmit = async () => {
    if (selectedRepoIds.length === 0) return;
    await onSubmit(name || undefined, selectedRepoIds);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          <DialogTitle>
            {isEdit ? "Edit Repository Group" : "Create Repository Group"}
          </DialogTitle>
          <DialogDescription>
            Group related repositories for multi-repo tasks.
          </DialogDescription>
        </DialogHeader>
        <div className="space-y-4 py-4">
          <div className="space-y-2">
            <label className="text-sm font-medium">Group Name (Optional)</label>
            <Input
              placeholder="Full Stack App"
              value={name}
              onChange={(e) => setName(e.target.value)}
            />
          </div>
          <div className="space-y-2">
            <label className="text-sm font-medium">
              Select Repositories (at least 1)
            </label>
            <div className="max-h-48 overflow-y-auto rounded-md border border-[hsl(var(--border))]">
              {repositories.length === 0 ? (
                <p className="p-3 text-sm text-[hsl(var(--muted-foreground))]">
                  No repositories available. Add repositories first.
                </p>
              ) : (
                repositories.map((repo) => (
                  <label
                    key={repo.id}
                    className="flex cursor-pointer items-center gap-3 border-b border-[hsl(var(--border))] p-3 last:border-b-0 hover:bg-[hsl(var(--muted))]"
                  >
                    <input
                      type="checkbox"
                      checked={selectedRepoIds.includes(repo.id)}
                      onChange={() => handleToggleRepo(repo.id)}
                      className="h-4 w-4 rounded border-[hsl(var(--input))]"
                    />
                    <div className="flex-1 overflow-hidden">
                      <p className="truncate text-sm font-medium">{repo.name}</p>
                      <p className="truncate text-xs text-[hsl(var(--muted-foreground))]">
                        {repo.remoteUrl}
                      </p>
                    </div>
                  </label>
                ))
              )}
            </div>
          </div>
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={selectedRepoIds.length === 0 || isPending}
          >
            {isPending
              ? isEdit
                ? "Saving..."
                : "Creating..."
              : isEdit
                ? "Save Changes"
                : "Create Group"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
