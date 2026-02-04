import { useState } from "react";
import type { CommandExecutionEvent } from "@/api/types";
import { cn } from "@/lib/utils";
import { ChevronDownIcon, ChevronRightIcon } from "@/components/ui/Icons";

interface CommandExecutionMessageProps {
  event: CommandExecutionEvent;
}

export function CommandExecutionMessage({
  event,
}: CommandExecutionMessageProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasOutput = !!event.output?.trim();
  const isSuccess = event.exit_code === 0 || event.exit_code === undefined;

  return (
    <div className="space-y-1">
      <div
        className={cn("flex items-start gap-1", hasOutput && "cursor-pointer")}
        onClick={() => hasOutput && setIsExpanded(!isExpanded)}
      >
        {hasOutput && (
          <span className="text-muted-foreground mt-0.5">
            {isExpanded ? (
              <ChevronDownIcon size={14} />
            ) : (
              <ChevronRightIcon size={14} />
            )}
          </span>
        )}
        <div className="flex-1">
          <code className="text-yellow-500">{formatCommand(event.command)}</code>
          {event.exit_code !== undefined && (
            <span
              className={cn(
                "ml-2 text-xs",
                isSuccess ? "text-green-500" : "text-destructive"
              )}
            >
              (exit: {event.exit_code})
            </span>
          )}
        </div>
      </div>

      {isExpanded && hasOutput && (
        <div className="ml-4 mt-2">
          <pre
            className={cn(
              "text-xs p-2 rounded overflow-x-auto max-h-60 overflow-y-auto",
              isSuccess ? "bg-muted/50" : "bg-destructive/10"
            )}
          >
            {event.output}
          </pre>
        </div>
      )}
    </div>
  );
}

function formatCommand(command: string): string {
  // Show truncated version for very long commands
  if (command.length > 80 && !command.includes("\n")) {
    return command.substring(0, 77) + "...";
  }
  // For multi-line commands, show first line with indicator
  const lines = command.split("\n");
  if (lines.length > 1) {
    const firstLine = lines[0];
    if (firstLine.length > 60) {
      return firstLine.substring(0, 57) + "... (+more)";
    }
    return firstLine + " (+more)";
  }
  return command;
}
