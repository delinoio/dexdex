// React hooks for task log streaming
import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useQuery } from "@tanstack/react-query";
import { getTaskLogs } from "@/api/client";
import type {
  AgentOutputEvent,
  NormalizedEvent,
  NormalizedEventEntry,
  UnitTaskStatus,
} from "@/api/types";

// Query keys for task logs
export const taskLogsKeys = {
  all: ["taskLogs"] as const,
  logs: (taskId: string) => [...taskLogsKeys.all, taskId] as const,
};

interface UseTaskLogsOptions {
  taskId: string;
  taskStatus: UnitTaskStatus;
  enabled?: boolean;
  pollingInterval?: number;
}

interface UseTaskLogsResult {
  events: NormalizedEventEntry[];
  isLoading: boolean;
  isComplete: boolean;
  error: Error | null;
}

/**
 * Hook for streaming task logs from an AI agent session.
 *
 * This hook:
 * - Polls for logs using react-query
 * - Listens for real-time agent-output Tauri events
 * - Accumulates logs in state
 * - Stops polling when task is complete
 */
export function useTaskLogs({
  taskId,
  taskStatus,
  enabled = true,
  pollingInterval = 2000,
}: UseTaskLogsOptions): UseTaskLogsResult {
  const [events, setEvents] = useState<NormalizedEventEntry[]>([]);
  const [lastEventId, setLastEventId] = useState<number | undefined>();
  const eventIdCounter = useRef(0);

  // Track if task is complete based on status
  const isComplete =
    taskStatus !== "in_progress" && taskStatus !== ("in_progress" as string);

  // Poll for logs
  const { data, isLoading, error } = useQuery({
    queryKey: [...taskLogsKeys.logs(taskId), lastEventId],
    queryFn: () => getTaskLogs(taskId, lastEventId),
    enabled: enabled && !!taskId,
    refetchInterval: isComplete ? false : pollingInterval,
  });

  // Update events when we receive new data from polling
  useEffect(() => {
    if (data?.events && data.events.length > 0) {
      setEvents((prev) => {
        const existingIds = new Set(prev.map((e) => e.id));
        const newEvents = data.events.filter((e) => !existingIds.has(e.id));
        if (newEvents.length > 0) {
          return [...prev, ...newEvents];
        }
        return prev;
      });

      if (data.lastEventId !== undefined) {
        setLastEventId(data.lastEventId);
      }
    }
  }, [data]);

  // Listen for real-time agent output events
  useEffect(() => {
    if (!enabled || !taskId) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<AgentOutputEvent>("agent-output", (event) => {
        if (event.payload.taskId === taskId) {
          // Create a new event entry for real-time events
          const newEntry: NormalizedEventEntry = {
            id: ++eventIdCounter.current,
            timestamp: new Date().toISOString(),
            event: event.payload.event,
          };

          setEvents((prev) => [...prev, newEntry]);
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

  // Reset events when task changes
  useEffect(() => {
    setEvents([]);
    setLastEventId(undefined);
    eventIdCounter.current = 0;
  }, [taskId]);

  return {
    events,
    isLoading,
    isComplete: isComplete || (data?.isComplete ?? false),
    error: error as Error | null,
  };
}

/**
 * Extracts a display-friendly summary from a normalized event.
 */
export function getEventSummary(event: NormalizedEvent): string {
  switch (event.type) {
    case "text_output":
      return event.content;
    case "error_output":
      return `Error: ${event.content}`;
    case "tool_use":
      return `Using tool: ${event.toolName}`;
    case "tool_result":
      return `Tool result: ${event.toolName}${event.isError ? " (error)" : ""}`;
    case "file_change":
      const changeType =
        typeof event.changeType === "string"
          ? event.changeType
          : "rename";
      return `File ${changeType}: ${event.path}`;
    case "command_execution":
      return `Running: ${event.command}`;
    case "ask_user_question":
      return `Question: ${event.question}`;
    case "user_response":
      return `Response: ${event.response}`;
    case "session_start":
      return `Session started (${event.agentType})`;
    case "session_end":
      return event.success ? "Session completed" : `Session failed: ${event.error}`;
    case "thinking":
      return `Thinking: ${event.content.slice(0, 100)}...`;
    case "raw":
      return event.content;
    default:
      return "Unknown event";
  }
}
