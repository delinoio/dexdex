import { cn } from '@/lib/utils';
import { Badge } from '@/components/ui/Badge';
import { FormattedDateTime } from '@/components/ui/FormattedDateTime';
import type { UnitTask, UnitTaskStatus, ActionType } from '@/api/types';

function statusBadgeVariant(status: UnitTaskStatus): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (status) {
    case 'action_required':
      return 'default';
    case 'in_progress':
      return 'secondary';
    case 'queued':
    case 'blocked':
      return 'outline';
    case 'completed':
      return 'default';
    case 'failed':
    case 'cancelled':
      return 'destructive';
    default:
      return 'outline';
  }
}

function formatStatus(status: UnitTaskStatus): string {
  switch (status) {
    case 'action_required':
      return 'Action Required';
    case 'in_progress':
      return 'In Progress';
    case 'queued':
      return 'Queued';
    case 'blocked':
      return 'Blocked';
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

function formatActionType(actionType: ActionType): string {
  switch (actionType) {
    case 'review_requested':
      return 'Review';
    case 'pr_creation_ready':
      return 'PR Ready';
    case 'plan_approval_required':
      return 'Plan Approval';
    case 'ci_failed':
      return 'CI Failed';
    case 'merge_conflict':
      return 'Merge Conflict';
    case 'security_alert':
      return 'Security Alert';
    case 'user_input_required':
      return 'Input Required';
    default:
      return actionType;
  }
}

interface UnitTaskCardProps {
  task: UnitTask;
  onClick?: () => void;
  className?: string;
}

export function UnitTaskCard({ task, onClick, className }: UnitTaskCardProps) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={cn(
        'w-full rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-4 text-left transition-colors',
        'hover:bg-[hsl(var(--accent))] hover:border-[hsl(var(--primary))/30]',
        'focus:outline-none focus-visible:ring-2 focus-visible:ring-[hsl(var(--ring))]',
        task.status === 'action_required' && 'border-[hsl(var(--primary))/50]',
        className
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="min-w-0 flex-1">
          <p className="truncate text-sm font-medium text-[hsl(var(--foreground))]">
            {task.title}
          </p>
          {task.prompt && (
            <p className="mt-0.5 line-clamp-2 text-xs text-[hsl(var(--muted-foreground))]">
              {task.prompt}
            </p>
          )}
        </div>
        <div className="flex shrink-0 flex-col items-end gap-1">
          <Badge variant={statusBadgeVariant(task.status)}>
            {formatStatus(task.status)}
          </Badge>
        </div>
      </div>

      <div className="mt-2 flex flex-wrap items-center gap-1.5">
        {task.actionTypes.map((actionType) => (
          <Badge key={actionType} variant="outline" className="text-xs">
            {formatActionType(actionType)}
          </Badge>
        ))}
        {task.branchName && (
          <span className="font-mono text-xs text-[hsl(var(--muted-foreground))]">
            {task.branchName}
          </span>
        )}
        <span className="ml-auto text-xs text-[hsl(var(--muted-foreground))]">
          <FormattedDateTime date={task.createdAt} />
        </span>
      </div>
    </button>
  );
}
