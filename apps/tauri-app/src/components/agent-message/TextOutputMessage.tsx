import type { TextOutputEvent } from "@/api/types";

interface TextOutputMessageProps {
  event: TextOutputEvent;
}

export function TextOutputMessage({ event }: TextOutputMessageProps) {
  return <pre className="whitespace-pre-wrap break-words">{event.content}</pre>;
}
