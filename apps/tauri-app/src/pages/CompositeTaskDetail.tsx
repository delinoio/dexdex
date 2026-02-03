import { useMemo } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { TaskGraph, TaskNodeStatus, type TaskNodeData } from "@/components/task";
import { useTask, useApproveTask, useRejectTask } from "@/hooks/useTasks";
import { CompositeTaskStatus } from "@/api/types";

export function CompositeTaskDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data, isLoading, error } = useTask(id ?? "");
  const approveMutation = useApproveTask();
  const rejectMutation = useRejectTask();

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (error || !data?.compositeTask) {
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

  const task = data.compositeTask;

  // Convert nodeIds to graph node/edge data
  // Note: Currently using nodeIds as placeholder. When the API provides
  // full node data with titles, prompts, and dependencies, this can be enhanced.
  const { graphNodes, graphEdges } = useMemo(() => {
    const nodes: TaskNodeData[] = task.nodeIds.map((nodeId, index) => ({
      id: nodeId,
      title: `Task ${index + 1}`,
      prompt: `Sub-task ${nodeId.slice(0, 8)}...`,
      status: TaskNodeStatus.Pending,
    }));

    // For now, create a simple linear dependency chain as placeholder
    // This will be replaced with actual dependency data when available
    const edges: Array<{ source: string; target: string }> = [];
    for (let i = 0; i < task.nodeIds.length - 1; i++) {
      edges.push({
        source: task.nodeIds[i],
        target: task.nodeIds[i + 1],
      });
    }

    return { graphNodes: nodes, graphEdges: edges };
  }, [task.nodeIds]);

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
              <span>Created {new Date(task.createdAt).toLocaleDateString()}</span>
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
              <TaskGraph
                nodes={graphNodes}
                edges={graphEdges}
                onNodeClick={(nodeId) => {
                  // Navigate to unit task detail when clicking a node
                  navigate(`/unit-tasks/${nodeId}`);
                }}
              />
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
              {task.nodeIds.length === 0 ? (
                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                  No sub-tasks yet. The plan is still being generated.
                </p>
              ) : (
                <div className="space-y-2">
                  {task.nodeIds.map((nodeId, index) => (
                    <div
                      key={nodeId}
                      className="flex items-center justify-between rounded-md border border-[hsl(var(--border))] p-3"
                    >
                      <div className="flex items-center gap-3">
                        <span className="text-sm font-medium">
                          {index + 1}. Task {nodeId.slice(0, 8)}
                        </span>
                        <Badge variant="outline">Pending</Badge>
                      </div>
                      <Button variant="ghost" size="sm">
                        →
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          <div className="rounded-md bg-[hsl(var(--muted))] p-4">
            <p className="text-sm text-[hsl(var(--muted-foreground))]">
              Progress: 0/{task.nodeIds.length} tasks complete
            </p>
            <div className="mt-2 h-2 overflow-hidden rounded-full bg-[hsl(var(--border))]">
              <div
                className="h-full bg-[hsl(var(--primary))]"
                style={{ width: "0%" }}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
