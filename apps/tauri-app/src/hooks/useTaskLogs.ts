// React hooks for task log streaming via Tauri events
import { useEffect, useRef, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { getTaskLogs } from "@/api/client";
import type {
  AgentOutputEvent,
  NormalizedEvent,
  NormalizedEventEntry,
  UnitTaskStatus,
} from "@/api/types";

interface UseTaskLogsOptions {
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
 * This hook uses an event-driven approach:
 * 1. On mount, performs a single initial fetch from the database
 *    to recover events from before the component mounted (e.g., page reload).
 * 2. Listens for real-time `agent-output` Tauri events for live streaming.
 * 3. No polling - all updates come through the Tauri event system.
 */
export function useTaskLogs({
  agentTaskId,
  taskStatus,
  enabled = true,
}: UseTaskLogsOptions): UseTaskLogsResult {
  const [events, setEvents] = useState<NormalizedEventEntry[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const eventIdCounter = useRef(0);

  // Track if task is complete based on status
  const isComplete = taskStatus !== "in_progress";

  // Track the previous agentTaskId to detect changes
  const prevAgentTaskIdRef = useRef(agentTaskId);

  // Track whether initial fetch has completed to avoid duplicating events
  const initialFetchDoneRef = useRef(false);

  // Reset events when agent task changes
  useEffect(() => {
    if (prevAgentTaskIdRef.current !== agentTaskId) {
      setEvents([]);
      setError(null);
      setIsLoading(true);
      eventIdCounter.current = 0;
      initialFetchDoneRef.current = false;
      prevAgentTaskIdRef.current = agentTaskId;
    }
  }, [agentTaskId]);

  // Initial fetch: load existing events from the database once on mount.
  // This handles page reloads and late-mounting components that missed real-time events.
  useEffect(() => {
    if (!enabled || !agentTaskId) {
      setIsLoading(false);
      return;
    }

    let cancelled = false;

    const fetchInitialEvents = async () => {
      try {
        const result = await getTaskLogs(agentTaskId);
        if (cancelled) return;

        if (result.events.length > 0) {
          setEvents(result.events);
          eventIdCounter.current = result.events.length;
        }
        initialFetchDoneRef.current = true;
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err : new Error(String(err)));
        }
      } finally {
        if (!cancelled) {
          setIsLoading(false);
        }
      }
    };

    fetchInitialEvents();

    return () => {
      cancelled = true;
    };
  }, [agentTaskId, enabled]);

  // Listen for real-time agent output events
  useEffect(() => {
    if (!enabled || !agentTaskId) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<AgentOutputEvent>("agent-output", (event) => {
        if (event.payload.taskId === agentTaskId) {
          const newEntry: NormalizedEventEntry = {
            id: `rt-${agentTaskId}-${++eventIdCounter.current}`,
            timestamp: new Date().toISOString(),
            event: event.payload.event,
          };

          setEvents((prev) => [...prev, newEntry]);
        }
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
  }, [agentTaskId, enabled]);

  return {
    events,
    isLoading,
    isComplete: isComplete,
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
    case "file_change":
      const changeType =
        typeof event.change_type === "string"
          ? event.change_type
          : "rename";
      return `File ${changeType}: ${event.path}`;
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
