import { useQuery } from '@tanstack/react-query';
import { rpcCall } from '../client';
import type {
  ListPrTrackingsRequest,
  ListPrTrackingsResponse,
} from '../types';

export function usePrTrackings(unitTaskId: string) {
  return useQuery({
    queryKey: ['pr-trackings', unitTaskId],
    queryFn: () =>
      rpcCall<ListPrTrackingsRequest, ListPrTrackingsResponse>('PrManagementService', 'List', {
        unitTaskId,
      }),
    enabled: !!unitTaskId,
    refetchInterval: 30000,
  });
}
