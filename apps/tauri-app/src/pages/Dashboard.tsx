import { useState, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { PlusIcon, RefreshIcon, AlertCircleIcon } from "@/components/ui/Icons";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/Dialog";
import { KanbanBoard } from "@/components/dashboard/KanbanBoard";
import { useTasks, useDeleteTask } from "@/hooks/useTasks";
import { useUiStore } from "@/stores/uiStore";

export function Dashboard() {
  const navigate = useNavigate();
  const { data, isLoading, error, refetch, isRefetching } = useTasks({});
  const setTaskCreationOpen = useUiStore((state) => state.setTaskCreationOpen);
  const deleteMutation = useDeleteTask();
  const [deleteTaskId, setDeleteTaskId] = useState<string | null>(null);

  const handleTaskClick = (taskId: string, isUnit: boolean) => {
    if (isUnit) {
      navigate(`/unit-tasks/${taskId}`);
    } else {
      navigate(`/composite-tasks/${taskId}`);
    }
  };

  const handleNewTask = () => {
    setTaskCreationOpen(true);
    navigate("/tasks/new");
  };

  const handleRetry = () => {
    refetch();
  };

  const handleDeleteRequest = useCallback((taskId: string) => {
    setDeleteTaskId(taskId);
  }, []);

  const handleDeleteConfirm = useCallback(async () => {
    if (!deleteTaskId || deleteMutation.isPending) return;
    try {
      await deleteMutation.mutateAsync(deleteTaskId);
    } catch (err) {
      console.error("Failed to delete task:", err);
    }
    setDeleteTaskId(null);
  }, [deleteTaskId, deleteMutation]);

  const handleDeleteCancel = useCallback(() => {
    setDeleteTaskId(null);
  }, []);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center" role="status" aria-live="polite">
        <div className="text-[hsl(var(--muted-foreground))]">Loading tasks...</div>
      </div>
    );
  }

  if (error) {
    const errorMessage = error instanceof Error ? error.message : "An unexpected error occurred";
    const isNetworkError = errorMessage.toLowerCase().includes("network") ||
                           errorMessage.toLowerCase().includes("fetch");

    return (
      <div className="flex h-full items-center justify-center" role="alert" aria-live="assertive">
        <div className="text-center max-w-md px-4">
          <AlertCircleIcon size={48} className="mx-auto mb-4 text-[hsl(var(--destructive))]" />
          <h2 className="text-lg font-semibold text-[hsl(var(--destructive))]">
            Failed to load tasks
          </h2>
          <p className="mt-2 text-sm text-[hsl(var(--muted-foreground))]">
            {errorMessage}
          </p>
          <p className="mt-1 text-xs text-[hsl(var(--muted-foreground))]">
            {isNetworkError
              ? "Please check your connection and try again."
              : "Please try again or contact support if the problem persists."}
          </p>
          <Button
            onClick={handleRetry}
            disabled={isRefetching}
            className="mt-4"
            aria-label="Retry loading tasks"
          >
            <RefreshIcon size={16} className={`mr-2 ${isRefetching ? "animate-spin" : ""}`} />
            {isRefetching ? "Retrying..." : "Try Again"}
          </Button>
        </div>
      </div>
    );
  }

  const unitTasks = data?.unitTasks ?? [];
  const compositeTasks = data?.compositeTasks ?? [];

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-6 py-4">
        <div>
          <h1 className="text-2xl font-bold">Dashboard</h1>
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            {data?.totalCount ?? 0} total tasks
          </p>
        </div>
        <Button onClick={handleNewTask} aria-label="Create new task">
          <PlusIcon size={16} className="mr-2" />
          New Task
        </Button>
      </div>

      <div className="flex-1 overflow-hidden p-6">
        <KanbanBoard
          unitTasks={unitTasks}
          compositeTasks={compositeTasks}
          onTaskClick={handleTaskClick}
          onDeleteTask={handleDeleteRequest}
        />
      </div>

      {/* Delete confirmation dialog */}
      <Dialog open={deleteTaskId !== null} onOpenChange={(open) => { if (!open) handleDeleteCancel(); }}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Task</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this task? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={handleDeleteCancel}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDeleteConfirm}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
