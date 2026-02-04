import { useState } from "react";
import type { ToolResultEvent } from "@/api/types";
import { cn } from "@/lib/utils";
import { ChevronDownIcon, ChevronRightIcon } from "@/components/ui/Icons";

interface ToolResultMessageProps {
  event: ToolResultEvent;
}

export function ToolResultMessage({ event }: ToolResultMessageProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const output = event.output;
  const isError = event.is_error;

  const { summary, hasMore, content } = processOutput(output, isError);

  return (
    <div className="space-y-1">
      <div
        className={cn("flex items-start gap-1", hasMore && "cursor-pointer")}
        onClick={() => hasMore && setIsExpanded(!isExpanded)}
      >
        {hasMore && (
          <span className="text-muted-foreground mt-0.5">
            {isExpanded ? (
              <ChevronDownIcon size={14} />
            ) : (
              <ChevronRightIcon size={14} />
            )}
          </span>
        )}
        <div className="flex-1">
          <span
            className={cn("font-medium", isError ? "text-destructive" : "text-green-500")}
          >
            {event.tool_name} {isError ? "(error)" : "(success)"}
          </span>
          {summary && (
            <span className="ml-2 text-muted-foreground text-sm">
              {summary}
            </span>
          )}
        </div>
      </div>

      {isExpanded && hasMore && content && (
        <div className="ml-4 mt-2">
          <pre
            className={cn(
              "text-xs p-2 rounded overflow-x-auto max-h-60 overflow-y-auto",
              isError ? "bg-destructive/10" : "bg-muted/50"
            )}
          >
            {content}
          </pre>
        </div>
      )}
    </div>
  );
}

interface ProcessedOutput {
  summary: string | null;
  hasMore: boolean;
  content: string | null;
}

function processOutput(
  output: unknown,
  isError: boolean
): ProcessedOutput {
  if (output === null || output === undefined) {
    return { summary: null, hasMore: false, content: null };
  }

  if (typeof output === "string") {
    const trimmed = output.trim();
    if (trimmed.length === 0) {
      return { summary: null, hasMore: false, content: null };
    }

    const lines = trimmed.split("\n");
    const firstLine = lines[0];

    if (lines.length === 1 && firstLine.length <= 80) {
      // Short single line - show inline
      return { summary: firstLine, hasMore: false, content: null };
    }

    // Longer content - show summary and allow expansion
    const summaryText =
      firstLine.length > 60 ? firstLine.substring(0, 60) + "..." : firstLine;
    const suffix =
      lines.length > 1 ? ` (+${lines.length - 1} lines)` : "";

    return {
      summary: summaryText + suffix,
      hasMore: true,
      content: trimmed,
    };
  }

  // Object/array output
  const jsonStr = JSON.stringify(output, null, 2);
  const lines = jsonStr.split("\n");

  if (lines.length <= 3 && jsonStr.length <= 100) {
    // Short object - show inline
    return {
      summary: JSON.stringify(output),
      hasMore: false,
      content: null,
    };
  }

  // Larger object - show expandable
  const preview = getObjectPreview(output);
  return {
    summary: preview,
    hasMore: true,
    content: jsonStr,
  };
}

function getObjectPreview(obj: unknown): string {
  if (Array.isArray(obj)) {
    return `Array (${obj.length} items)`;
  }

  if (typeof obj === "object" && obj !== null) {
    const keys = Object.keys(obj);
    if (keys.length <= 3) {
      return `{${keys.join(", ")}}`;
    }
    return `Object (${keys.length} properties)`;
  }

  return String(obj);
}
