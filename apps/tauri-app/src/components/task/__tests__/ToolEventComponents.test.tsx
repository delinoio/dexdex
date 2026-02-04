import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { ToolUseContent, ToolResultContent } from "../ToolEventComponents";

describe("ToolUseContent", () => {
  it("renders Read tool with file path", () => {
    render(
      <ToolUseContent
        toolName="Read"
        input={{ file_path: "/path/to/file.ts" }}
      />
    );

    expect(screen.getByText("Reading")).toBeInTheDocument();
    expect(screen.getByText("/path/to/file.ts")).toBeInTheDocument();
  });

  it("renders Read tool with offset and limit", () => {
    render(
      <ToolUseContent
        toolName="Read"
        input={{ file_path: "/path/to/file.ts", offset: 10, limit: 50 }}
      />
    );

    expect(screen.getByText("Reading")).toBeInTheDocument();
    expect(screen.getByText(/from line 10/)).toBeInTheDocument();
    expect(screen.getByText(/50 lines/)).toBeInTheDocument();
  });

  it("renders Write tool with file path and content preview", () => {
    render(
      <ToolUseContent
        toolName="Write"
        input={{
          file_path: "/path/to/new-file.ts",
          content: "export const test = 'hello';",
        }}
      />
    );

    expect(screen.getByText("Writing")).toBeInTheDocument();
    expect(screen.getByText("/path/to/new-file.ts")).toBeInTheDocument();
    expect(screen.getByText(/Show content/)).toBeInTheDocument();
  });

  it("renders Edit tool with diff view", () => {
    render(
      <ToolUseContent
        toolName="Edit"
        input={{
          file_path: "/path/to/file.ts",
          old_string: "const old = true;",
          new_string: "const new = true;",
        }}
      />
    );

    expect(screen.getByText("Editing")).toBeInTheDocument();
    expect(screen.getByText("/path/to/file.ts")).toBeInTheDocument();
    expect(screen.getByText("const old = true;")).toBeInTheDocument();
    expect(screen.getByText("const new = true;")).toBeInTheDocument();
  });

  it("renders Edit tool with replace_all flag", () => {
    render(
      <ToolUseContent
        toolName="Edit"
        input={{
          file_path: "/path/to/file.ts",
          old_string: "old",
          new_string: "new",
          replace_all: true,
        }}
      />
    );

    expect(screen.getByText("replace all")).toBeInTheDocument();
  });

  it("renders Bash tool with command", () => {
    render(
      <ToolUseContent
        toolName="Bash"
        input={{ command: "npm install" }}
      />
    );

    expect(screen.getByText("Running")).toBeInTheDocument();
    expect(screen.getByText("$ npm install")).toBeInTheDocument();
  });

  it("renders Bash tool with description", () => {
    render(
      <ToolUseContent
        toolName="Bash"
        input={{
          command: "npm test",
          description: "Run tests",
        }}
      />
    );

    expect(screen.getByText("(Run tests)")).toBeInTheDocument();
  });

  it("renders Glob tool with pattern", () => {
    render(
      <ToolUseContent
        toolName="Glob"
        input={{ pattern: "**/*.ts" }}
      />
    );

    expect(screen.getByText("Finding files matching")).toBeInTheDocument();
    expect(screen.getByText("**/*.ts")).toBeInTheDocument();
  });

  it("renders Glob tool with pattern and path", () => {
    render(
      <ToolUseContent
        toolName="Glob"
        input={{ pattern: "*.ts", path: "/src" }}
      />
    );

    expect(screen.getByText("*.ts")).toBeInTheDocument();
    expect(screen.getByText("/src")).toBeInTheDocument();
  });

  it("renders Grep tool with pattern", () => {
    render(
      <ToolUseContent
        toolName="Grep"
        input={{ pattern: "function\\s+\\w+" }}
      />
    );

    expect(screen.getByText("Searching for")).toBeInTheDocument();
    expect(screen.getByText("function\\s+\\w+")).toBeInTheDocument();
  });

  it("renders WebSearch tool with query", () => {
    render(
      <ToolUseContent
        toolName="WebSearch"
        input={{ query: "React best practices" }}
      />
    );

    expect(screen.getByText("Searching web for")).toBeInTheDocument();
    expect(screen.getByText(/React best practices/)).toBeInTheDocument();
  });

  it("renders Task tool with agent info", () => {
    render(
      <ToolUseContent
        toolName="Task"
        input={{
          description: "Search for files",
          prompt: "Find all TypeScript files",
          subagent_type: "Explore",
        }}
      />
    );

    expect(screen.getByText("Spawning")).toBeInTheDocument();
    expect(screen.getByText("Explore")).toBeInTheDocument();
    expect(screen.getByText("Search for files")).toBeInTheDocument();
  });

  it("renders TodoWrite tool with todos", () => {
    render(
      <ToolUseContent
        toolName="TodoWrite"
        input={{
          todos: [
            { content: "First task", status: "completed" },
            { content: "Second task", status: "in_progress" },
            { content: "Third task", status: "pending" },
          ],
        }}
      />
    );

    expect(screen.getByText("Updating todo list:")).toBeInTheDocument();
    expect(screen.getByText("First task")).toBeInTheDocument();
    expect(screen.getByText("Second task")).toBeInTheDocument();
    expect(screen.getByText("Third task")).toBeInTheDocument();
  });

  it("renders unknown tool with JSON fallback", () => {
    render(
      <ToolUseContent
        toolName="UnknownTool"
        input={{ custom: "value", nested: { key: "data" } }}
      />
    );

    expect(screen.getByText("UnknownTool")).toBeInTheDocument();
    expect(screen.getByText(/"custom": "value"/)).toBeInTheDocument();
  });

  it("handles invalid input gracefully", () => {
    render(
      <ToolUseContent
        toolName="Read"
        input={{ invalid: "input" }}
      />
    );

    // Should fall back to default JSON display
    expect(screen.getByText("Read")).toBeInTheDocument();
    expect(screen.getByText(/"invalid": "input"/)).toBeInTheDocument();
  });
});

describe("ToolResultContent", () => {
  it("renders Read result with file content", () => {
    render(
      <ToolResultContent
        toolName="Read"
        output={"line 1\nline 2\nline 3"}
        isError={false}
      />
    );

    expect(screen.getByText(/File content/)).toBeInTheDocument();
    expect(screen.getByText(/line 1/)).toBeInTheDocument();
  });

  it("renders Bash result with output", () => {
    render(
      <ToolResultContent
        toolName="Bash"
        output="Command output here"
        isError={false}
      />
    );

    expect(screen.getByText("Command output here")).toBeInTheDocument();
  });

  it("renders Bash result with empty output", () => {
    render(
      <ToolResultContent
        toolName="Bash"
        output=""
        isError={false}
      />
    );

    expect(screen.getByText("(no output)")).toBeInTheDocument();
  });

  it("renders Glob result with file count", () => {
    render(
      <ToolResultContent
        toolName="Glob"
        output={"file1.ts\nfile2.ts\nfile3.ts"}
        isError={false}
      />
    );

    expect(screen.getByText(/Found/)).toBeInTheDocument();
    expect(screen.getAllByText(/file/).length).toBeGreaterThan(0);
  });

  it("renders Grep result with match count", () => {
    render(
      <ToolResultContent
        toolName="Grep"
        output={"match1\nmatch2"}
        isError={false}
      />
    );

    expect(screen.getAllByText(/match/).length).toBeGreaterThan(0);
  });

  it("renders error output correctly", () => {
    render(
      <ToolResultContent
        toolName="Bash"
        output="Command failed: exit code 1"
        isError={true}
      />
    );

    expect(screen.getByText("Error:")).toBeInTheDocument();
    expect(screen.getByText("Command failed: exit code 1")).toBeInTheDocument();
  });

  it("handles object output", () => {
    render(
      <ToolResultContent
        toolName="UnknownTool"
        output={{ result: "data", count: 5 }}
        isError={false}
      />
    );

    expect(screen.getByText(/"result": "data"/)).toBeInTheDocument();
    expect(screen.getByText(/"count": 5/)).toBeInTheDocument();
  });
});
