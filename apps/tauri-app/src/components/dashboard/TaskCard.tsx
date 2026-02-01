import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import type { UnitTask, CompositeTask, UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import { cn } from "@/lib/utils";

interface TaskCardProps {
  task: UnitTask | CompositeTask;
  onClick?: () => void;
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
      return "secondary";
    case "rejected":
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
    default:
      return status;
  }
}

export function TaskCard({ task, onClick }: TaskCardProps) {
  const title = task.title || task.prompt.slice(0, 50) + (task.prompt.length > 50 ? "..." : "");
  const isUnit = isUnitTask(task);
  const status = task.status;

  return (
    <Card
      className={cn(
        "cursor-pointer transition-shadow hover:shadow-md",
        onClick && "hover:border-[hsl(var(--primary))]"
      )}
      onClick={onClick}
    >
      <CardHeader className="pb-2">
        <div className="flex items-start justify-between gap-2">
          <CardTitle className="text-sm font-medium line-clamp-2">
            {title}
          </CardTitle>
          <Badge variant={getStatusBadgeVariant(status)} className="shrink-0">
            {formatStatus(status)}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="pb-3">
        <div className="flex items-center gap-2 text-xs text-[hsl(var(--muted-foreground))]">
          <span className="flex items-center gap-1">
            {isUnit ? (
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
              >
                <path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z" />
              </svg>
            ) : (
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
              >
                <rect width="7" height="7" x="3" y="3" rx="1" />
                <rect width="7" height="7" x="14" y="3" rx="1" />
                <rect width="7" height="7" x="14" y="14" rx="1" />
                <rect width="7" height="7" x="3" y="14" rx="1" />
              </svg>
            )}
            {isUnit ? "Unit" : "Composite"}
          </span>
          <span>•</span>
          <span>
            {new Date(task.createdAt).toLocaleDateString()}
          </span>
        </div>
        <p className="mt-2 text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
          {task.prompt}
        </p>
      </CardContent>
    </Card>
  );
}
