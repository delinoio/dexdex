import type { SessionEndEvent } from "@/api/types";
import { cn } from "@/lib/utils";
import { CheckCircleIcon, XCircleIcon } from "@/components/ui/Icons";

interface SessionEndMessageProps {
  event: SessionEndEvent;
}

export function SessionEndMessage({ event }: SessionEndMessageProps) {
  const isSuccess = event.success;

  return (
    <div
      className={cn(
        "flex items-center gap-2",
        isSuccess ? "text-green-500" : "text-destructive"
      )}
    >
      {isSuccess ? <CheckCircleIcon size={16} /> : <XCircleIcon size={16} />}
      <span>
        Session {isSuccess ? "completed successfully" : "failed"}
      </span>
      {!isSuccess && event.error && (
        <span className="text-sm opacity-80">: {event.error}</span>
      )}
    </div>
  );
}
