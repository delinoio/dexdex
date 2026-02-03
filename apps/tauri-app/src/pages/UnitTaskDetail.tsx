import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { Textarea } from "@/components/ui/Textarea";
import { useTask, useApproveTask, useRejectTask, useRequestChanges } from "@/hooks/useTasks";
import { UnitTaskStatus } from "@/api/types";
import { useState } from "react";

export function UnitTaskDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [feedback, setFeedback] = useState("");
  const [showFeedback, setShowFeedback] = useState(false);

  const { data, isLoading, error } = useTask(id ?? "");
  const approveMutation = useApproveTask();
  const rejectMutation = useRejectTask();
  const requestChangesMutation = useRequestChanges();

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading...</div>
      </div>
    );
  }

  if (error || !data?.unitTask) {
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

  const task = data.unitTask;

  const handleApprove = async () => {
    await approveMutation.mutateAsync(task.id);
  };

  const handleReject = async () => {
    await rejectMutation.mutateAsync({ taskId: task.id, reason: feedback || undefined });
  };

  const handleRequestChanges = async () => {
    if (!feedback) return;
    await requestChangesMutation.mutateAsync({ taskId: task.id, feedback });
    setFeedback("");
    setShowFeedback(false);
  };

  const getStatusBadgeVariant = (status: UnitTaskStatus) => {
    switch (status) {
      case UnitTaskStatus.InProgress:
        return "default";
      case UnitTaskStatus.InReview:
        return "secondary";
      case UnitTaskStatus.Rejected:
        return "destructive";
      default:
        return "outline";
    }
  };

  const formatStatus = (status: UnitTaskStatus): string => {
    switch (status) {
      case UnitTaskStatus.InProgress:
        return "In Progress";
      case UnitTaskStatus.InReview:
        return "In Review";
      case UnitTaskStatus.PrOpen:
        return "PR Open";
      case UnitTaskStatus.Done:
        return "Done";
      case UnitTaskStatus.Rejected:
        return "Rejected";
      case UnitTaskStatus.Approved:
        return "Approved";
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
              {task.title || "Unit Task"}
            </h1>
            <div className="mt-2 flex items-center gap-3 text-sm text-[hsl(var(--muted-foreground))]">
              <Badge variant={getStatusBadgeVariant(task.status)}>
                {formatStatus(task.status)}
              </Badge>
              <span>Created <FormattedDateTime date={task.createdAt} /></span>
              {task.branchName && (
                <span className="font-mono text-xs">
                  {task.branchName}
                </span>
              )}
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

          {task.status === UnitTaskStatus.InReview && (
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
                  <CardTitle>Review Required</CardTitle>
                </div>
                <CardDescription>
                  The AI agent has completed its work. Please review the changes.
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="flex gap-2">
                  <Button onClick={handleApprove} disabled={approveMutation.isPending}>
                    {approveMutation.isPending ? "Approving..." : "Approve"}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setShowFeedback(true)}
                  >
                    Request Changes
                  </Button>
                  <Button
                    variant="destructive"
                    onClick={handleReject}
                    disabled={rejectMutation.isPending}
                  >
                    {rejectMutation.isPending ? "Rejecting..." : "Reject"}
                  </Button>
                </div>

                {showFeedback && (
                  <div className="space-y-2">
                    <Textarea
                      placeholder="Describe the changes you'd like..."
                      value={feedback}
                      onChange={(e) => setFeedback(e.target.value)}
                      rows={4}
                    />
                    <div className="flex gap-2">
                      <Button
                        onClick={handleRequestChanges}
                        disabled={!feedback || requestChangesMutation.isPending}
                      >
                        {requestChangesMutation.isPending
                          ? "Sending..."
                          : "Send Feedback"}
                      </Button>
                      <Button
                        variant="outline"
                        onClick={() => {
                          setShowFeedback(false);
                          setFeedback("");
                        }}
                      >
                        Cancel
                      </Button>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          )}

          <Card>
            <CardHeader>
              <CardTitle>Agent Session Log</CardTitle>
              <CardDescription>
                Output from the AI coding agent
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="max-h-96 overflow-y-auto rounded-md bg-[hsl(var(--muted))] p-4 font-mono text-xs">
                <p className="text-[hsl(var(--muted-foreground))]">
                  [Awaiting agent session logs...]
                </p>
              </div>
            </CardContent>
          </Card>

          {task.status !== UnitTaskStatus.InProgress &&
            task.status !== UnitTaskStatus.Unspecified && (
              <div className="flex gap-2">
                <Button variant="outline">View Diff</Button>
                <Button variant="outline">Open in Editor</Button>
              </div>
            )}
        </div>
      </div>
    </div>
  );
}
