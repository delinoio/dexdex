// React hook for handling TTY input requests from AI agents
import { useCallback, useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useMutation } from "@tanstack/react-query";
import { respondTtyInput } from "@/api/client";
import type { TtyInputRequestEvent } from "@/api/types";

interface PendingTtyRequest {
  requestId: string;
  taskId: string;
  sessionId: string;
  question: string;
  options?: string[];
}

interface UseTtyInputOptions {
  taskId: string;
  enabled?: boolean;
}

interface UseTtyInputResult {
  pendingRequest: PendingTtyRequest | null;
  respond: (response: string) => Promise<void>;
  isResponding: boolean;
  error: Error | null;
}

/**
 * Hook for handling TTY input requests from AI agents.
 *
 * This hook:
 * - Listens for tty-input-request Tauri events
 * - Tracks pending request state
 * - Provides a respond function to send responses
 */
export function useTtyInput({
  taskId,
  enabled = true,
}: UseTtyInputOptions): UseTtyInputResult {
  const [pendingRequest, setPendingRequest] = useState<PendingTtyRequest | null>(null);

  // Mutation for responding to TTY input
  const respondMutation = useMutation({
    mutationFn: async (response: string) => {
      if (!pendingRequest) {
        throw new Error("No pending request to respond to");
      }
      await respondTtyInput({
        requestId: pendingRequest.requestId,
        response,
      });
    },
    onSuccess: () => {
      // Clear the pending request after successful response
      setPendingRequest(null);
    },
    onError: (error) => {
      // Clear the pending request on error to avoid stuck UI state
      // The request either doesn't exist anymore or failed permanently
      console.error("Failed to respond to TTY input:", error);
      setPendingRequest(null);
    },
  });

  // Listen for TTY input request events
  useEffect(() => {
    if (!enabled || !taskId) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<TtyInputRequestEvent>("tty-input-request", (event) => {
        if (event.payload.taskId === taskId) {
          setPendingRequest({
            requestId: event.payload.requestId,
            taskId: event.payload.taskId,
            sessionId: event.payload.sessionId,
            question: event.payload.question,
            options: event.payload.options,
          });
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [taskId, enabled]);

  // Clear pending request when task changes
  useEffect(() => {
    setPendingRequest(null);
  }, [taskId]);

  const respond = useCallback(
    async (response: string) => {
      await respondMutation.mutateAsync(response);
    },
    [respondMutation]
  );

  return {
    pendingRequest,
    respond,
    isResponding: respondMutation.isPending,
    error: respondMutation.error as Error | null,
  };
}
