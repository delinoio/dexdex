import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { KanbanBoard } from "@/components/dashboard/KanbanBoard";
import { useTasks } from "@/hooks/useTasks";
import { useUiStore } from "@/stores/uiStore";

export function Dashboard() {
  const navigate = useNavigate();
  const { data, isLoading, error } = useTasks({});
  const setTaskCreationOpen = useUiStore((state) => state.setTaskCreationOpen);

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
          <p className="text-[hsl(var(--destructive))]">Failed to load tasks</p>
          <p className="mt-1 text-sm text-[hsl(var(--muted-foreground))]">
            {error instanceof Error ? error.message : "Unknown error"}
          </p>
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
        <Button onClick={handleNewTask}>
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
          New Task
        </Button>
      </div>

      <div className="flex-1 overflow-hidden p-6">
        <KanbanBoard
          unitTasks={unitTasks}
          compositeTasks={compositeTasks}
          onTaskClick={handleTaskClick}
        />
      </div>
    </div>
  );
}
