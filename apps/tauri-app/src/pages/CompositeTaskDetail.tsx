import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { TaskGraph } from "@/components/task/TaskGraph";
import { useTask, useApproveTask, useRejectTask, useCompositeTaskNodes } from "@/hooks/useTasks";
import type { CompositeTaskNodeWithUnitTask } from "@/api/types";
import { CompositeTaskStatus, UnitTaskStatus } from "@/api/types";
import { useTabTitle } from "@/hooks/useTabNavigation";
        
interface ProgressSectionProps {
  nodes: CompositeTaskNodeWithUnitTask[];
  totalCount: number;
}

function ProgressSection({ nodes, totalCount }: ProgressSectionProps) {
  const completedCount = nodes.filter(
    (n) => n.unitTask?.status === UnitTaskStatus.Done
  ).length;

  const percentage = totalCount > 0 ? (completedCount / totalCount) * 100 : 0;

  return (
    <div className="rounded-md bg-[hsl(var(--muted))] p-4">
      <p className="text-sm text-[hsl(var(--muted-foreground))]">
        Progress: {completedCount}/{totalCount} tasks complete
      </p>
      <div className="mt-2 h-2 overflow-hidden rounded-full bg-[hsl(var(--border))]">
        <div
          className="h-full bg-[hsl(var(--primary))] transition-all duration-300"
          style={{ width: `${percentage}%` }}
        />
      </div>
    </div>
  );
}

export function CompositeTaskDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading, error } = useTask(id ?? "");
  const { data: nodesData, isLoading: nodesLoading, error: nodesError } = useCompositeTaskNodes(id ?? "");
  const approveMutation = useApproveTask();
  const rejectMutation = useRejectTask();

  const task = data?.compositeTask;

  // Set dynamic tab title with task context
  // Must be called before any early returns to follow React's Rules of Hooks
  const tabTitle = task?.title ? `Composite Task: ${task.title}` : "Composite Task";
  useTabTitle(tabTitle);

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (error || !task) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center">
          <p className="text-[hsl(var(--destructive))]">Task not found</p>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => navigate("/")}
          >
            Back to Dashboard
          </Button>
        </div>
      </div>
    );
  }

  const handleApprovePlan = async () => {
    await approveMutation.mutateAsync(task.id);
  };

  const handleRejectPlan = async () => {
    await rejectMutation.mutateAsync({ taskId: task.id });
  };

  const getStatusBadgeVariant = (status: CompositeTaskStatus) => {
    switch (status) {
      case CompositeTaskStatus.Planning:
      case CompositeTaskStatus.InProgress:
        return "default";
      case CompositeTaskStatus.PendingApproval:
        return "secondary";
      case CompositeTaskStatus.Rejected:
        return "destructive";
      default:
        return "outline";
    }
  };

  const formatStatus = (status: CompositeTaskStatus): string => {
    switch (status) {
      case CompositeTaskStatus.Planning:
        return "Planning";
      case CompositeTaskStatus.PendingApproval:
        return "Pending Approval";
      case CompositeTaskStatus.InProgress:
        return "In Progress";
      case CompositeTaskStatus.Done:
        return "Done";
      case CompositeTaskStatus.Rejected:
        return "Rejected";
      default:
        return status;
    }
  };

  const getUnitTaskBadgeVariant = (status?: UnitTaskStatus) => {
    switch (status) {
      case UnitTaskStatus.InProgress:
      case UnitTaskStatus.InReview:
        return "default";
      case UnitTaskStatus.Approved:
      case UnitTaskStatus.PrOpen:
      case UnitTaskStatus.Done:
        return "outline";
      case UnitTaskStatus.Rejected:
        return "destructive";
      case UnitTaskStatus.Unspecified:
      default:
        return "secondary";
    }
  };

  const formatUnitTaskStatus = (status?: UnitTaskStatus): string => {
    switch (status) {
      case UnitTaskStatus.InProgress:
        return "In Progress";
      case UnitTaskStatus.InReview:
        return "In Review";
      case UnitTaskStatus.Approved:
        return "Approved";
      case UnitTaskStatus.PrOpen:
        return "PR Open";
      case UnitTaskStatus.Done:
        return "Done";
      case UnitTaskStatus.Rejected:
        return "Rejected";
      case UnitTaskStatus.Unspecified:
      default:
        return "Pending";
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-2xl font-bold">
              {task.title || "Composite Task"}
            </h1>
            <div className="mt-2 flex items-center gap-3 text-sm text-[hsl(var(--muted-foreground))]">
              <Badge variant={getStatusBadgeVariant(task.status)}>
                {formatStatus(task.status)}
              </Badge>
              <span>Created <FormattedDateTime date={task.createdAt} /></span>
              <span>{task.nodeIds.length} sub-tasks</span>
            </div>
          </div>
          <Button variant="outline" onClick={() => navigate("/")}>
            ← Back
          </Button>
        </div>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-4xl space-y-6">
          <Card>
            <CardHeader>
              <CardTitle>Task Prompt</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="whitespace-pre-wrap text-sm">{task.prompt}</p>
            </CardContent>
          </Card>

          {task.status === CompositeTaskStatus.PendingApproval && (
            <Card className="border-[hsl(var(--primary))]">
              <CardHeader>
                <div className="flex items-center gap-2">
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
                    className="text-[hsl(var(--primary))]"
                  >
                    <circle cx="12" cy="12" r="10" />
                    <path d="M12 16v-4" />
                    <path d="M12 8h.01" />
                  </svg>
                  <CardTitle>Plan Approval Required</CardTitle>
                </div>
                <CardDescription>
                  The AI has generated a plan for this task. Please review and approve to proceed.
                </CardDescription>
              </CardHeader>
              <CardContent className="flex gap-2">
                <Button onClick={handleApprovePlan} disabled={approveMutation.isPending}>
                  {approveMutation.isPending ? "Approving..." : "Approve Plan"}
                </Button>
                <Button variant="outline">View PLAN.yaml</Button>
                <Button
                  variant="destructive"
                  onClick={handleRejectPlan}
                  disabled={rejectMutation.isPending}
                >
                  {rejectMutation.isPending ? "Rejecting..." : "Reject"}
                </Button>
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader>
              <CardTitle>Task Graph</CardTitle>
              <CardDescription>
                Visualization of the task dependencies
              </CardDescription>
            </CardHeader>
            <CardContent>
              {nodesLoading ? (
                <div className="flex h-64 items-center justify-center rounded-md border border-dashed border-[hsl(var(--border))] bg-[hsl(var(--muted))]">
                  <p className="text-sm text-[hsl(var(--muted-foreground))]">
                    Loading task graph...
                  </p>
                </div>
              ) : nodesError ? (
                <div className="flex h-64 items-center justify-center rounded-md border border-dashed border-[hsl(var(--destructive))] bg-[hsl(var(--destructive)/0.1)]">
                  <p className="text-sm text-[hsl(var(--destructive))]">
                    Failed to load task graph. Please try again.
                  </p>
                </div>
              ) : (
                <TaskGraph nodes={nodesData?.nodes || []} />
              )}
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Sub-Tasks</CardTitle>
              <CardDescription>
                Individual tasks in this composite plan
              </CardDescription>
            </CardHeader>
            <CardContent>
              {nodesLoading ? (
                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                  Loading sub-tasks...
                </p>
              ) : !nodesData?.nodes || nodesData.nodes.length === 0 ? (
                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                  No sub-tasks yet. The plan is still being generated.
                </p>
              ) : (
                <div className="space-y-2">
                  {nodesData.nodes.map((nodeWithTask, index) => (
                    <div
                      key={nodeWithTask.node.id}
                      className="flex items-center justify-between rounded-md border border-[hsl(var(--border))] p-3"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-medium">
                          {index + 1}. {nodeWithTask.unitTask?.title || `Task ${nodeWithTask.node.id.slice(0, 8)}`}
                        </span>
                        <Badge variant={getUnitTaskBadgeVariant(nodeWithTask.unitTask?.status)}>
                          {formatUnitTaskStatus(nodeWithTask.unitTask?.status)}
                        </Badge>
                      </div>
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => navigate(`/unit-tasks/${nodeWithTask.unitTask?.id}`)}
                      >
                        →
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <ProgressSection
            nodes={nodesData?.nodes || []}
            totalCount={task.nodeIds.length}
          />
        </div>
      </div>
    </div>
  );
}
