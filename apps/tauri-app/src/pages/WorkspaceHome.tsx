import { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { Button } from '@/components/ui/Button';
import { useTasks, useCreateTask } from '@/api/hooks/useTasks';
import { useUiStore } from '@/stores/uiStore';
import { UnitTaskCard } from '@/components/task/UnitTaskCard';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter } from '@/components/ui/Dialog';
import { Input } from '@/components/ui/Input';
import { Textarea } from '@/components/ui/Textarea';
import type { UnitTask, UnitTaskStatus } from '@/api/types';

function TaskSection({
  title,
  tasks,
  onTaskClick,
}: {
  title: string;
  tasks: UnitTask[];
  onTaskClick: (id: string) => void;
}) {
  if (tasks.length === 0) return null;

  return (
    <div className="space-y-2">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-[hsl(var(--muted-foreground))]">
        {title} <span className="ml-1 text-[hsl(var(--muted-foreground))]">({tasks.length})</span>
      </h2>
      <div className="space-y-2">
        {tasks.map((task) => (
          <UnitTaskCard key={task.id} task={task} onClick={() => onTaskClick(task.id)} />
        ))}
      </div>
    </div>
  );
}

export function WorkspaceHome() {
  const navigate = useNavigate();
  const currentWorkspaceId = useUiStore((s) => s.currentWorkspaceId);
  const { data, isLoading, error, refetch } = useTasks(
    currentWorkspaceId ? { workspaceId: currentWorkspaceId } : {}
  );
  const createTask = useCreateTask();
  const [showNewTask, setShowNewTask] = useState(false);
  const [newTaskTitle, setNewTaskTitle] = useState('');
  const [newTaskPrompt, setNewTaskPrompt] = useState('');

  const handleTaskClick = (id: string) => {
    navigate(`/tasks/${id}`);
  };

  const handleCreateTask = async () => {
    if (!newTaskTitle.trim() || !newTaskPrompt.trim()) return;
    if (!currentWorkspaceId) return;

    await createTask.mutateAsync({
      workspaceId: currentWorkspaceId,
      repositoryGroupId: '',
      title: newTaskTitle.trim(),
      prompt: newTaskPrompt.trim(),
    });

    setShowNewTask(false);
    setNewTaskTitle('');
    setNewTaskPrompt('');
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-[hsl(var(--muted-foreground))]">Loading tasks...</div>
      </div>
    );
  }

  if (error) {
    const message = error instanceof Error ? error.message : 'Failed to load tasks';
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-center max-w-md">
          <p className="text-[hsl(var(--destructive))]">{message}</p>
          <Button variant="outline" className="mt-4" onClick={() => refetch()}>
            Try Again
          </Button>
        </div>
      </div>
    );
  }

  const tasks = data?.tasks ?? [];

  const statusOrder: UnitTaskStatus[] = [
    'action_required',
    'in_progress',
    'queued',
    'blocked',
    'completed',
    'failed',
    'cancelled',
  ];

  const grouped = statusOrder.reduce<Record<string, UnitTask[]>>((acc, s) => {
    acc[s] = tasks.filter((t) => t.status === s);
    return acc;
  }, {});

  const sectionLabels: Record<string, string> = {
    action_required: 'Action Required',
    in_progress: 'In Progress',
    queued: 'Queued',
    blocked: 'Blocked',
    completed: 'Completed',
    failed: 'Failed',
    cancelled: 'Cancelled',
  };

  return (
    <div className="flex h-full flex-col">
      <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-6 py-4">
        <div>
          <h1 className="text-2xl font-bold">Tasks</h1>
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            {tasks.length} total
          </p>
        </div>
        <Button onClick={() => setShowNewTask(true)}>
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
            className="mr-2"
          >
            <path d="M5 12h14" />
            <path d="M12 5v14" />
          </svg>
          New Task
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        {tasks.length === 0 ? (
          <div className="flex h-full items-center justify-center">
            <div className="text-center">
              <p className="text-[hsl(var(--muted-foreground))]">No tasks yet.</p>
              <Button className="mt-4" onClick={() => setShowNewTask(true)}>
                Create your first task
              </Button>
            </div>
          </div>
        ) : (
          <div className="mx-auto max-w-3xl space-y-8">
            {statusOrder.map((status) => (
              <TaskSection
                key={status}
                title={sectionLabels[status]}
                tasks={grouped[status]}
                onTaskClick={handleTaskClick}
              />
            ))}
          </div>
        )}
      </div>

      <Dialog open={showNewTask} onOpenChange={setShowNewTask}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>New Task</DialogTitle>
          </DialogHeader>
          <div className="space-y-4">
            <div>
              <label className="mb-1 block text-sm font-medium" htmlFor="task-title">
                Title
              </label>
              <Input
                id="task-title"
                placeholder="Brief title for the task"
                value={newTaskTitle}
                onChange={(e) => setNewTaskTitle(e.target.value)}
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium" htmlFor="task-prompt">
                Prompt
              </label>
              <Textarea
                id="task-prompt"
                placeholder="Describe what you want the AI agent to do..."
                value={newTaskPrompt}
                onChange={(e) => setNewTaskPrompt(e.target.value)}
                onKeyDown={(e) => {
                  if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                    e.preventDefault();
                    handleCreateTask();
                  }
                }}
                rows={5}
              />
            </div>
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setShowNewTask(false)}>
              Cancel
            </Button>
            <Button
              onClick={handleCreateTask}
              disabled={
                !newTaskTitle.trim() ||
                !newTaskPrompt.trim() ||
                !currentWorkspaceId ||
                createTask.isPending
              }
            >
              {createTask.isPending ? 'Creating...' : 'Create Task'}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
