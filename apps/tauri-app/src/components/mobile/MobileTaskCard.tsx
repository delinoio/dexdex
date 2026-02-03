// Mobile-optimized task card with swipe actions
import { memo } from "react";
import { useNavigate } from "react-router-dom";
import { Badge } from "@/components/ui/Badge";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { UnitTaskIcon, CompositeTaskIcon } from "@/components/ui/Icons";
import { SwipeableCard } from "./SwipeableCard";
import type { UnitTask, CompositeTask, UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import { cn } from "@/lib/utils";

interface MobileTaskCardProps {
  task: UnitTask | CompositeTask;
  onApprove?: () => void;
  onReject?: () => void;
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
      return "Pending";
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

export const MobileTaskCard = memo(function MobileTaskCard({
  task,
  onApprove,
  onReject,
}: MobileTaskCardProps) {
  const navigate = useNavigate();
  const title = task.title || task.prompt.slice(0, 50) + (task.prompt.length > 50 ? "..." : "");
  const isUnit = isUnitTask(task);
  const status = task.status;

  const canApprove = status === "in_review" || status === "pending_approval";
  const canReject = status === "in_review" || status === "pending_approval";

  const handleClick = () => {
    if (isUnit) {
      navigate(`/unit-tasks/${task.id}`);
    } else {
      navigate(`/composite-tasks/${task.id}`);
    }
  };

  const cardContent = (
    <div
      className={cn(
        "p-4 border border-[hsl(var(--border))] rounded-lg bg-[hsl(var(--card))]",
        "active:bg-[hsl(var(--muted))] touch-manipulation"
      )}
      onClick={handleClick}
      role="button"
      tabIndex={0}
      aria-label={`${isUnit ? "Unit" : "Composite"} task: ${title}`}
    >
      {/* Header */}
      <div className="flex items-start justify-between gap-3">
        <div className="flex items-center gap-2 min-w-0 flex-1">
          <span className="shrink-0 text-[hsl(var(--muted-foreground))]">
            {isUnit ? <UnitTaskIcon size={16} /> : <CompositeTaskIcon size={16} />}
          </span>
          <h3 className="font-medium text-sm truncate">{title}</h3>
        </div>
        <Badge variant={getStatusBadgeVariant(status)} className="shrink-0 text-xs">
          {formatStatus(status)}
        </Badge>
      </div>

      {/* Body */}
      <p className="mt-2 text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
        {task.prompt}
      </p>

      {/* Footer */}
      <div className="mt-3 flex items-center justify-between text-xs text-[hsl(var(--muted-foreground))]">
        <span>{isUnit ? "Unit Task" : "Composite Task"}</span>
        <FormattedDateTime date={task.createdAt} />
      </div>
    </div>
  );

  // Add swipe actions for reviewable tasks
  if (canApprove || canReject) {
    return (
      <SwipeableCard
        leftAction={
          canApprove && onApprove
            ? {
                label: "Approve",
                icon: (
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
                    <polyline points="20 6 9 17 4 12" />
                  </svg>
                ),
                color: "success",
                onClick: onApprove,
              }
            : undefined
        }
        rightAction={
          canReject && onReject
            ? {
                label: "Reject",
                icon: (
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
                    <line x1="18" x2="6" y1="6" y2="18" />
                    <line x1="6" x2="18" y1="6" y2="18" />
                  </svg>
                ),
                color: "destructive",
                onClick: onReject,
              }
            : undefined
        }
      >
        {cardContent}
      </SwipeableCard>
    );
  }

  return cardContent;
});
