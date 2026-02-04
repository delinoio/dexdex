import { UnitTask, CompositeTask, UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import { KanbanColumn } from "./KanbanColumn";
import { TaskCard } from "./TaskCard";

interface KanbanBoardProps {
  unitTasks: UnitTask[];
  compositeTasks: CompositeTask[];
  onTaskClick?: (taskId: string, isUnit: boolean) => void;
}

type TaskWithType =
  | { type: "unit"; task: UnitTask }
  | { type: "composite"; task: CompositeTask };

const columns: {
  id: string;
  title: string;
  unitStatuses: UnitTaskStatus[];
  compositeStatuses: CompositeTaskStatus[];
}[] = [
  {
    id: "in-progress",
    title: "In Progress",
    unitStatuses: [UnitTaskStatus.InProgress],
    compositeStatuses: [CompositeTaskStatus.Planning, CompositeTaskStatus.InProgress],
  },
  {
    id: "in-review",
    title: "In Review",
    unitStatuses: [UnitTaskStatus.InReview],
    compositeStatuses: [CompositeTaskStatus.PendingApproval],
  },
  {
    id: "pr-open",
    title: "PR Open",
    unitStatuses: [UnitTaskStatus.PrOpen, UnitTaskStatus.Approved],
    compositeStatuses: [],
  },
  {
    id: "done",
    title: "Done",
    unitStatuses: [UnitTaskStatus.Done],
    compositeStatuses: [CompositeTaskStatus.Done],
  },
  {
    id: "rejected",
    title: "Rejected / Failed",
    unitStatuses: [UnitTaskStatus.Rejected, UnitTaskStatus.Failed],
    compositeStatuses: [CompositeTaskStatus.Rejected, CompositeTaskStatus.Failed],
  },
];

export function KanbanBoard({
  unitTasks,
  compositeTasks,
  onTaskClick,
}: KanbanBoardProps) {
  const getTasksForColumn = (column: typeof columns[number]): TaskWithType[] => {
    const tasks: TaskWithType[] = [];

    unitTasks.forEach((task) => {
      if (column.unitStatuses.includes(task.status)) {
        tasks.push({ type: "unit", task });
      }
    });

    compositeTasks.forEach((task) => {
      if (column.compositeStatuses.includes(task.status)) {
        tasks.push({ type: "composite", task });
      }
    });

    // Sort by updated date descending
    tasks.sort((a, b) => {
      const dateA = new Date(a.task.updatedAt).getTime();
      const dateB = new Date(b.task.updatedAt).getTime();
      return dateB - dateA;
    });

    return tasks;
  };

  return (
    <div className="flex h-full gap-4 overflow-x-auto pb-4">
      {columns.map((column) => {
        const tasks = getTasksForColumn(column);
        return (
          <KanbanColumn
            key={column.id}
            title={column.title}
            count={tasks.length}
            className="min-w-[280px] flex-1"
          >
            {tasks.map(({ type, task }) => (
              <TaskCard
                key={task.id}
                task={task}
                onClick={() => onTaskClick?.(task.id, type === "unit")}
              />
            ))}
            {tasks.length === 0 && (
              <div className="flex h-24 items-center justify-center rounded-lg border border-dashed border-[hsl(var(--border))] text-sm text-[hsl(var(--muted-foreground))]">
                No tasks
              </div>
            )}
          </KanbanColumn>
        );
      })}
    </div>
  );
}
