import type { ErrorOutputEvent } from "@/api/types";

interface ErrorOutputMessageProps {
  event: ErrorOutputEvent;
}

export function ErrorOutputMessage({ event }: ErrorOutputMessageProps) {
  return (
    <pre className="whitespace-pre-wrap break-words text-destructive">
      {event.content}
    </pre>
  );
}
