import { memo, type KeyboardEvent } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { UnitTaskIcon, CompositeTaskIcon, MoreVerticalIcon, TrashIcon } from "@/components/ui/Icons";
import {
  DropdownMenu,
  DropdownMenuTrigger,
  DropdownMenuContent,
  DropdownMenuItem,
} from "@/components/ui/DropdownMenu";
import type { UnitTask, CompositeTask, UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import { cn } from "@/lib/utils";

interface TaskCardProps {
  task: UnitTask | CompositeTask;
  onClick?: () => void;
  onDelete?: (taskId: string) => void;
}

function isUnitTask(task: UnitTask | CompositeTask): task is UnitTask {
  return "agentTaskId" in task;
}

function getStatusBadgeVariant(
  status: UnitTaskStatus | CompositeTaskStatus
): "default" | "secondary" | "destructive" | "outline" {
  switch (status) {
    case "in_progress":
    case "planning":
      return "default";
    case "in_review":
    case "pending_approval":
    case "approved":
      return "secondary";
    case "rejected":
    case "failed":
      return "destructive";
    default:
      return "outline";
  }
}

function formatStatus(status: UnitTaskStatus | CompositeTaskStatus): string {
  switch (status) {
    case "in_progress":
      return "In Progress";
    case "in_review":
      return "In Review";
    case "pr_open":
      return "PR Open";
    case "planning":
      return "Planning";
    case "pending_approval":
      return "Pending Approval";
    case "done":
      return "Done";
    case "rejected":
      return "Rejected";
    case "approved":
      return "Approved";
    case "failed":
      return "Failed";
    case "cancelled":
      return "Cancelled";
    default:
      return status;
  }
}

export const TaskCard = memo(function TaskCard({ task, onClick, onDelete }: TaskCardProps) {
  const title = task.title || task.prompt.slice(0, 50) + (task.prompt.length > 50 ? "..." : "");
  const isUnit = isUnitTask(task);
  const status = task.status;
  const taskType = isUnit ? "Unit task" : "Composite task";

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onClick?.();
    }
  };

  return (
    <Card
      className={cn(
        "cursor-pointer transition-shadow hover:shadow-md focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[hsl(var(--ring))] focus-visible:ring-offset-2",
        onClick && "hover:border-[hsl(var(--primary))]"
      )}
      onClick={onClick}
      onKeyDown={handleKeyDown}
      role="button"
      tabIndex={0}
      aria-label={`${taskType}: ${title}. Status: ${formatStatus(status)}`}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <CardTitle className="text-sm font-medium line-clamp-2">
            {title}
          </CardTitle>
          <div className="flex items-center gap-1 shrink-0">
            <Badge variant={getStatusBadgeVariant(status)} className="shrink-0">
              {formatStatus(status)}
            </Badge>
            {onDelete && (
              <DropdownMenu>
                <DropdownMenuTrigger
                  className="inline-flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--accent))] hover:text-[hsl(var(--accent-foreground))]"
                  aria-label="Task actions"
                >
                  <MoreVerticalIcon size={14} />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end">
                  <DropdownMenuItem
                    destructive
                    onClick={() => onDelete(task.id)}
                  >
                    <TrashIcon size={14} className="mr-2" />
                    Delete
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="pb-3">
        <div className="flex items-center gap-2 text-xs text-[hsl(var(--muted-foreground))]">
          <span className="flex items-center gap-1">
            {isUnit ? (
              <UnitTaskIcon size={12} />
            ) : (
              <CompositeTaskIcon size={12} />
            )}
            {isUnit ? "Unit" : "Composite"}
          </span>
          <span aria-hidden="true">&bull;</span>
          <FormattedDateTime date={task.createdAt} />
        </div>
        <p className="mt-2 text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
          {task.prompt}
        </p>
      </CardContent>
    </Card>
  );
});
