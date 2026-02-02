import { useMemo } from "react";
import { Button } from "@/components/ui/Button";
import type { Repository, RepositoryGroup } from "@/api/types";

interface RepositoryGroupCardProps {
  group: RepositoryGroup;
  repositories: Repository[];
  onEdit: () => void;
  onDelete: () => void;
  isDeleting: boolean;
}

export function RepositoryGroupCard({
  group,
  repositories,
  onEdit,
  onDelete,
  isDeleting,
}: RepositoryGroupCardProps) {
  const repositoryMap = useMemo(
    () => new Map(repositories.map((r) => [r.id, r])),
    [repositories]
  );

  // Handle case where repositoryIds might be undefined
  const repositoryIds = group.repositoryIds ?? [];

  const groupRepositories = useMemo(
    () =>
      repositoryIds
        .map((id) => repositoryMap.get(id))
        .filter((r): r is Repository => r !== undefined),
    [repositoryIds, repositoryMap]
  );

  // If no name is set, show the list of repository names as the title
  const displayName = useMemo(() => {
    if (group.name) {
      return group.name;
    }
    if (groupRepositories.length > 0) {
      return groupRepositories.map((r) => r.name).join(", ");
    }
    return "Unnamed Group";
  }, [group.name, groupRepositories]);

  const repoCount = repositoryIds.length;

  return (
    <div className="rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-4">
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 overflow-hidden">
          <div className="flex items-center gap-2">
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="20"
              height="20"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
              className="shrink-0 text-[hsl(var(--primary))]"
            >
              <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
              <path d="M12 11v6" />
              <path d="M9 14h6" />
            </svg>
            <h3 className="truncate text-lg font-semibold">{displayName}</h3>
          </div>
          <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
            {repoCount} {repoCount === 1 ? "repository" : "repositories"}
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-1">
          <Button variant="ghost" size="sm" onClick={onEdit}>
            Edit
          </Button>
          <Button
            variant="ghost"
            size="icon"
            onClick={onDelete}
            disabled={isDeleting}
            aria-label="Delete repository group"
            className="text-[hsl(var(--destructive))] hover:bg-[hsl(var(--destructive))] hover:text-white"
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
      </div>
      <div className="mt-3 flex flex-wrap gap-2">
        {groupRepositories.map((repo) => (
          <div
            key={repo.id}
            className="flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted))] px-2 py-1 text-xs"
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
            <span className="truncate max-w-[150px]">{repo.name}</span>
          </div>
        ))}
        {repositoryIds.length > groupRepositories.length && (
          <div className="flex items-center gap-1.5 rounded-md bg-[hsl(var(--muted))] px-2 py-1 text-xs text-[hsl(var(--muted-foreground))]">
            +{repositoryIds.length - groupRepositories.length} unknown
          </div>
        )}
      </div>
    </div>
  );
}
