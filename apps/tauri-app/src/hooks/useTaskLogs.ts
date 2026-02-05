// React hooks for task log streaming via Tauri events.
//
// Uses an event-driven approach: loads existing logs once on mount, then
// streams new events in real-time via Tauri's `agent-output` event.
// No polling is required because the backend emits every event via Tauri.
import { useEffect, useRef, useState, useCallback } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getTaskLogs } from "@/api/client";
import type {
  AgentOutputEvent,
  NormalizedEvent,
  NormalizedEventEntry,
  UnitTaskStatus,
} from "@/api/types";

// Query keys for task logs (kept for external cache invalidation if needed)
export const taskLogsKeys = {
  all: ["taskLogs"] as const,
  logs: (agentTaskId: string) => [...taskLogsKeys.all, agentTaskId] as const,
};

interface UseTaskLogsOptions {
  /** The unit/composite task ID used for matching real-time Tauri events. */
  taskId: string;
  /** The agent task ID used for fetching persisted logs from the database. */
  agentTaskId: string;
  taskStatus: UnitTaskStatus;
  enabled?: boolean;
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
 * Uses a purely event-driven approach:
 * 1. On mount (or when agentTaskId changes), fetches existing logs from the
 *    database to catch up on any events that happened before the component
 *    was rendered.
 * 2. Listens for real-time `agent-output` Tauri events for new events.
 *
 * No polling is used. The Tauri backend emits every agent output event in
 * real-time, so the frontend receives updates immediately.
 */
export function useTaskLogs({
  taskId,
  agentTaskId,
  taskStatus,
  enabled = true,
}: UseTaskLogsOptions): UseTaskLogsResult {
  const [events, setEvents] = useState<NormalizedEventEntry[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const eventIdCounter = useRef(0);
  // Track whether the initial fetch has completed so that real-time events
  // arriving before the fetch finishes do not cause confusion.
  const initialFetchDone = useRef(false);

  // Track if task is complete based on status
  const isComplete = taskStatus !== "in_progress";

  // Fetch existing logs once when the component mounts or agentTaskId changes.
  // This loads any events that were persisted before the component rendered.
  useEffect(() => {
    if (!enabled || !agentTaskId) return;

    let cancelled = false;
    initialFetchDone.current = false;

    const fetchExistingLogs = async () => {
      setIsLoading(true);
      setError(null);
      try {
        const result = await getTaskLogs(agentTaskId);
        if (cancelled) return;

        if (result.events.length > 0) {
          setEvents(result.events);
          eventIdCounter.current = result.events.length;
        }
      } catch (err) {
        if (!cancelled) {
          console.error("Failed to fetch existing task logs:", err);
          setError(err instanceof Error ? err : new Error(String(err)));
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
          initialFetchDone.current = true;
        }
      }
    };

    fetchExistingLogs();

    return () => {
      cancelled = true;
    };
  }, [agentTaskId, enabled]);

  // Reset events when agent task changes
  const prevAgentTaskIdRef = useRef(agentTaskId);
  useEffect(() => {
    if (prevAgentTaskIdRef.current !== agentTaskId) {
      setEvents([]);
      setError(null);
      eventIdCounter.current = 0;
      initialFetchDone.current = false;
      prevAgentTaskIdRef.current = agentTaskId;
    }
  }, [agentTaskId]);

  // Append a new event from the real-time listener
  const appendEvent = useCallback(
    (normalizedEvent: NormalizedEvent) => {
      const newEntry: NormalizedEventEntry = {
        id: `rt-${agentTaskId}-${++eventIdCounter.current}`,
        timestamp: new Date().toISOString(),
        event: normalizedEvent,
      };
      setEvents((prev) => [...prev, newEntry]);
    },
    [agentTaskId],
  );

  // Listen for real-time agent output events.
  // The backend emits events with the unit/composite task ID (not the agent
  // task ID), so we filter by `taskId` here.
  useEffect(() => {
    if (!enabled || !taskId) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<AgentOutputEvent>("agent-output", (event) => {
        if (event.payload.taskId !== taskId) return;
        appendEvent(event.payload.event);
      });
    };

    setupListener().catch((err) => {
      console.error("Failed to set up agent-output listener:", err);
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [taskId, enabled, appendEvent]);

  return {
    events,
    isLoading,
    isComplete,
    error,
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
      return `Using tool: ${event.tool_name}`;
    case "tool_result":
      return `Tool result: ${event.tool_name}${event.is_error ? " (error)" : ""}`;
    case "file_change": {
      const changeType =
        typeof event.change_type === "string"
          ? event.change_type
          : "rename";
      return `File ${changeType}: ${event.path}`;
    }
    case "command_execution":
      return `Running: ${event.command}`;
    case "ask_user_question":
      return `Question: ${event.question}`;
    case "user_response":
      return `Response: ${event.response}`;
    case "session_start":
      return `Session started (${event.agent_type})`;
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
