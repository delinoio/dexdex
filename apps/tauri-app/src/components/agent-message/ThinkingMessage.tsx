import { useState } from "react";
import type { ThinkingEvent } from "@/api/types";
import { ChevronDownIcon, ChevronRightIcon } from "@/components/ui/Icons";

interface ThinkingMessageProps {
  event: ThinkingEvent;
}

export function ThinkingMessage({ event }: ThinkingMessageProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const hasContent = !!event.content?.trim();

  return (
    <div className="text-cyan-500">
      <div
        className={hasContent ? "cursor-pointer flex items-center gap-1" : ""}
        onClick={() => hasContent && setIsExpanded(!isExpanded)}
      >
        {hasContent && (
          <span className="text-muted-foreground">
            {isExpanded ? (
              <ChevronDownIcon size={14} />
            ) : (
              <ChevronRightIcon size={14} />
            )}
          </span>
        )}
        <span>Thinking...</span>
      </div>
      {isExpanded && hasContent && (
        <pre className="mt-2 text-xs whitespace-pre-wrap opacity-70 bg-cyan-500/10 p-2 rounded max-h-60 overflow-y-auto">
          {event.content}
        </pre>
      )}
    </div>
  );
}
