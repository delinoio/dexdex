import { useState } from "react";
import { Button } from "@/components/ui/Button";
import {
  RepositoryGroupCard,
  RepositoryGroupDialog,
} from "@/components/repository-groups";
import {
  useRepositoryGroups,
  useCreateRepositoryGroup,
  useUpdateRepositoryGroup,
  useDeleteRepositoryGroup,
} from "@/hooks/useRepositoryGroups";
import { useRepositories } from "@/hooks/useRepositories";
import type { RepositoryGroup } from "@/api/types";

export function RepositoryGroups() {
  const [isDialogOpen, setIsDialogOpen] = useState(false);
  const [editingGroup, setEditingGroup] = useState<RepositoryGroup | undefined>(
    undefined
  );
  const [deletingGroupId, setDeletingGroupId] = useState<string | null>(null);

  const { data: groupsData, isLoading, error } = useRepositoryGroups({});
  const { data: repositoriesData } = useRepositories({});
  const createGroup = useCreateRepositoryGroup();
  const updateGroup = useUpdateRepositoryGroup();
  const deleteGroup = useDeleteRepositoryGroup();

  const handleCreate = async (
    name: string | undefined,
    repositoryIds: string[]
  ) => {
    try {
      await createGroup.mutateAsync({
        name,
        repositoryIds,
      });
    } catch (error) {
      console.error("Failed to create repository group:", error);
    }
  };

  const handleUpdate = async (
    name: string | undefined,
    repositoryIds: string[]
  ) => {
    if (!editingGroup) return;
    try {
      await updateGroup.mutateAsync({
        groupId: editingGroup.id,
        params: {
          name,
          repositoryIds,
        },
      });
    } catch (error) {
      console.error("Failed to update repository group:", error);
    }
  };

  const handleDelete = async (groupId: string) => {
    setDeletingGroupId(groupId);
    try {
      await deleteGroup.mutateAsync(groupId);
    } catch (error) {
      console.error("Failed to delete repository group:", error);
    } finally {
      setDeletingGroupId(null);
    }
  };

  const openCreateDialog = () => {
    setEditingGroup(undefined);
    setIsDialogOpen(true);
  };

  const openEditDialog = (group: RepositoryGroup) => {
    setEditingGroup(group);
    setIsDialogOpen(true);
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
            Failed to load repository groups
          </p>
          <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
            {error instanceof Error ? error.message : "Unknown error"}
          </p>
        </div>
      </div>
    );
  }

  const groups = groupsData?.groups ?? [];
  const repositories = repositoriesData?.repositories ?? [];

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-6 py-4">
        <div>
          <h1 className="text-2xl font-bold">Repository Groups</h1>
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            Create groups of repositories for multi-repository tasks.
          </p>
        </div>
        <Button onClick={openCreateDialog}>
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
          Create Group
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl space-y-4">
          {groups.length === 0 ? (
            <div className="flex flex-col items-center justify-center rounded-lg border border-dashed border-[hsl(var(--border))] py-12">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="48"
                height="48"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="text-[hsl(var(--muted-foreground))]"
              >
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z" />
                <path d="M12 11v6" />
                <path d="M9 14h6" />
              </svg>
              <h3 className="mt-4 text-lg font-medium">No repository groups</h3>
              <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
                Create a group to work with multiple repositories at once.
              </p>
              <Button className="mt-4" onClick={openCreateDialog}>
                Create your first group
              </Button>
            </div>
          ) : (
            groups.map((group) => (
              <RepositoryGroupCard
                key={group.id}
                group={group}
                repositories={repositories}
                onEdit={() => openEditDialog(group)}
                onDelete={() => handleDelete(group.id)}
                isDeleting={deletingGroupId === group.id}
              />
            ))
          )}
        </div>
      </div>

      <RepositoryGroupDialog
        open={isDialogOpen}
        onOpenChange={setIsDialogOpen}
        group={editingGroup}
        onSubmit={editingGroup ? handleUpdate : handleCreate}
        isPending={createGroup.isPending || updateGroup.isPending}
      />
    </div>
  );
}
