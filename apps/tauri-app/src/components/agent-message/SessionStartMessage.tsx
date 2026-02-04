import type { SessionStartEvent } from "@/api/types";
import { Badge } from "@/components/ui/Badge";

interface SessionStartMessageProps {
  event: SessionStartEvent;
}

export function SessionStartMessage({ event }: SessionStartMessageProps) {
  return (
    <div className="flex items-center gap-2 text-muted-foreground">
      <span>Session started</span>
      <Badge variant="secondary" className="text-xs">
        {event.agent_type}
      </Badge>
      {event.model && (
        <Badge variant="outline" className="text-xs">
          {event.model}
        </Badge>
      )}
    </div>
  );
}
