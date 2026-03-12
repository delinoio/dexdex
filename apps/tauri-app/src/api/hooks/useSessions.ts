import { useQuery } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListAgentSessionsRequest,
  ListAgentSessionsResponse,
  ListSessionOutputsRequest,
  ListSessionOutputsResponse,
  AgentSessionStatus,
} from '../types';

export function useAgentSessions(subTaskId: string) {
  return useQuery({
    queryKey: ['agent-sessions', subTaskId],
    queryFn: () =>
      rpcCall<ListAgentSessionsRequest, ListAgentSessionsResponse>('SessionService', 'List', {
        subTaskId,
      }),
    enabled: !!subTaskId,
    refetchInterval: 3000,
  });
}

const ACTIVE_STATUSES: AgentSessionStatus[] = ['starting', 'running', 'waiting_for_input'];

export function useSessionOutputs(sessionId: string, isActive: boolean) {
  return useQuery({
    queryKey: ['session-outputs', sessionId],
    queryFn: () =>
      rpcCall<ListSessionOutputsRequest, ListSessionOutputsResponse>(
        'SessionService',
        'ListOutputs',
        { sessionId }
      ),
    enabled: !!sessionId,
    // Poll more frequently when session is active
    refetchInterval: isActive ? 1500 : false,
  });
}

export function isActiveSession(status: AgentSessionStatus): boolean {
  return ACTIVE_STATUSES.includes(status);
}
