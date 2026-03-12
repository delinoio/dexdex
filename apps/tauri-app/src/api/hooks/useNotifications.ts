import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListNotificationsRequest,
  ListNotificationsResponse,
  MarkNotificationReadRequest,
  MarkAllNotificationsReadRequest,
} from '../types';

export function useNotifications(workspaceId: string, unreadOnly?: boolean) {
  return useQuery({
    queryKey: ['notifications', workspaceId, unreadOnly],
    queryFn: () =>
      rpcCall<ListNotificationsRequest, ListNotificationsResponse>(
        'NotificationService',
        'List',
        { workspaceId, unreadOnly, limit: 100, offset: 0 }
      ),
    enabled: !!workspaceId,
    refetchInterval: 30000,
  });
}

export function useMarkNotificationRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (notificationId: string) =>
      rpcCall<MarkNotificationReadRequest, Record<string, never>>(
        'NotificationService',
        'MarkRead',
        { notificationId }
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
    },
  });
}

export function useMarkAllNotificationsRead() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (workspaceId: string) =>
      rpcCall<MarkAllNotificationsReadRequest, Record<string, never>>(
        'NotificationService',
        'MarkAllRead',
        { workspaceId }
      ),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['notifications'] });
    },
  });
}
