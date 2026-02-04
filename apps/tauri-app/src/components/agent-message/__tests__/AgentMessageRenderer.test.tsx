import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { AgentMessageRenderer } from "../AgentMessageRenderer";
import type {
  TextOutputEvent,
  ErrorOutputEvent,
  ToolUseEvent,
  ToolResultEvent,
  FileChangeEvent,
  CommandExecutionEvent,
  AskUserQuestionEvent,
  UserResponseEvent,
  SessionStartEvent,
  SessionEndEvent,
  ThinkingEvent,
  RawEvent,
  FileChangeType,
} from "@/api/types";

describe("AgentMessageRenderer", () => {
  it("renders text_output event", () => {
    const event: TextOutputEvent = {
      type: "text_output",
      content: "Hello World",
      stream: false,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Hello World")).toBeInTheDocument();
  });

  it("renders error_output event with destructive styling", () => {
    const event: ErrorOutputEvent = {
      type: "error_output",
      content: "Error occurred",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Error occurred")).toBeInTheDocument();
    expect(screen.getByText("Error occurred")).toHaveClass("text-destructive");
  });

  it("renders tool_use event with tool name", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Read",
      input: { file_path: "/test/file.ts" },
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Read")).toBeInTheDocument();
    expect(screen.getByText("/test/file.ts")).toBeInTheDocument();
  });

  it("renders tool_result event with success indicator", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Read",
      output: "File content here",
      is_error: false,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Read (success)")).toBeInTheDocument();
  });

  it("renders tool_result event with error indicator", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Read",
      output: "File not found",
      is_error: true,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Read (error)")).toBeInTheDocument();
  });

  it("renders file_change event with path", () => {
    const event: FileChangeEvent = {
      type: "file_change",
      path: "src/components/Test.tsx",
      change_type: "create" as FileChangeType,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Create")).toBeInTheDocument();
    expect(screen.getByText("src/components/Test.tsx")).toBeInTheDocument();
  });

  it("renders command_execution event", () => {
    const event: CommandExecutionEvent = {
      type: "command_execution",
      command: "npm test",
      exit_code: 0,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("npm test")).toBeInTheDocument();
    expect(screen.getByText("(exit: 0)")).toBeInTheDocument();
  });

  it("renders ask_user_question event", () => {
    const event: AskUserQuestionEvent = {
      type: "ask_user_question",
      question: "What framework do you prefer?",
      options: ["React", "Vue"],
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText(/What framework do you prefer\?/)).toBeInTheDocument();
    expect(screen.getByText("React")).toBeInTheDocument();
    expect(screen.getByText("Vue")).toBeInTheDocument();
  });

  it("renders user_response event", () => {
    const event: UserResponseEvent = {
      type: "user_response",
      response: "React",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Response:")).toBeInTheDocument();
    expect(screen.getByText("React")).toBeInTheDocument();
  });

  it("renders session_start event", () => {
    const event: SessionStartEvent = {
      type: "session_start",
      agent_type: "claude_code",
      model: "claude-sonnet-4-20250514",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Session started")).toBeInTheDocument();
    expect(screen.getByText("claude_code")).toBeInTheDocument();
    expect(screen.getByText("claude-sonnet-4-20250514")).toBeInTheDocument();
  });

  it("renders session_end event with success", () => {
    const event: SessionEndEvent = {
      type: "session_end",
      success: true,
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Session completed successfully")).toBeInTheDocument();
  });

  it("renders session_end event with failure", () => {
    const event: SessionEndEvent = {
      type: "session_end",
      success: false,
      error: "Connection timeout",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Session failed")).toBeInTheDocument();
    expect(screen.getByText(": Connection timeout")).toBeInTheDocument();
  });

  it("renders thinking event", () => {
    const event: ThinkingEvent = {
      type: "thinking",
      content: "Let me analyze this...",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Thinking...")).toBeInTheDocument();
  });

  it("renders raw event", () => {
    const event: RawEvent = {
      type: "raw",
      content: "Some raw output",
    };
    render(<AgentMessageRenderer event={event} />);
    expect(screen.getByText("Some raw output")).toBeInTheDocument();
  });
});
