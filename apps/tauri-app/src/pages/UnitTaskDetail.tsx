import { useParams, useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Badge } from "@/components/ui/Badge";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { Textarea } from "@/components/ui/Textarea";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/Dialog";
import { AgentLogViewer, StaticSessionLogViewer } from "@/components/task/AgentLogViewer";
import { TokenUsageCard, aggregateTokenUsage } from "@/components/task/TokenUsageCard";
import { DiffViewer, DiffFileList, type DiffFile } from "@/components/review/DiffViewer";
import { useTask, useApproveTask, useRejectTask, useRequestChanges, useCancelTask, useDeleteTask, useDismissApproval, useCreatePr, useCommitToLocal } from "@/hooks/useTasks";
import { useMode } from "@/hooks/useMode";
import { useTaskDetailShortcuts } from "@/hooks/useReviewShortcuts";
import { useTabTitle } from "@/hooks/useTabNavigation";
import { useTaskLogs } from "@/hooks/useTaskLogs";
import { useReviewComments } from "@/hooks/useReviewComments";
import { parseUnifiedDiff } from "@/lib/parseDiff";
import type { TokenUsage, SessionEndEvent } from "@/api/types";
import { UnitTaskStatus } from "@/api/types";
import { open } from "@tauri-apps/plugin-dialog";
import { useState, useCallback, useMemo, useEffect, useRef } from "react";

export function UnitTaskDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const [showLog, setShowLog] = useState(true);
  // Tracks which session IDs are collapsed (all default to expanded)
  const [collapsedSessions, setCollapsedSessions] = useState<Set<string>>(new Set());
  const [showDiff, setShowDiff] = useState(false);
  const [selectedDiffFile, setSelectedDiffFile] = useState<string | undefined>();
  const [showRequestChangesDialog, setShowRequestChangesDialog] = useState(false);
  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [extraComment, setExtraComment] = useState("");

  const { data, isLoading, error } = useTask(id ?? "");
  const approveMutation = useApproveTask();
  const rejectMutation = useRejectTask();
  const requestChangesMutation = useRequestChanges();
  const cancelMutation = useCancelTask();
  const deleteMutation = useDeleteTask();
  const dismissApprovalMutation = useDismissApproval();
  const createPrMutation = useCreatePr();
  const commitToLocalMutation = useCommitToLocal();
  const { data: mode } = useMode();

  const task = data?.unitTask;
  const isLocalMode = mode === "local";

  // Manage inline review comments (local state)
  const {
    comments: reviewComments,
    addComment: addReviewComment,
    updateComment: updateReviewComment,
    deleteComment: deleteReviewComment,
    clearAll: clearReviewComments,
    getCommentsForFile,
    commentCount: reviewCommentCount,
  } = useReviewComments({ taskId: id ?? "" });

  // Whether the user has written any review comments
  const hasReviewComments = reviewCommentCount > 0;

  // Fetch task logs to extract token usage from session_end events
  const { events, sessions } = useTaskLogs({
    taskId: task?.id ?? "",
    agentTaskId: task?.agentTaskId ?? "",
    taskStatus: task?.status ?? UnitTaskStatus.Unspecified,
    enabled: !!task?.agentTaskId,
  });

  // Auto-show the diff when the task is first loaded in InReview status
  // (e.g. user navigates to a task that is already awaiting review).
  const hasInitializedDiffRef = useRef(false);
  useEffect(() => {
    if (
      !hasInitializedDiffRef.current &&
      task?.status === UnitTaskStatus.InReview &&
      task?.gitPatch
    ) {
      hasInitializedDiffRef.current = true;
      setShowDiff(true);
    }
  }, [task?.status, task?.gitPatch]);

  // Auto-collapse the session log and auto-show the diff when the task
  // transitions out of InProgress. We track the previous status so that only a
  // genuine transition triggers the change (e.g. going from InProgress ->
  // InReview), rather than toggling every time the component re-renders with a
  // non-InProgress status.
  const prevTaskStatusRef = useRef(task?.status);
  useEffect(() => {
    const prevStatus = prevTaskStatusRef.current;
    const currentStatus = task?.status;
    prevTaskStatusRef.current = currentStatus;

    if (
      prevStatus === UnitTaskStatus.InProgress &&
      currentStatus !== undefined &&
      currentStatus !== UnitTaskStatus.InProgress
    ) {
      setShowLog(false);
      // Auto-show the diff when transitioning to review (or any post-InProgress state)
      setShowDiff(true);
    }
  }, [task?.status]);

  // Extract token usage from session_end events
  const tokenUsage = useMemo<TokenUsage | null>(() => {
    const sessionEndEvents = events
      .filter((e): e is { id: number | string; timestamp: string; event: SessionEndEvent } =>
        e.event.type === "session_end"
      )
      .map((e) => e.event.token_usage)
      .filter((tu): tu is TokenUsage => tu !== null && tu !== undefined);

    return aggregateTokenUsage(sessionEndEvents);
  }, [events]);

  // Set dynamic tab title with task context
  const tabTitle = task?.title ? `Task: ${task.title}` : "Task";
  useTabTitle(tabTitle);

  // Keyboard shortcut handlers
  const handleShortcutApprove = useCallback(async () => {
    if (task?.status === UnitTaskStatus.InReview && !approveMutation.isPending && !hasReviewComments) {
      await approveMutation.mutateAsync(task.id);
    }
  }, [task, approveMutation, hasReviewComments]);

  const handleShortcutDeny = useCallback(async () => {
    if (task?.status === UnitTaskStatus.InReview && !rejectMutation.isPending) {
      await rejectMutation.mutateAsync({ taskId: task.id });
    }
  }, [task, rejectMutation]);

  const handleToggleLog = useCallback(() => {
    setShowLog((prev) => !prev);
  }, []);

  const handleStop = useCallback(async () => {
    if (task?.id && !cancelMutation.isPending) {
      try {
        await cancelMutation.mutateAsync(task.id);
      } catch (error) {
        console.error("Failed to cancel task:", error);
        // Error is handled by React Query, user will see the mutation state
      }
    }
  }, [task?.id, cancelMutation]);

  const handleDelete = useCallback(async () => {
    if (task?.id && !deleteMutation.isPending) {
      try {
        await deleteMutation.mutateAsync(task.id);
        navigate("/");
      } catch (error) {
        console.error("Failed to delete task:", error);
      }
    }
  }, [task?.id, deleteMutation, navigate]);

  // Parse git patch into structured diff files for the DiffViewer
  const diffFiles = useMemo<DiffFile[]>(() => {
    if (!task?.gitPatch) return [];
    return parseUnifiedDiff(task.gitPatch);
  }, [task?.gitPatch]);

  // Handle View Diff button click
  const handleViewDiff = useCallback(() => {
    setShowDiff((prev) => {
      if (!prev && diffFiles.length > 0 && !selectedDiffFile) {
        setSelectedDiffFile(diffFiles[0].filePath);
      }
      return !prev;
    });
  }, [diffFiles, selectedDiffFile]);

  // Register keyboard shortcuts
  useTaskDetailShortcuts({
    onApprove: handleShortcutApprove,
    onDeny: handleShortcutDeny,
    onToggleLog: handleToggleLog,
    onStop: handleStop,
    enabled: !!task,
  });

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

  const handleApprove = async () => {
    if (hasReviewComments) {
      return;
    }
    await approveMutation.mutateAsync(task.id);
  };

  const handleReject = async () => {
    await rejectMutation.mutateAsync({ taskId: task.id });
  };

  // Build the full feedback string from inline review comments + optional extra comment.
  // Uses filename:line format so the AI coding agent can locate each comment precisely.
  const buildFeedbackFromComments = (extra: string): string => {
    const parts: string[] = [];

    if (reviewCommentCount > 0) {
      parts.push("## Inline Review Comments\n");
      // Group comments by file
      const commentsByFile = new Map<string, typeof reviewComments>();
      for (const comment of reviewComments) {
        const existing = commentsByFile.get(comment.filePath) ?? [];
        existing.push(comment);
        commentsByFile.set(comment.filePath, existing);
      }
      for (const [filePath, fileComments] of commentsByFile) {
        // Sort comments by line number within each file
        const sorted = [...fileComments].sort((a, b) => a.lineNumber - b.lineNumber);
        for (const comment of sorted) {
          parts.push(`- ${filePath}:${comment.lineNumber}: ${comment.content}`);
        }
      }
      parts.push("");
    }

    if (extra.trim()) {
      parts.push("## Additional Comments\n");
      parts.push(extra.trim());
    }

    return parts.join("\n");
  };

  const handleRequestChanges = async () => {
    const feedback = buildFeedbackFromComments(extraComment);
    if (!feedback.trim()) return;
    await requestChangesMutation.mutateAsync({ taskId: task.id, feedback });
    clearReviewComments();
    setExtraComment("");
    setShowRequestChangesDialog(false);
  };

  const handleDismissApproval = async () => {
    await dismissApprovalMutation.mutateAsync(task.id);
  };

  const handleCreatePr = async () => {
    await createPrMutation.mutateAsync(task.id);
  };

  const handleCommitToLocal = async () => {
    const selectedPath = await open({
      directory: true,
      multiple: false,
      title: "Select local git repository",
    });
    if (!selectedPath) return;
    await commitToLocalMutation.mutateAsync({ taskId: task.id, localPath: selectedPath });
  };

  const getStatusBadgeVariant = (status: UnitTaskStatus) => {
    switch (status) {
      case UnitTaskStatus.InProgress:
        return "default";
      case UnitTaskStatus.InReview:
        return "secondary";
      case UnitTaskStatus.Approved:
      case UnitTaskStatus.PrOpen:
      case UnitTaskStatus.Done:
        return "default";
      case UnitTaskStatus.Rejected:
      case UnitTaskStatus.Failed:
      case UnitTaskStatus.Cancelled:
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
      case UnitTaskStatus.Failed:
        return "Failed";
      case UnitTaskStatus.Cancelled:
        return "Cancelled";
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
          <div className="flex gap-2">
            <Button
              variant="ghost"
              className="text-[hsl(var(--destructive))]"
              onClick={() => setShowDeleteDialog(true)}
            >
              Delete
            </Button>
            <Button variant="outline" onClick={() => navigate("/")}>
              ← Back
            </Button>
          </div>
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

          {tokenUsage && (
            <TokenUsageCard tokenUsage={tokenUsage} />
          )}

          {task.status === UnitTaskStatus.InProgress && (
            <Card className="border-[hsl(var(--warning))]">
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
                    className="animate-spin text-[hsl(var(--primary))]"
                  >
                    <path d="M21 12a9 9 0 1 1-6.219-8.56" />
                  </svg>
                  <CardTitle>Agent Running</CardTitle>
                </div>
                <CardDescription>
                  The AI agent is working on your task. You can stop it at any time.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <Button
                  variant="destructive"
                  onClick={handleStop}
                  disabled={cancelMutation.isPending}
                  title="Stop execution (S)"
                >
                  {cancelMutation.isPending ? "Stopping..." : "Stop Agent"}
                </Button>
              </CardContent>
            </Card>
          )}

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
                <div className="flex flex-col gap-2">
                  <div className="flex gap-2">
                    <Button
                      onClick={handleApprove}
                      disabled={approveMutation.isPending || hasReviewComments}
                    >
                      {approveMutation.isPending ? "Approving..." : "Approve"}
                    </Button>
                    <Button
                      variant="outline"
                      onClick={() => setShowRequestChangesDialog(true)}
                    >
                      Request Changes{hasReviewComments ? ` (${reviewCommentCount})` : ""}
                    </Button>
                    <Button
                      variant="destructive"
                      onClick={handleReject}
                      disabled={rejectMutation.isPending}
                    >
                      {rejectMutation.isPending ? "Rejecting..." : "Reject"}
                    </Button>
                  </div>
                  {hasReviewComments && (
                    <p className="text-sm text-[hsl(var(--muted-foreground))]">
                      You have {reviewCommentCount} review comment{reviewCommentCount !== 1 ? "s" : ""}. Please submit them via &quot;Request Changes&quot; before approving.
                    </p>
                  )}
                </div>

                {/* Request Changes dialog with optional extra comment */}
                <Dialog open={showRequestChangesDialog} onOpenChange={setShowRequestChangesDialog}>
                  <DialogContent>
                    <DialogHeader>
                      <DialogTitle>Request Changes</DialogTitle>
                      <DialogDescription>
                        {hasReviewComments
                          ? `Your ${reviewCommentCount} inline review comment${reviewCommentCount !== 1 ? "s" : ""} will be sent to the AI agent. You can add an optional extra comment below.`
                          : "Describe the changes you'd like the AI agent to make."}
                      </DialogDescription>
                    </DialogHeader>
                    <Textarea
                      placeholder={hasReviewComments ? "Optional additional comments..." : "Describe the changes you'd like..."}
                      value={extraComment}
                      onChange={(e) => setExtraComment(e.target.value)}
                      onKeyDown={(e) => {
                        if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
                          e.preventDefault();
                          if (hasReviewComments || extraComment.trim()) {
                            handleRequestChanges();
                          }
                        }
                      }}
                      rows={4}
                    />
                    <DialogFooter>
                      <Button
                        variant="outline"
                        onClick={() => {
                          setShowRequestChangesDialog(false);
                          setExtraComment("");
                        }}
                      >
                        Cancel
                      </Button>
                      <Button
                        onClick={handleRequestChanges}
                        disabled={
                          (!hasReviewComments && !extraComment.trim()) ||
                          requestChangesMutation.isPending
                        }
                      >
                        {requestChangesMutation.isPending
                          ? "Sending..."
                          : "Submit"}
                      </Button>
                    </DialogFooter>
                  </DialogContent>
                </Dialog>
              </CardContent>
            </Card>
          )}

          {task.status === UnitTaskStatus.PrOpen && (
            <Card className="border-[hsl(var(--success))]">
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
                    className="text-[hsl(var(--success))]"
                  >
                    <circle cx="18" cy="18" r="3" />
                    <circle cx="6" cy="6" r="3" />
                    <path d="M13 6h3a2 2 0 0 1 2 2v7" />
                    <line x1="6" y1="9" x2="6" y2="21" />
                  </svg>
                  <CardTitle>Pull Request Created</CardTitle>
                </div>
                <CardDescription>
                  A pull request has been created for this task.
                </CardDescription>
              </CardHeader>
              {task.linkedPrUrl && (
                <CardContent>
                  <a
                    href={task.linkedPrUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-sm text-[hsl(var(--primary))] underline"
                  >
                    {task.linkedPrUrl}
                  </a>
                </CardContent>
              )}
            </Card>
          )}

          {task.status === UnitTaskStatus.Done && (
            <Card className="border-[hsl(var(--success))]">
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
                    className="text-[hsl(var(--success))]"
                  >
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                    <polyline points="22 4 12 14.01 9 11.01" />
                  </svg>
                  <CardTitle>Completed</CardTitle>
                </div>
                <CardDescription>
                  This task has been completed and the changes have been applied.
                </CardDescription>
              </CardHeader>
            </Card>
          )}

          {task.status === UnitTaskStatus.Approved && (
            <Card className="border-[hsl(var(--success))]">
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
                    className="text-[hsl(var(--success))]"
                  >
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                    <polyline points="22 4 12 14.01 9 11.01" />
                  </svg>
                  <CardTitle>Approved</CardTitle>
                </div>
                <CardDescription>
                  This task has been approved. Choose how to apply the changes.
                </CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex gap-2">
                  <Button
                    onClick={handleCreatePr}
                    disabled={createPrMutation.isPending}
                  >
                    {createPrMutation.isPending ? "Creating PR..." : "Create PR"}
                  </Button>
                  {isLocalMode && (
                    <Button
                      variant="outline"
                      onClick={handleCommitToLocal}
                      disabled={commitToLocalMutation.isPending}
                    >
                      {commitToLocalMutation.isPending ? "Committing..." : "Commit to Local"}
                    </Button>
                  )}
                  <Button
                    variant="ghost"
                    onClick={handleDismissApproval}
                    disabled={dismissApprovalMutation.isPending}
                  >
                    {dismissApprovalMutation.isPending ? "Dismissing..." : "Dismiss Approval"}
                  </Button>
                </div>
              </CardContent>
            </Card>
          )}

          {task.status !== UnitTaskStatus.InProgress &&
            task.status !== UnitTaskStatus.Unspecified && (
              <div className="space-y-4">
                <div className="flex gap-2">
                  {task.gitPatch && (
                    <Button variant="outline" onClick={handleViewDiff}>
                      {showDiff ? "Hide Diff" : "View Diff"}
                    </Button>
                  )}
                </div>

                {showDiff && diffFiles.length > 0 && (
                  <Card>
                    <CardHeader>
                      <CardTitle>Changes</CardTitle>
                      <CardDescription>
                        {diffFiles.length} file{diffFiles.length !== 1 ? "s" : ""} changed
                      </CardDescription>
                    </CardHeader>
                    <CardContent>
                      <div className="flex gap-4">
                        {diffFiles.length > 1 && (
                          <div className="w-48 shrink-0 border-r border-[hsl(var(--border))] pr-4">
                            <DiffFileList
                              files={diffFiles}
                              selectedFilePath={selectedDiffFile}
                              onSelectFile={setSelectedDiffFile}
                              viewedFiles={new Set()}
                              commentCounts={Object.fromEntries(
                                diffFiles.map((f) => [
                                  f.filePath,
                                  getCommentsForFile(f.filePath).length,
                                ])
                              )}
                            />
                          </div>
                        )}
                        <div className="min-w-0 flex-1">
                          {(selectedDiffFile
                            ? diffFiles.filter((f) => f.filePath === selectedDiffFile)
                            : diffFiles
                          ).map((file) => (
                            <DiffViewer
                              key={file.filePath}
                              file={file}
                              comments={getCommentsForFile(file.filePath)}
                              onAddComment={addReviewComment}
                              onEditComment={updateReviewComment}
                              onDeleteComment={deleteReviewComment}
                              className="mb-4"
                            />
                          ))}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                )}
              </div>
            )}

          {/* Render separate log sections per agent session.
              Previous (completed) sessions use a static viewer; the latest
              session uses the real-time streaming AgentLogViewer. */}
          {sessions.length > 1 ? (
            sessions.map((session, idx) => {
              const isLatest = idx === sessions.length - 1;
              const isCollapsed = collapsedSessions.has(session.sessionId);
              return (
                <Card key={session.sessionId}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <div>
                        <CardTitle>{session.label}</CardTitle>
                        <CardDescription>
                          {isLatest
                            ? "Active agent session"
                            : `Completed session`}
                        </CardDescription>
                      </div>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() =>
                          setCollapsedSessions((prev) => {
                            const next = new Set(prev);
                            if (next.has(session.sessionId)) {
                              next.delete(session.sessionId);
                            } else {
                              next.add(session.sessionId);
                            }
                            return next;
                          })
                        }
                      >
                        {isCollapsed ? "Show" : "Hide"}
                      </Button>
                    </div>
                  </CardHeader>
                  {!isCollapsed && (
                    <CardContent>
                      {isLatest ? (
                        <AgentLogViewer
                          taskId={task.id}
                          agentTaskId={task.agentTaskId}
                          taskStatus={task.status}
                          className="min-h-64 max-h-[500px]"
                        />
                      ) : (
                        <StaticSessionLogViewer
                          events={session.events}
                          className="min-h-32 max-h-[500px]"
                        />
                      )}
                    </CardContent>
                  )}
                </Card>
              );
            })
          ) : (
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle>Agent Session Log</CardTitle>
                    <CardDescription>
                      Output from the AI coding agent
                    </CardDescription>
                  </div>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleToggleLog}
                    title="Toggle log visibility (L)"
                  >
                    {showLog ? "Hide" : "Show"}
                  </Button>
                </div>
              </CardHeader>
              {showLog && (
                <CardContent>
                  <AgentLogViewer
                    taskId={task.id}
                    agentTaskId={task.agentTaskId}
                    taskStatus={task.status}
                    className="min-h-64 max-h-[500px]"
                  />
                </CardContent>
              )}
            </Card>
          )}
        </div>
      </div>

      {/* Delete confirmation dialog */}
      <Dialog open={showDeleteDialog} onOpenChange={setShowDeleteDialog}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Delete Task</DialogTitle>
            <DialogDescription>
              Are you sure you want to delete this task? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setShowDeleteDialog(false)}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={deleteMutation.isPending}
            >
              {deleteMutation.isPending ? "Deleting..." : "Delete"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
