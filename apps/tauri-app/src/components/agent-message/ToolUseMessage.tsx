import { useState } from "react";
import type { ToolUseEvent } from "@/api/types";
import { cn } from "@/lib/utils";
import { ChevronDownIcon, ChevronRightIcon } from "@/components/ui/Icons";

interface ToolUseMessageProps {
  event: ToolUseEvent;
}

// Common tool parameter types for better display
interface ReadToolInput {
  file_path?: string;
  path?: string;
}

interface WriteToolInput {
  file_path?: string;
  path?: string;
  content?: string;
}

interface EditToolInput {
  file_path?: string;
  path?: string;
  old_string?: string;
  new_string?: string;
  old_text?: string;
  new_text?: string;
}

interface BashToolInput {
  command?: string;
}

interface GlobToolInput {
  pattern?: string;
  path?: string;
}

interface GrepToolInput {
  pattern?: string;
  path?: string;
  include?: string;
}

interface WebFetchToolInput {
  url?: string;
}

interface WebSearchToolInput {
  query?: string;
}

export function ToolUseMessage({ event }: ToolUseMessageProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const toolName = event.tool_name;
  const input = event.input as Record<string, unknown>;

  const summary = getToolSummary(toolName, input);
  const hasDetails = hasDetailedContent(toolName, input);

  return (
    <div className="space-y-1">
      <div
        className={cn(
          "flex items-start gap-1",
          hasDetails && "cursor-pointer"
        )}
        onClick={() => hasDetails && setIsExpanded(!isExpanded)}
      >
        {hasDetails && (
          <span className="text-muted-foreground mt-0.5">
            {isExpanded ? (
              <ChevronDownIcon size={14} />
            ) : (
              <ChevronRightIcon size={14} />
            )}
          </span>
        )}
        <div className="flex-1">
          <span className="text-blue-500 font-medium">{toolName}</span>
          {summary && (
            <span className="ml-2 text-muted-foreground">{summary}</span>
          )}
        </div>
      </div>

      {isExpanded && hasDetails && (
        <div className="ml-4 mt-2 space-y-2">
          {renderToolDetails(toolName, input)}
        </div>
      )}
    </div>
  );
}

function getToolSummary(
  toolName: string,
  input: Record<string, unknown>
): string | null {
  switch (toolName.toLowerCase()) {
    case "read": {
      const readInput = input as ReadToolInput;
      const path = readInput.file_path || readInput.path;
      return path ? formatPath(path) : null;
    }

    case "write": {
      const writeInput = input as WriteToolInput;
      const path = writeInput.file_path || writeInput.path;
      return path ? formatPath(path) : null;
    }

    case "edit": {
      const editInput = input as EditToolInput;
      const path = editInput.file_path || editInput.path;
      return path ? formatPath(path) : null;
    }

    case "bash": {
      const bashInput = input as BashToolInput;
      if (bashInput.command) {
        const cmd = bashInput.command;
        return cmd.length > 60 ? cmd.substring(0, 60) + "..." : cmd;
      }
      return null;
    }

    case "glob": {
      const globInput = input as GlobToolInput;
      return globInput.pattern || null;
    }

    case "grep": {
      const grepInput = input as GrepToolInput;
      if (grepInput.pattern) {
        const pattern = grepInput.pattern;
        return pattern.length > 40
          ? pattern.substring(0, 40) + "..."
          : pattern;
      }
      return null;
    }

    case "webfetch":
    case "web_fetch": {
      const fetchInput = input as WebFetchToolInput;
      if (fetchInput.url) {
        try {
          const url = new URL(fetchInput.url);
          return url.hostname + url.pathname;
        } catch {
          return fetchInput.url.substring(0, 50);
        }
      }
      return null;
    }

    case "websearch":
    case "web_search": {
      const searchInput = input as WebSearchToolInput;
      return searchInput.query || null;
    }

    default:
      return null;
  }
}

function hasDetailedContent(
  toolName: string,
  input: Record<string, unknown>
): boolean {
  switch (toolName.toLowerCase()) {
    case "write":
      return !!(input as WriteToolInput).content;
    case "edit":
      return !!(
        (input as EditToolInput).old_string ||
        (input as EditToolInput).old_text
      );
    case "bash":
      return !!((input as BashToolInput).command?.includes("\n"));
    default:
      // Show details if there are more than 2 keys or complex nested objects
      const keys = Object.keys(input);
      if (keys.length > 2) return true;
      for (const key of keys) {
        if (typeof input[key] === "object" && input[key] !== null) return true;
      }
      return false;
  }
}

function renderToolDetails(
  toolName: string,
  input: Record<string, unknown>
): React.ReactNode {
  switch (toolName.toLowerCase()) {
    case "write": {
      const writeInput = input as WriteToolInput;
      if (writeInput.content) {
        return (
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Content:</div>
            <pre className="text-xs bg-muted/50 p-2 rounded overflow-x-auto max-h-60 overflow-y-auto">
              {writeInput.content}
            </pre>
          </div>
        );
      }
      return null;
    }

    case "edit": {
      const editInput = input as EditToolInput;
      const oldText = editInput.old_string || editInput.old_text;
      const newText = editInput.new_string || editInput.new_text;
      return (
        <div className="space-y-2">
          {oldText && (
            <div className="space-y-1">
              <div className="text-xs text-red-400">- Old:</div>
              <pre className="text-xs bg-red-500/10 p-2 rounded overflow-x-auto max-h-40 overflow-y-auto border-l-2 border-red-500/50">
                {oldText}
              </pre>
            </div>
          )}
          {newText && (
            <div className="space-y-1">
              <div className="text-xs text-green-400">+ New:</div>
              <pre className="text-xs bg-green-500/10 p-2 rounded overflow-x-auto max-h-40 overflow-y-auto border-l-2 border-green-500/50">
                {newText}
              </pre>
            </div>
          )}
        </div>
      );
    }

    case "bash": {
      const bashInput = input as BashToolInput;
      if (bashInput.command) {
        return (
          <div className="space-y-1">
            <div className="text-xs text-muted-foreground">Command:</div>
            <pre className="text-xs bg-muted/50 p-2 rounded overflow-x-auto">
              {bashInput.command}
            </pre>
          </div>
        );
      }
      return null;
    }

    default:
      // Generic JSON display for unknown tools
      return (
        <pre className="text-xs text-muted-foreground bg-muted/50 p-2 rounded overflow-x-auto max-h-60 overflow-y-auto">
          {JSON.stringify(input, null, 2)}
        </pre>
      );
  }
}

function formatPath(path: string): string {
  // Show only the last 2-3 path segments for brevity
  const segments = path.split("/").filter(Boolean);
  if (segments.length <= 3) {
    return path;
  }
  return ".../" + segments.slice(-3).join("/");
}
