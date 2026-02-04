import type { RawEvent } from "@/api/types";

interface RawMessageProps {
  event: RawEvent;
}

export function RawMessage({ event }: RawMessageProps) {
  return (
    <pre className="whitespace-pre-wrap break-words opacity-70">
      {event.content}
    </pre>
  );
}
