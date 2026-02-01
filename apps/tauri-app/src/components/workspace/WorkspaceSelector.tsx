import { useEffect, useState } from "react";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/uiStore";
import {
  useWorkspaces,
  useDefaultWorkspaceId,
  useCreateWorkspace,
} from "@/hooks/useWorkspaces";
import type { Workspace } from "@/api/types";

interface WorkspaceSelectorProps {
  collapsed?: boolean;
}

export function WorkspaceSelector({ collapsed = false }: WorkspaceSelectorProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [isCreating, setIsCreating] = useState(false);
  const [newWorkspaceName, setNewWorkspaceName] = useState("");

  const currentWorkspaceId = useUiStore((state) => state.currentWorkspaceId);
  const setCurrentWorkspaceId = useUiStore(
    (state) => state.setCurrentWorkspaceId
  );

  const { data: workspacesData, isLoading: isLoadingWorkspaces } =
    useWorkspaces();
  const { data: defaultWorkspaceId, isLoading: isLoadingDefault } =
    useDefaultWorkspaceId();
  const createWorkspace = useCreateWorkspace();

  const workspaces = workspacesData?.workspaces ?? [];
  const currentWorkspace = workspaces.find((w) => w.id === currentWorkspaceId);

  // Set default workspace on initial load
  useEffect(() => {
    if (!currentWorkspaceId && defaultWorkspaceId && !isLoadingDefault) {
      setCurrentWorkspaceId(defaultWorkspaceId);
    }
  }, [currentWorkspaceId, defaultWorkspaceId, isLoadingDefault, setCurrentWorkspaceId]);

  const handleSelectWorkspace = (workspace: Workspace) => {
    setCurrentWorkspaceId(workspace.id);
    setIsOpen(false);
  };

  const handleCreateWorkspace = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newWorkspaceName.trim()) return;

    try {
      const newWorkspace = await createWorkspace.mutateAsync({
        name: newWorkspaceName.trim(),
      });
      setCurrentWorkspaceId(newWorkspace.id);
      setNewWorkspaceName("");
      setIsCreating(false);
      setIsOpen(false);
    } catch {
      // Error handling is done by react-query
    }
  };

  if (isLoadingWorkspaces || isLoadingDefault) {
    return (
      <div className="px-3 py-2">
        <div className="h-8 w-full animate-pulse rounded-md bg-[hsl(var(--muted))]" />
      </div>
    );
  }

  if (collapsed) {
    return (
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex h-10 w-10 items-center justify-center rounded-md hover:bg-[hsl(var(--muted))]"
        title={currentWorkspace?.name ?? "Select workspace"}
      >
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
        >
          <path d="M3 9h18v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9Z" />
          <path d="m3 9 2.45-4.9A2 2 0 0 1 7.24 3h9.52a2 2 0 0 1 1.8 1.1L21 9" />
          <path d="M12 3v6" />
        </svg>
      </button>
    );
  }

  return (
    <div className="relative px-3 py-2">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          "flex w-full items-center justify-between gap-2 rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--background))] px-3 py-2 text-sm transition-colors",
          "hover:bg-[hsl(var(--muted))]",
          isOpen && "ring-2 ring-[hsl(var(--ring))]"
        )}
      >
        <div className="flex items-center gap-2 overflow-hidden">
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
            className="shrink-0"
          >
            <path d="M3 9h18v10a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V9Z" />
            <path d="m3 9 2.45-4.9A2 2 0 0 1 7.24 3h9.52a2 2 0 0 1 1.8 1.1L21 9" />
            <path d="M12 3v6" />
          </svg>
          <span className="truncate font-medium">
            {currentWorkspace?.name ?? "Select workspace"}
          </span>
        </div>
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
          className={cn(
            "shrink-0 transition-transform",
            isOpen && "rotate-180"
          )}
        >
          <path d="m6 9 6 6 6-6" />
        </svg>
      </button>

      {isOpen && (
        <>
          <div
            className="fixed inset-0 z-10"
            onClick={() => {
              setIsOpen(false);
              setIsCreating(false);
            }}
          />
          <div className="absolute left-3 right-3 top-full z-20 mt-1 max-h-64 overflow-auto rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--background))] shadow-lg">
            {workspaces.map((workspace) => (
              <button
                key={workspace.id}
                onClick={() => handleSelectWorkspace(workspace)}
                className={cn(
                  "flex w-full items-center gap-2 px-3 py-2 text-sm transition-colors",
                  "hover:bg-[hsl(var(--muted))]",
                  workspace.id === currentWorkspaceId &&
                    "bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]"
                )}
              >
                <span className="truncate">{workspace.name}</span>
                {workspace.id === currentWorkspaceId && (
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
                    className="ml-auto shrink-0"
                  >
                    <path d="M20 6 9 17l-5-5" />
                  </svg>
                )}
              </button>
            ))}

            <div className="border-t border-[hsl(var(--border))]">
              {isCreating ? (
                <form onSubmit={handleCreateWorkspace} className="p-2">
                  <input
                    type="text"
                    value={newWorkspaceName}
                    onChange={(e) => setNewWorkspaceName(e.target.value)}
                    placeholder="Workspace name"
                    autoFocus
                    className="w-full rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--background))] px-3 py-1.5 text-sm outline-none focus:ring-2 focus:ring-[hsl(var(--ring))]"
                    onKeyDown={(e) => {
                      if (e.key === "Escape") {
                        setIsCreating(false);
                        setNewWorkspaceName("");
                      }
                    }}
                  />
                  <div className="mt-2 flex justify-end gap-2">
                    <button
                      type="button"
                      onClick={() => {
                        setIsCreating(false);
                        setNewWorkspaceName("");
                      }}
                      className="rounded-md px-2 py-1 text-sm hover:bg-[hsl(var(--muted))]"
                    >
                      Cancel
                    </button>
                    <button
                      type="submit"
                      disabled={
                        !newWorkspaceName.trim() || createWorkspace.isPending
                      }
                      className="rounded-md bg-[hsl(var(--primary))] px-2 py-1 text-sm text-[hsl(var(--primary-foreground))] disabled:opacity-50"
                    >
                      {createWorkspace.isPending ? "Creating..." : "Create"}
                    </button>
                  </div>
                </form>
              ) : (
                <button
                  onClick={() => setIsCreating(true)}
                  className="flex w-full items-center gap-2 px-3 py-2 text-sm transition-colors hover:bg-[hsl(var(--muted))]"
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
                    <path d="M5 12h14" />
                    <path d="M12 5v14" />
                  </svg>
                  <span>Create workspace</span>
                </button>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
