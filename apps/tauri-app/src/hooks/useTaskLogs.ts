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
  logs: (agentTaskId: string) => [...taskLogsKeys.all, agentTaskId] as const,
};

interface UseTaskLogsOptions {
  agentTaskId: string;
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
 * Creates a content-based fingerprint for an event to detect duplicates.
 * This is used to match real-time events with their polled equivalents.
 */
function getEventFingerprint(event: NormalizedEvent): string {
  // Create a fingerprint based on event type and key content
  switch (event.type) {
    case "text_output":
    case "error_output":
    case "thinking":
    case "raw":
      // For text-based events, use type + first 200 chars of content
      return `${event.type}:${event.content.slice(0, 200)}`;
    case "tool_use":
      return `${event.type}:${event.tool_name}:${JSON.stringify(event.input).slice(0, 100)}`;
    case "tool_result":
      return `${event.type}:${event.tool_name}:${JSON.stringify(event.output).slice(0, 100)}`;
    case "file_change":
      return `${event.type}:${event.path}:${JSON.stringify(event.change_type)}`;
    case "command_execution":
      return `${event.type}:${event.command}`;
    case "ask_user_question":
      return `${event.type}:${event.question}`;
    case "user_response":
      return `${event.type}:${event.response}`;
    case "session_start":
      return `${event.type}:${event.agent_type}`;
    case "session_end":
      return `${event.type}:${event.success}:${event.error || ""}`;
    default:
      return `unknown:${JSON.stringify(event)}`;
  }
}

// Maximum number of fingerprints to track to prevent unbounded memory growth
// This is typically more than enough for even long-running tasks
const MAX_FINGERPRINTS = 10000;

/**
 * Hook for streaming task logs from an AI agent session.
 *
 * This hook:
 * - Polls for logs using react-query
 * - Listens for real-time agent-output Tauri events
 * - Deduplicates events using content-based fingerprinting
 * - Stops polling when task is complete
 *
 * ## Deduplication Strategy
 * Real-time events are shown immediately for responsiveness, but may also
 * arrive later via polling with different IDs. We use content-based fingerprinting
 * to detect and skip duplicates, ensuring each logical event appears only once.
 *
 * ## Memory Management
 * The fingerprint set is bounded to MAX_FINGERPRINTS entries. When the limit is
 * reached, we reset the set. This may cause some duplicate events to appear in
 * rare cases for very long-running tasks, but prevents unbounded memory growth.
 */
export function useTaskLogs({
  agentTaskId,
  taskStatus,
  enabled = true,
  pollingInterval = 2000,
}: UseTaskLogsOptions): UseTaskLogsResult {
  const [events, setEvents] = useState<NormalizedEventEntry[]>([]);
  const [lastEventId, setLastEventId] = useState<number | undefined>();
  // Track fingerprints of all events we've seen to detect duplicates
  // Bounded to MAX_FINGERPRINTS to prevent unbounded memory growth
  const seenFingerprints = useRef(new Set<string>());
  const eventIdCounter = useRef(0);

  // Track if task is complete based on status
  // Task is complete when status is NOT "in_progress"
  const isComplete = taskStatus !== "in_progress";

  // Use a ref to track lastEventId for the query function
  // This avoids changing the query key on every fetch, which would reset the cache
  const lastEventIdRef = useRef<number | undefined>(lastEventId);
  lastEventIdRef.current = lastEventId;

  // Poll for logs
  // Note: queryKey does NOT include lastEventId to avoid cache invalidation
  // The lastEventId is passed via ref to the queryFn
  const { data, isLoading, error } = useQuery({
    queryKey: taskLogsKeys.logs(agentTaskId),
    queryFn: async () => {
      const afterEventId = lastEventIdRef.current;
      console.log("[useTaskLogs] Fetching logs for agentTaskId:", agentTaskId, "afterEventId:", afterEventId);
      const result = await getTaskLogs(agentTaskId, afterEventId);
      console.log("[useTaskLogs] Received", result.events.length, "events, isComplete:", result.isComplete);
      return result;
    },
    enabled: enabled && !!agentTaskId,
    refetchInterval: isComplete ? false : pollingInterval,
  });

  // Track the previous agentTaskId to detect changes
  const prevAgentTaskIdRef = useRef(agentTaskId);

  // Reset events when agent task changes
  // IMPORTANT: This effect must be declared BEFORE the data update effect
  // to ensure proper effect ordering. React runs effects in declaration order,
  // so this reset runs first when agentTaskId changes.
  useEffect(() => {
    // Only reset if agentTaskId actually changed to a different value
    // Skip the initial mount to avoid resetting before data loads
    if (prevAgentTaskIdRef.current !== agentTaskId) {
      console.log("[useTaskLogs] agentTaskId changed from", prevAgentTaskIdRef.current, "to", agentTaskId, "- resetting state");
      setEvents([]);
      setLastEventId(undefined);
      lastEventIdRef.current = undefined;
      seenFingerprints.current = new Set();
      eventIdCounter.current = 0;
      prevAgentTaskIdRef.current = agentTaskId;
    }
  }, [agentTaskId]);

  // Update events when we receive new data from polling
  useEffect(() => {
    if (!data?.events || data.events.length === 0) {
      return;
    }

    console.log("[useTaskLogs] Data effect: processing", data.events.length, "events");

    // Filter events SYNCHRONOUSLY before calling setEvents
    // This is critical for StrictMode: when the effect runs twice,
    // the second run will see fingerprints already added and skip those events.
    // If we did this inside setEvents callback, both callbacks would run
    // and the second would overwrite the first with empty results.
    const newEvents: NormalizedEventEntry[] = [];
    for (const e of data.events) {
      const fingerprint = getEventFingerprint(e.event);
      if (!seenFingerprints.current.has(fingerprint)) {
        // Prevent unbounded memory growth by resetting if we exceed the limit
        if (seenFingerprints.current.size >= MAX_FINGERPRINTS) {
          console.warn(
            "Fingerprint set exceeded limit, resetting. Some duplicates may appear.",
          );
          seenFingerprints.current.clear();
        }
        seenFingerprints.current.add(fingerprint);
        newEvents.push(e);
      }
    }

    console.log("[useTaskLogs] After filtering, found", newEvents.length, "new events");

    // Only call setEvents if we have new events to add
    // This prevents the second StrictMode effect run from overwriting
    // the first run's results with an empty append
    if (newEvents.length > 0) {
      setEvents((prev) => {
        console.log("[useTaskLogs] Appending", newEvents.length, "events to", prev.length, "existing");
        return [...prev, ...newEvents];
      });
    }

    if (data.lastEventId !== undefined) {
      setLastEventId(data.lastEventId);
    }
  }, [data]);

  // Listen for real-time agent output events
  useEffect(() => {
    if (!enabled || !agentTaskId) return;

    let unlisten: UnlistenFn | undefined;

    const setupListener = async () => {
      unlisten = await listen<AgentOutputEvent>("agent-output", (event) => {
        if (event.payload.taskId === agentTaskId) {
          // Check if we've already seen this event (from polling)
          const fingerprint = getEventFingerprint(event.payload.event);
          if (seenFingerprints.current.has(fingerprint)) {
            // Skip duplicate - already have this event from polling
            return;
          }
          // Prevent unbounded memory growth by resetting if we exceed the limit
          if (seenFingerprints.current.size >= MAX_FINGERPRINTS) {
            console.warn(
              "Fingerprint set exceeded limit, resetting. Some duplicates may appear.",
            );
            seenFingerprints.current.clear();
          }
          seenFingerprints.current.add(fingerprint);

          // Create a new event entry for real-time events
          // Use a string prefix "rt-" to distinguish from polled event IDs
          const newEntry: NormalizedEventEntry = {
            id: `rt-${agentTaskId}-${++eventIdCounter.current}`,
            timestamp: new Date().toISOString(),
            event: event.payload.event,
          };

          setEvents((prev) => [...prev, newEntry]);
        }
      });
    };

    setupListener().catch((error) => {
      console.error("Failed to set up agent-output listener:", error);
    });

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, [agentTaskId, enabled]);

  // Clean up fingerprint set when task completes to free memory
  useEffect(() => {
    if (isComplete) {
      // Task is complete - no more events will arrive, so we can clear the fingerprint set
      // The events are already stored in state, so we don't need fingerprints anymore
      seenFingerprints.current = new Set();
    }
  }, [isComplete]);

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
