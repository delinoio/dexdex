import type { NormalizedEvent } from "@/api/types";
import { TextOutputMessage } from "./TextOutputMessage";
import { ErrorOutputMessage } from "./ErrorOutputMessage";
import { ToolUseMessage } from "./ToolUseMessage";
import { ToolResultMessage } from "./ToolResultMessage";
import { FileChangeMessage } from "./FileChangeMessage";
import { CommandExecutionMessage } from "./CommandExecutionMessage";
import { AskUserQuestionMessage } from "./AskUserQuestionMessage";
import { UserResponseMessage } from "./UserResponseMessage";
import { SessionStartMessage } from "./SessionStartMessage";
import { SessionEndMessage } from "./SessionEndMessage";
import { ThinkingMessage } from "./ThinkingMessage";
import { RawMessage } from "./RawMessage";

interface AgentMessageRendererProps {
  event: NormalizedEvent;
}

export function AgentMessageRenderer({ event }: AgentMessageRendererProps) {
  switch (event.type) {
    case "text_output":
      return <TextOutputMessage event={event} />;

    case "error_output":
      return <ErrorOutputMessage event={event} />;

    case "tool_use":
      return <ToolUseMessage event={event} />;

    case "tool_result":
      return <ToolResultMessage event={event} />;

    case "file_change":
      return <FileChangeMessage event={event} />;

    case "command_execution":
      return <CommandExecutionMessage event={event} />;

    case "ask_user_question":
      return <AskUserQuestionMessage event={event} />;

    case "user_response":
      return <UserResponseMessage event={event} />;

    case "session_start":
      return <SessionStartMessage event={event} />;

    case "session_end":
      return <SessionEndMessage event={event} />;

    case "thinking":
      return <ThinkingMessage event={event} />;

    case "raw":
      return <RawMessage event={event} />;

    default:
      return <span className="text-muted-foreground">Unknown event</span>;
  }
}
