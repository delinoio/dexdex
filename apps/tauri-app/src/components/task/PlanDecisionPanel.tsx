import { useState, useCallback } from 'react';
import { Button } from '@/components/ui/Button';
import { Textarea } from '@/components/ui/Textarea';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/Card';
import { useApprovePlan, useRevisePlan } from '@/api/hooks/useTasks';
import type { SubTask } from '@/api/types';

interface PlanDecisionPanelProps {
  subTask: SubTask;
  planContent?: string;
  onDecision?: () => void;
}

export function PlanDecisionPanel({ subTask, planContent, onDecision }: PlanDecisionPanelProps) {
  const [reviseMode, setReviseMode] = useState(false);
  const [reviseFeedback, setReviseFeedback] = useState('');

  const approvePlan = useApprovePlan();
  const revisePlan = useRevisePlan();

  const handleApprove = useCallback(async () => {
    await approvePlan.mutateAsync(subTask.id);
    onDecision?.();
  }, [approvePlan, subTask.id, onDecision]);

  const handleRevise = useCallback(async () => {
    if (!reviseFeedback.trim()) return;
    await revisePlan.mutateAsync({ subTaskId: subTask.id, feedback: reviseFeedback.trim() });
    setReviseMode(false);
    setReviseFeedback('');
    onDecision?.();
  }, [revisePlan, subTask.id, reviseFeedback, onDecision]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault();
        handleRevise();
      }
    },
    [handleRevise]
  );

  return (
    <Card className="border-yellow-500/50">
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
            className="text-yellow-500"
          >
            <circle cx="12" cy="12" r="10" />
            <path d="M12 16v-4" />
            <path d="M12 8h.01" />
          </svg>
          <CardTitle>Plan Approval Required</CardTitle>
        </div>
        <CardDescription>
          The AI agent has created a plan. Review it and approve or request changes.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {planContent && (
          <div className="rounded-md bg-[hsl(var(--muted))] p-4">
            <pre className="whitespace-pre-wrap text-sm">{planContent}</pre>
          </div>
        )}

        {reviseMode ? (
          <div className="space-y-3">
            <Textarea
              placeholder="Describe the changes you'd like to the plan..."
              value={reviseFeedback}
              onChange={(e) => setReviseFeedback(e.target.value)}
              onKeyDown={handleKeyDown}
              rows={4}
              autoFocus
            />
            <div className="flex gap-2">
              <Button
                onClick={handleRevise}
                disabled={!reviseFeedback.trim() || revisePlan.isPending}
              >
                {revisePlan.isPending ? 'Sending...' : 'Submit Feedback'}
              </Button>
              <Button
                variant="outline"
                onClick={() => {
                  setReviseMode(false);
                  setReviseFeedback('');
                }}
              >
                Cancel
              </Button>
            </div>
            <p className="text-xs text-[hsl(var(--muted-foreground))]">
              Press Cmd+Enter to submit
            </p>
          </div>
        ) : (
          <div className="flex gap-2">
            <Button onClick={handleApprove} disabled={approvePlan.isPending}>
              {approvePlan.isPending ? 'Approving...' : 'Approve Plan'}
            </Button>
            <Button variant="outline" onClick={() => setReviseMode(true)}>
              Revise Plan
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
