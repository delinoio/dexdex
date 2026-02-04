import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ToolUseMessage } from "../ToolUseMessage";
import type { ToolUseEvent } from "@/api/types";

describe("ToolUseMessage", () => {
  it("renders tool name", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Read",
      input: { file_path: "/path/to/file.ts" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText("Read")).toBeInTheDocument();
  });

  it("displays file path summary for Read tool", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Read",
      input: { file_path: "/home/user/project/src/file.ts" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText(".../project/src/file.ts")).toBeInTheDocument();
  });

  it("displays command summary for Bash tool", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Bash",
      input: { command: "npm install" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText("npm install")).toBeInTheDocument();
  });

  it("truncates long commands", () => {
    const longCommand = "a".repeat(100);
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Bash",
      input: { command: longCommand },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText(longCommand.substring(0, 60) + "...")).toBeInTheDocument();
  });

  it("displays pattern summary for Glob tool", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Glob",
      input: { pattern: "**/*.ts" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText("**/*.ts")).toBeInTheDocument();
  });

  it("displays query for WebSearch tool", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "WebSearch",
      input: { query: "react hooks tutorial" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText("react hooks tutorial")).toBeInTheDocument();
  });

  it("expands Edit tool details on click", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Edit",
      input: {
        file_path: "/path/to/file.ts",
        old_string: "const foo = 1;",
        new_string: "const bar = 2;",
      },
    };
    render(<ToolUseMessage event={event} />);

    // Initially, details should not be visible
    expect(screen.queryByText("- Old:")).not.toBeInTheDocument();

    // Click to expand
    fireEvent.click(screen.getByText("Edit"));

    // Now details should be visible
    expect(screen.getByText("- Old:")).toBeInTheDocument();
    expect(screen.getByText("+ New:")).toBeInTheDocument();
    expect(screen.getByText("const foo = 1;")).toBeInTheDocument();
    expect(screen.getByText("const bar = 2;")).toBeInTheDocument();
  });

  it("expands Write tool details on click", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "Write",
      input: {
        file_path: "/path/to/new-file.ts",
        content: "export const hello = 'world';",
      },
    };
    render(<ToolUseMessage event={event} />);

    // Click to expand
    fireEvent.click(screen.getByText("Write"));

    // Details should be visible
    expect(screen.getByText("Content:")).toBeInTheDocument();
    expect(screen.getByText("export const hello = 'world';")).toBeInTheDocument();
  });

  it("displays URL hostname for WebFetch tool", () => {
    const event: ToolUseEvent = {
      type: "tool_use",
      tool_name: "WebFetch",
      input: { url: "https://example.com/api/data" },
    };
    render(<ToolUseMessage event={event} />);
    expect(screen.getByText("example.com/api/data")).toBeInTheDocument();
  });
});
