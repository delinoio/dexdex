import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/Badge';
import { FormattedDateTime } from '@/components/ui/FormattedDateTime';
import type { SubTask, SubTaskStatus, SubTaskType } from '@/api/types';

function statusIcon(status: SubTaskStatus) {
  switch (status) {
    case 'completed':
      return (
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
          className="text-green-500"
        >
          <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
          <polyline points="22 4 12 14.01 9 11.01" />
        </svg>
      );
    case 'in_progress':
      return (
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
          className="animate-spin text-[hsl(var(--primary))]"
        >
          <path d="M21 12a9 9 0 1 1-6.219-8.56" />
        </svg>
      );
    case 'failed':
      return (
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
          className="text-[hsl(var(--destructive))]"
        >
          <circle cx="12" cy="12" r="10" />
          <line x1="15" y1="9" x2="9" y2="15" />
          <line x1="9" y1="9" x2="15" y2="15" />
        </svg>
      );
    case 'waiting_for_plan_approval':
    case 'waiting_for_user_input':
      return (
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
          className="text-yellow-500"
        >
          <circle cx="12" cy="12" r="10" />
          <path d="M12 16v-4" />
          <path d="M12 8h.01" />
        </svg>
      );
    case 'queued':
      return (
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
          className="text-[hsl(var(--muted-foreground))]"
        >
          <circle cx="12" cy="12" r="10" />
        </svg>
      );
    default:
      return (
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
          className="text-[hsl(var(--muted-foreground))]"
        >
          <circle cx="12" cy="12" r="10" />
        </svg>
      );
  }
}

function formatSubTaskType(type: SubTaskType): string {
  switch (type) {
    case 'initial_implementation':
      return 'Initial Implementation';
    case 'request_changes':
      return 'Request Changes';
    case 'pr_create':
      return 'Create PR';
    case 'pr_review_fix':
      return 'PR Review Fix';
    case 'pr_ci_fix':
      return 'PR CI Fix';
    case 'manual_retry':
      return 'Manual Retry';
    default:
      return type;
  }
}

function formatStatus(status: SubTaskStatus): string {
  switch (status) {
    case 'queued':
      return 'Queued';
    case 'in_progress':
      return 'In Progress';
    case 'waiting_for_plan_approval':
      return 'Awaiting Plan Approval';
    case 'waiting_for_user_input':
      return 'Awaiting Input';
    case 'completed':
      return 'Completed';
    case 'failed':
      return 'Failed';
    case 'cancelled':
      return 'Cancelled';
    default:
      return status;
  }
}

function statusBadgeVariant(status: SubTaskStatus): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (status) {
    case 'completed':
      return 'default';
    case 'in_progress':
      return 'secondary';
    case 'waiting_for_plan_approval':
    case 'waiting_for_user_input':
      return 'outline';
    case 'failed':
    case 'cancelled':
      return 'destructive';
    default:
      return 'outline';
  }
}

interface SubTaskTimelineProps {
  subTasks: SubTask[];
  activeSubTaskId?: string;
  onSelectSubTask?: (id: string) => void;
  className?: string;
}

export function SubTaskTimeline({
  subTasks,
  activeSubTaskId,
  onSelectSubTask,
  className,
}: SubTaskTimelineProps) {
  if (subTasks.length === 0) {
    return (
      <div className={cn('text-sm text-[hsl(var(--muted-foreground))]', className)}>
        No subtasks yet.
      </div>
    );
  }

  return (
    <ol className={cn('space-y-2', className)}>
      {subTasks.map((subTask, idx) => (
        <li key={subTask.id}>
          <button
            type="button"
            onClick={() => onSelectSubTask?.(subTask.id)}
            className={cn(
              'flex w-full items-start gap-3 rounded-lg border p-3 text-left transition-colors',
              'hover:bg-[hsl(var(--accent))]',
              'focus:outline-none focus-visible:ring-2 focus-visible:ring-[hsl(var(--ring))]',
              activeSubTaskId === subTask.id
                ? 'border-[hsl(var(--primary))] bg-[hsl(var(--accent))]'
                : 'border-[hsl(var(--border))]'
            )}
          >
            {/* Step number + icon */}
            <div className="mt-0.5 flex items-center gap-2 shrink-0">
              <span className="text-xs text-[hsl(var(--muted-foreground))]">
                {idx + 1}
              </span>
              {statusIcon(subTask.status)}
            </div>

            <div className="min-w-0 flex-1">
              <div className="flex items-center gap-2">
                <span className="text-sm font-medium">
                  {formatSubTaskType(subTask.taskType)}
                </span>
                <Badge variant={statusBadgeVariant(subTask.status)} className="text-xs">
                  {formatStatus(subTask.status)}
                </Badge>
              </div>
              <div className="mt-1 flex items-center gap-2 text-xs text-[hsl(var(--muted-foreground))]">
                <FormattedDateTime date={subTask.createdAt} />
                {subTask.generatedCommits.length > 0 && (
                  <span>
                    {subTask.generatedCommits.length} commit
                    {subTask.generatedCommits.length !== 1 ? 's' : ''}
                  </span>
                )}
              </div>
            </div>
          </button>
        </li>
      ))}
    </ol>
  );
}
