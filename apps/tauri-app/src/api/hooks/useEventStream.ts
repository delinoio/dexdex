import { useEffect, useRef } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import type { StreamEvent } from '../types';

const BASE_URL = 'http://localhost:3000';

export function useEventStream(workspaceId: string) {
  const queryClient = useQueryClient();
  const esRef = useRef<EventSource | null>(null);

  useEffect(() => {
    if (!workspaceId) return;

    // SSE with GET query param (EventSource only supports GET)
    const es = new EventSource(
      `${BASE_URL}/EventStreamService/Subscribe?workspaceId=${encodeURIComponent(workspaceId)}`
    );
    esRef.current = es;

    es.onmessage = (e: MessageEvent<string>) => {
      let event: StreamEvent;
      try {
        event = JSON.parse(e.data) as StreamEvent;
      } catch {
        return;
      }

      switch (event.eventType) {
        case 'task_updated':
          queryClient.invalidateQueries({ queryKey: ['tasks'] });
          break;
        case 'subtask_updated':
          queryClient.invalidateQueries({ queryKey: ['subtasks'] });
          break;
        case 'session_output':
        case 'session_state_changed':
          queryClient.invalidateQueries({ queryKey: ['session-outputs'] });
          queryClient.invalidateQueries({ queryKey: ['agent-sessions'] });
          break;
        case 'pr_updated':
          queryClient.invalidateQueries({ queryKey: ['pr-trackings'] });
          break;
        case 'notification_created':
          queryClient.invalidateQueries({ queryKey: ['notifications'] });
          break;
        case 'review_assist_updated':
          queryClient.invalidateQueries({ queryKey: ['review-assist'] });
          break;
        case 'inline_comment_updated':
          queryClient.invalidateQueries({ queryKey: ['inline-comments'] });
          break;
        default:
          break;
      }
    };

    es.onerror = () => {
      // Connection error - EventSource will auto-retry
      // We don't need to do anything special here
    };

    return () => {
      es.close();
      esRef.current = null;
    };
  }, [workspaceId, queryClient]);
}
