import { useState, useCallback } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/Button';
import { Badge } from '@/components/ui/Badge';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { FormattedDateTime } from '@/components/ui/FormattedDateTime';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/Dialog';
import { Textarea } from '@/components/ui/Textarea';
import { SubTaskTimeline } from '@/components/task/SubTaskTimeline';
import { SessionLogViewer } from '@/components/task/SessionLogViewer';
import { PlanDecisionPanel } from '@/components/task/PlanDecisionPanel';
import {
  useTask,
  useDeleteTask,
  useStopTask,
  useApproveSubTask,
  useRequestChanges,
  useCreatePr,
} from '@/api/hooks/useTasks';
import { useSubTasks } from '@/api/hooks/useSubTasks';
import { useAgentSessions } from '@/api/hooks/useSessions';
import type {
  UnitTask,
  UnitTaskStatus,
  SubTask,
  ActionType,
} from '@/api/types';

function statusBadgeVariant(
  status: UnitTaskStatus
): 'default' | 'secondary' | 'destructive' | 'outline' {
  switch (status) {
    case 'action_required':
      return 'default';
    case 'in_progress':
      return 'secondary';
    case 'completed':
      return 'default';
    case 'failed':
    case 'cancelled':
      return 'destructive';
    default:
      return 'outline';
  }
}

function formatTaskStatus(status: UnitTaskStatus): string {
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

function ActionPanel({
  task,
  activeSubTask,
  onRefresh,
}: {
  task: UnitTask;
  activeSubTask?: SubTask;
  onRefresh: () => void;
}) {
  const approveSubTask = useApproveSubTask();
  const requestChanges = useRequestChanges();
  const createPr = useCreatePr();
  const [showRequestChangesDialog, setShowRequestChangesDialog] = useState(false);
  const [feedback, setFeedback] = useState('');

  const hasAction = (actionType: ActionType) => task.actionTypes.includes(actionType);

  const handleApprove = async () => {
    if (!activeSubTask) return;
    await approveSubTask.mutateAsync(activeSubTask.id);
    onRefresh();
  };

  const handleRequestChanges = async () => {
    if (!activeSubTask || !feedback.trim()) return;
    await requestChanges.mutateAsync({ subTaskId: activeSubTask.id, feedback: feedback.trim() });
    setFeedback('');
    setShowRequestChangesDialog(false);
    onRefresh();
  };

  const handleCreatePr = async () => {
    await createPr.mutateAsync(task.id);
    onRefresh();
  };

  if (hasAction('plan_approval_required') && activeSubTask) {
    return (
      <PlanDecisionPanel
        subTask={activeSubTask}
        onDecision={onRefresh}
      />
    );
  }

  if (hasAction('review_requested') && activeSubTask) {
    return (
      <>
        <Card className="border-[hsl(var(--primary))/50]">
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
          <CardContent>
            <div className="flex gap-2">
              <Button onClick={handleApprove} disabled={approveSubTask.isPending}>
                {approveSubTask.isPending ? 'Approving...' : 'Approve'}
              </Button>
              <Button variant="outline" onClick={() => setShowRequestChangesDialog(true)}>
                Request Changes
              </Button>
            </div>
          </CardContent>
        </Card>

        <Dialog open={showRequestChangesDialog} onOpenChange={setShowRequestChangesDialog}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Request Changes</DialogTitle>
              <DialogDescription>
                Describe the changes you&apos;d like the AI agent to make.
              </DialogDescription>
            </DialogHeader>
            <Textarea
              placeholder="Describe the changes you'd like..."
              value={feedback}
              onChange={(e) => setFeedback(e.target.value)}
              onKeyDown={(e) => {
                if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                  e.preventDefault();
                  handleRequestChanges();
                }
              }}
              rows={4}
            />
            <DialogFooter>
              <Button
                variant="outline"
                onClick={() => {
                  setShowRequestChangesDialog(false);
                  setFeedback('');
                }}
              >
                Cancel
              </Button>
              <Button
                onClick={handleRequestChanges}
                disabled={!feedback.trim() || requestChanges.isPending}
              >
                {requestChanges.isPending ? 'Sending...' : 'Submit'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </>
    );
  }

  if (hasAction('pr_creation_ready')) {
    return (
      <Card className="border-green-500/50">
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
              className="text-green-500"
            >
              <circle cx="18" cy="18" r="3" />
              <circle cx="6" cy="6" r="3" />
              <path d="M13 6h3a2 2 0 0 1 2 2v7" />
              <line x1="6" y1="9" x2="6" y2="21" />
            </svg>
            <CardTitle>Ready to Create PR</CardTitle>
          </div>
          <CardDescription>
            The AI agent has completed its work and is ready to create a pull request.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <Button onClick={handleCreatePr} disabled={createPr.isPending}>
            {createPr.isPending ? 'Creating PR...' : 'Create PR'}
          </Button>
        </CardContent>
      </Card>
    );
  }

  return null;
}

function ActiveSessionPanel({ subTask }: { subTask: SubTask }) {
  const { data } = useAgentSessions(subTask.id);
  const sessions = data?.sessions ?? [];
  const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

  if (sessions.length === 0) {
    return (
      <div className="text-sm text-[hsl(var(--muted-foreground))]">No agent sessions yet.</div>
    );
  }

  const activeSession = sessions.find((s) =>
    ['starting', 'running', 'waiting_for_input'].includes(s.status)
  );
  const displaySessionId = selectedSessionId ?? activeSession?.id ?? sessions[sessions.length - 1]?.id;
  const displaySession = sessions.find((s) => s.id === displaySessionId);

  return (
    <div className="space-y-3">
      {sessions.length > 1 && (
        <div className="flex flex-wrap gap-2">
          {sessions.map((s, idx) => (
            <Button
              key={s.id}
              variant={s.id === displaySessionId ? 'default' : 'outline'}
              size="sm"
              onClick={() => setSelectedSessionId(s.id)}
            >
              Session {idx + 1}
            </Button>
          ))}
        </div>
      )}

      {displaySession && (
        <SessionLogViewer
          sessionId={displaySession.id}
          sessionStatus={displaySession.status}
          className="min-h-64 max-h-[500px]"
        />
      )}
    </div>
  );
}

export function UnitTaskDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();

  const { data: taskData, isLoading, error, refetch } = useTask(id ?? '');
  const { data: subTasksData } = useSubTasks(id ?? '');
  const deleteTask = useDeleteTask();
  const stopTask = useStopTask();

  const [showDeleteDialog, setShowDeleteDialog] = useState(false);
  const [selectedSubTaskId, setSelectedSubTaskId] = useState<string | null>(null);

  const task = taskData?.task;
  const subTasks = subTasksData?.subTasks ?? [];

  // The "active" subtask is either the selected one, the in-progress one, or the last one
  const activeSubTask = selectedSubTaskId
    ? subTasks.find((s) => s.id === selectedSubTaskId)
    : subTasks.find((s) =>
        ['in_progress', 'waiting_for_plan_approval', 'waiting_for_user_input'].includes(s.status)
      ) ?? subTasks[subTasks.length - 1];

  const handleDelete = useCallback(async () => {
    if (!task) return;
    await deleteTask.mutateAsync(task.id);
    navigate('/');
  }, [task, deleteTask, navigate]);

  const handleStop = useCallback(async () => {
    if (!task) return;
    await stopTask.mutateAsync(task.id);
    refetch();
  }, [task, stopTask, refetch]);

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
          <Button variant="outline" className="mt-4" onClick={() => navigate('/')}>
            Back to Tasks
          </Button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col">
      {/* Header */}
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <div className="flex items-start justify-between">
          <div>
            <h1 className="text-2xl font-bold">{task.title}</h1>
            <div className="mt-2 flex items-center gap-3 text-sm text-[hsl(var(--muted-foreground))]">
              <Badge variant={statusBadgeVariant(task.status)}>
                {formatTaskStatus(task.status)}
              </Badge>
              <span>
                Created <FormattedDateTime date={task.createdAt} />
              </span>
              {task.branchName && (
                <span className="font-mono text-xs">{task.branchName}</span>
              )}
            </div>
          </div>
          <div className="flex gap-2">
            {task.status === 'in_progress' && (
              <Button
                variant="destructive"
                size="sm"
                onClick={handleStop}
                disabled={stopTask.isPending}
              >
                {stopTask.isPending ? 'Stopping...' : 'Stop'}
              </Button>
            )}
            <Button
              variant="ghost"
              size="sm"
              className="text-[hsl(var(--destructive))]"
              onClick={() => setShowDeleteDialog(true)}
            >
              Delete
            </Button>
            <Button variant="outline" size="sm" onClick={() => navigate('/')}>
              ← Back
            </Button>
          </div>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-4xl space-y-6">
          {/* Task prompt */}
          <Card>
            <CardHeader>
              <CardTitle>Task Prompt</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="whitespace-pre-wrap text-sm">{task.prompt}</p>
            </CardContent>
          </Card>

          {/* Action panel (if action required) */}
          {task.status === 'action_required' && (
            <ActionPanel
              task={task}
              activeSubTask={activeSubTask}
              onRefresh={() => refetch()}
            />
          )}

          {/* Completed state */}
          {task.status === 'completed' && (
            <Card className="border-green-500/50">
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
                    className="text-green-500"
                  >
                    <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14" />
                    <polyline points="22 4 12 14.01 9 11.01" />
                  </svg>
                  <CardTitle>Completed</CardTitle>
                </div>
                <CardDescription>
                  This task has been completed.
                </CardDescription>
              </CardHeader>
            </Card>
          )}

          {/* Failed state */}
          {task.status === 'failed' && (
            <Card className="border-[hsl(var(--destructive))/50]">
              <CardHeader>
                <CardTitle className="text-[hsl(var(--destructive))]">Task Failed</CardTitle>
                <CardDescription>
                  This task has failed. You can review the logs below for details.
                </CardDescription>
              </CardHeader>
            </Card>
          )}

          {/* SubTask timeline */}
          {subTasks.length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle>Progress</CardTitle>
                <CardDescription>
                  {subTasks.length} step{subTasks.length !== 1 ? 's' : ''}
                </CardDescription>
              </CardHeader>
              <CardContent>
                <SubTaskTimeline
                  subTasks={subTasks}
                  activeSubTaskId={activeSubTask?.id}
                  onSelectSubTask={setSelectedSubTaskId}
                />
              </CardContent>
            </Card>
          )}

          {/* Session log for selected/active subtask */}
          {activeSubTask && (
            <Card>
              <CardHeader>
                <CardTitle>Agent Session Log</CardTitle>
                <CardDescription>
                  Output from the AI coding agent
                </CardDescription>
              </CardHeader>
              <CardContent>
                <ActiveSessionPanel subTask={activeSubTask} />
              </CardContent>
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
            <Button variant="outline" onClick={() => setShowDeleteDialog(false)}>
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={handleDelete}
              disabled={deleteTask.isPending}
            >
              {deleteTask.isPending ? 'Deleting...' : 'Delete'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
