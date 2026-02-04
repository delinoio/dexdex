import type { UserResponseEvent } from "@/api/types";

interface UserResponseMessageProps {
  event: UserResponseEvent;
}

export function UserResponseMessage({ event }: UserResponseMessageProps) {
  return (
    <div>
      <span className="text-purple-500 font-medium">Response:</span>{" "}
      <span>{event.response}</span>
    </div>
  );
}
