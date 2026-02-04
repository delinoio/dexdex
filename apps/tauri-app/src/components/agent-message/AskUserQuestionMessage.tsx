import type { AskUserQuestionEvent } from "@/api/types";
import { Badge } from "@/components/ui/Badge";

interface AskUserQuestionMessageProps {
  event: AskUserQuestionEvent;
}

export function AskUserQuestionMessage({ event }: AskUserQuestionMessageProps) {
  return (
    <div className="text-purple-500 space-y-1">
      <div>
        <span className="font-medium">Question:</span> {event.question}
      </div>
      {event.options && event.options.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-1">
          {event.options.map((option) => (
            <Badge key={option} variant="secondary" className="text-xs">
              {option}
            </Badge>
          ))}
        </div>
      )}
    </div>
  );
}
