import { useEffect, useRef, useState } from "react";
import { cn } from "@/lib/utils";
import { useTaskLogs } from "@/hooks/useTaskLogs";
import { useTtyInput } from "@/hooks/useTtyInput";
import type { NormalizedEvent, NormalizedEventEntry, UnitTaskStatus } from "@/api/types";
import { Button } from "@/components/ui/Button";
import { Input } from "@/components/ui/Input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/Card";
import {
  LoaderIcon,
  TerminalIcon,
  FileCodeIcon,
  PlayIcon,
  AlertCircleIcon,
  MessageSquareIcon,
  BrainIcon,
} from "@/components/ui/Icons";
import { ToolUseContent, ToolResultContent } from "./ToolEventComponents";

interface AgentLogViewerProps {
  agentTaskId: string;
  taskStatus: UnitTaskStatus;
  className?: string;
}

/**
 * Component for displaying streaming agent logs with formatting per event type.
 */
export function AgentLogViewer({ agentTaskId, taskStatus, className }: AgentLogViewerProps) {
  const scrollRef = useRef<HTMLDivElement>(null);
  const [autoScroll, setAutoScroll] = useState(true);

  console.log("[AgentLogViewer] Rendering with agentTaskId:", agentTaskId, "taskStatus:", taskStatus);

  const { events, isLoading, isComplete, error } = useTaskLogs({
    agentTaskId,
    taskStatus,
    enabled: !!agentTaskId,
  });

  console.log("[AgentLogViewer] useTaskLogs result - events:", events.length, "isLoading:", isLoading, "isComplete:", isComplete, "error:", error);

  const { pendingRequest, respond, isResponding } = useTtyInput({
    taskId: agentTaskId,
    enabled: !!agentTaskId,
  });

  // Auto-scroll to bottom when new events arrive
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [events, autoScroll]);

  // Detect when user scrolls away from bottom
  const handleScroll = () => {
    if (scrollRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollRef.current;
      const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
      setAutoScroll(isAtBottom);
    }
  };

  if (error) {
    return (
      <div className={cn("flex items-center gap-2 text-destructive p-4", className)}>
        <AlertCircleIcon size={16} />
        <span>Failed to load logs: {error.message}</span>
      </div>
    );
  }

  const isTaskRunning = taskStatus === ("in_progress" as UnitTaskStatus);

  return (
    <div className={cn("flex flex-col h-full", className)}>
      {/* Header */}
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2 text-sm text-muted-foreground">
          <TerminalIcon size={16} />
          <span>Agent Session Log</span>
          {isTaskRunning && !isComplete && (
            <LoaderIcon size={12} className="animate-spin ml-2" />
          )}
        </div>
        {events.length > 0 && (
          <span className="text-xs text-muted-foreground">
            {events.length} event{events.length !== 1 ? "s" : ""}
          </span>
        )}
      </div>

      {/* TTY Input Dialog */}
      {pendingRequest && (
        <TtyInputDialog
          question={pendingRequest.question}
          options={pendingRequest.options}
          onRespond={respond}
          isResponding={isResponding}
        />
      )}

      {/* Log Content */}
      <div
        ref={scrollRef}
        onScroll={handleScroll}
        className="flex-1 overflow-y-auto font-mono text-sm bg-muted/30 rounded-lg p-4 space-y-2"
      >
        {isLoading && events.length === 0 ? (
          <div className="flex items-center gap-2 text-muted-foreground">
            <LoaderIcon size={16} className="animate-spin" />
            <span>Loading logs...</span>
          </div>
        ) : events.length === 0 ? (
          <div className="text-muted-foreground">
            {isTaskRunning
              ? "Waiting for agent output..."
              : "No logs available for this task."}
          </div>
        ) : (
          events.map((entry) => (
            <LogEntry key={entry.id} entry={entry} />
          ))
        )}

        {isComplete && events.length > 0 && (
          <div className="pt-2 border-t border-border/50 text-muted-foreground text-xs">
            Task execution completed
          </div>
        )}
      </div>

      {/* Auto-scroll indicator */}
      {!autoScroll && (
        <Button
          variant="outline"
          size="sm"
          className="mt-2"
          onClick={() => {
            setAutoScroll(true);
            if (scrollRef.current) {
              scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
            }
          }}
        >
          Scroll to bottom
        </Button>
      )}
    </div>
  );
}

interface LogEntryProps {
  entry: NormalizedEventEntry;
}

function LogEntry({ entry }: LogEntryProps) {
  const { event } = entry;

  return (
    <div className={cn("flex gap-2", getEventStyles(event))}>
      <EventIcon event={event} />
      <div className="flex-1 min-w-0">
        <EventContent event={event} />
      </div>
    </div>
  );
}

function EventIcon({ event }: { event: NormalizedEvent }) {
  const iconClass = "flex-shrink-0 mt-0.5";

  switch (event.type) {
    case "text_output":
      return <TerminalIcon size={16} className={cn(iconClass, "text-foreground")} />;
    case "error_output":
      return <AlertCircleIcon size={16} className={cn(iconClass, "text-destructive")} />;
    case "tool_use":
    case "tool_result":
      return <PlayIcon size={16} className={cn(iconClass, "text-blue-500")} />;
    case "file_change":
      return <FileCodeIcon size={16} className={cn(iconClass, "text-green-500")} />;
    case "command_execution":
      return <TerminalIcon size={16} className={cn(iconClass, "text-yellow-500")} />;
    case "ask_user_question":
    case "user_response":
      return <MessageSquareIcon size={16} className={cn(iconClass, "text-purple-500")} />;
    case "thinking":
      return <BrainIcon size={16} className={cn(iconClass, "text-cyan-500")} />;
    default:
      return <TerminalIcon size={16} className={cn(iconClass, "text-muted-foreground")} />;
  }
}

function EventContent({ event }: { event: NormalizedEvent }) {
  switch (event.type) {
    case "text_output":
      return <pre className="whitespace-pre-wrap break-words">{event.content}</pre>;

    case "error_output":
      return (
        <pre className="whitespace-pre-wrap break-words text-destructive">
          {event.content}
        </pre>
      );

    case "tool_use":
      return <ToolUseContent toolName={event.tool_name} input={event.input} />;

    case "tool_result":
      return (
        <div>
          <span className={cn("font-medium", event.is_error ? "text-destructive" : "text-green-500")}>
            {event.tool_name} {event.is_error ? "(error)" : "(success)"}
          </span>
          <div className="mt-1">
            <ToolResultContent
              toolName={event.tool_name}
              output={event.output}
              isError={event.is_error}
            />
          </div>
        </div>
      );

    case "file_change":
      const changeType = typeof event.change_type === "string" ? event.change_type : "rename";
      return (
        <div>
          <span className="text-green-500 font-medium">{changeType}</span>
          <span className="ml-2 text-foreground">{event.path}</span>
        </div>
      );

    case "command_execution":
      return (
        <div>
          <code className="text-yellow-500">{event.command}</code>
          {event.exit_code !== undefined && (
            <span className={cn("ml-2 text-xs", event.exit_code === 0 ? "text-green-500" : "text-destructive")}>
              (exit: {event.exit_code})
            </span>
          )}
          {event.output && (
            <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto">
              {event.output}
            </pre>
          )}
        </div>
      );

    case "ask_user_question":
      return (
        <div className="text-purple-500">
          <span className="font-medium">Question:</span> {event.question}
          {event.options && event.options.length > 0 && (
            <div className="mt-1 text-xs">
              Options: {event.options.join(", ")}
            </div>
          )}
        </div>
      );

    case "user_response":
      return (
        <div>
          <span className="text-purple-500 font-medium">Response:</span>{" "}
          <span>{event.response}</span>
        </div>
      );

    case "session_start":
      return (
        <div className="text-muted-foreground">
          Session started ({event.agent_type}
          {event.model && `, ${event.model}`})
        </div>
      );

    case "session_end":
      return (
        <div className={event.success ? "text-green-500" : "text-destructive"}>
          Session {event.success ? "completed successfully" : `failed: ${event.error}`}
        </div>
      );

    case "thinking":
      return (
        <details className="text-cyan-500">
          <summary className="cursor-pointer">Thinking...</summary>
          <pre className="mt-1 text-xs whitespace-pre-wrap opacity-70">{event.content}</pre>
        </details>
      );

    case "raw":
      return <pre className="whitespace-pre-wrap break-words opacity-70">{event.content}</pre>;

    default:
      return <span className="text-muted-foreground">Unknown event</span>;
  }
}

function getEventStyles(event: NormalizedEvent): string {
  switch (event.type) {
    case "error_output":
      return "text-destructive";
    case "session_end":
      return event.success ? "text-green-500" : "text-destructive";
    default:
      return "";
  }
}

interface TtyInputDialogProps {
  question: string;
  options?: string[];
  onRespond: (response: string) => void | Promise<void>;
  isResponding: boolean;
}

function TtyInputDialog({ question, options, onRespond, isResponding }: TtyInputDialogProps) {
  const [inputValue, setInputValue] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    const trimmed = inputValue.trim();
    if (!trimmed) {
      return;
    }
    try {
      await onRespond(trimmed);
      setInputValue("");
    } catch (error) {
      // Log the error so rejections from async handlers are not unhandled.
      console.error("Failed to send TTY response:", error);
    }
  };

  const handleOptionClick = async (option: string) => {
    try {
      await onRespond(option);
    } catch (error) {
      // Log the error so rejections from async handlers are not unhandled.
      console.error("Failed to send TTY option response:", error);
    }
  };

  return (
    <Card className="mb-4 border-purple-500/50 bg-purple-500/5">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm flex items-center gap-2">
          <MessageSquareIcon size={16} className="text-purple-500" />
          Agent Question
        </CardTitle>
        <CardDescription>{question}</CardDescription>
      </CardHeader>
      <CardContent>
        {options && options.length > 0 ? (
          <div className="flex flex-wrap gap-2">
            {options.map((option) => (
              <Button
                key={option}
                variant="outline"
                size="sm"
                disabled={isResponding}
                onClick={() => handleOptionClick(option)}
              >
                {isResponding ? <LoaderIcon size={12} className="animate-spin mr-2" /> : null}
                {option}
              </Button>
            ))}
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="flex gap-2">
            <Input
              value={inputValue}
              onChange={(e) => setInputValue(e.target.value)}
              placeholder="Type your response..."
              disabled={isResponding}
              autoFocus
            />
            <Button type="submit" disabled={isResponding || !inputValue.trim()}>
              {isResponding ? <LoaderIcon size={16} className="animate-spin" /> : "Send"}
            </Button>
          </form>
        )}
      </CardContent>
    </Card>
  );
}
