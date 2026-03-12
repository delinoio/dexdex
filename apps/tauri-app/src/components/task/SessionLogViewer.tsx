import { useEffect, useRef } from 'react';
import { cn } from '@/lib/utils';
import { useSessionOutputs, isActiveSession } from '@/api/hooks/useSessions';
import type { AgentSessionStatus, SessionOutputKind, SessionOutputEvent } from '@/api/types';

function kindColor(kind: SessionOutputKind): string {
  switch (kind) {
    case 'error':
      return 'text-[hsl(var(--destructive))]';
    case 'warning':
      return 'text-yellow-500';
    case 'progress':
      return 'text-[hsl(var(--primary))]';
    case 'tool_call':
      return 'text-blue-400';
    case 'tool_result':
      return 'text-[hsl(var(--muted-foreground))]';
    case 'plan_update':
      return 'text-purple-400';
    default:
      return 'text-[hsl(var(--foreground))]';
  }
}

function kindPrefix(kind: SessionOutputKind): string {
  switch (kind) {
    case 'error':
      return '[ERROR] ';
    case 'warning':
      return '[WARN] ';
    case 'tool_call':
      return '[TOOL] ';
    case 'tool_result':
      return '[RESULT] ';
    case 'plan_update':
      return '[PLAN] ';
    case 'progress':
      return '[INFO] ';
    default:
      return '';
  }
}

interface LogLineProps {
  event: SessionOutputEvent;
}

function LogLine({ event }: LogLineProps) {
  return (
    <div className={cn('font-mono text-xs leading-relaxed', kindColor(event.kind))}>
      {kindPrefix(event.kind)}
      {event.message}
    </div>
  );
}

interface SessionLogViewerProps {
  sessionId: string;
  sessionStatus: AgentSessionStatus;
  className?: string;
}

export function SessionLogViewer({ sessionId, sessionStatus, className }: SessionLogViewerProps) {
  const active = isActiveSession(sessionStatus);
  const { data, isLoading, error } = useSessionOutputs(sessionId, active);
  const bottomRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (active && bottomRef.current) {
      bottomRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [data?.events, active]);

  if (isLoading) {
    return (
      <div
        className={cn(
          'flex items-center justify-center rounded-md bg-[hsl(var(--muted))] p-4 text-sm text-[hsl(var(--muted-foreground))]',
          className
        )}
      >
        Loading logs...
      </div>
    );
  }

  if (error) {
    return (
      <div
        className={cn(
          'rounded-md bg-[hsl(var(--muted))] p-4 text-sm text-[hsl(var(--destructive))]',
          className
        )}
      >
        Failed to load session logs.
      </div>
    );
  }

  const events = data?.events ?? [];

  return (
    <div
      className={cn(
        'overflow-y-auto rounded-md bg-[hsl(var(--muted))] p-4',
        className
      )}
    >
      {events.length === 0 ? (
        <div className="text-sm text-[hsl(var(--muted-foreground))]">
          {active ? 'Waiting for output...' : 'No output recorded.'}
        </div>
      ) : (
        <div className="space-y-0.5">
          {events.map((event) => (
            <LogLine key={event.id} event={event} />
          ))}
          {active && (
            <div className="mt-1 flex items-center gap-1 text-xs text-[hsl(var(--primary))]">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="12"
                height="12"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="animate-spin"
              >
                <path d="M21 12a9 9 0 1 1-6.219-8.56" />
              </svg>
              Running...
            </div>
          )}
          <div ref={bottomRef} />
        </div>
      )}
    </div>
  );
}
