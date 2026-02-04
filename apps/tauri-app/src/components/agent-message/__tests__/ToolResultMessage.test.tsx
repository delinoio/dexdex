import { describe, it, expect } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ToolResultMessage } from "../ToolResultMessage";
import type { ToolResultEvent } from "@/api/types";

describe("ToolResultMessage", () => {
  it("renders success indicator for successful result", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Read",
      output: "file content",
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Read (success)")).toBeInTheDocument();
    expect(screen.getByText("Read (success)")).toHaveClass("text-green-500");
  });

  it("renders error indicator for failed result", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Read",
      output: "File not found",
      is_error: true,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Read (error)")).toBeInTheDocument();
    expect(screen.getByText("Read (error)")).toHaveClass("text-destructive");
  });

  it("displays short string output inline", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Bash",
      output: "Done",
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Done")).toBeInTheDocument();
  });

  it("truncates long output with expansion", () => {
    const longOutput = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Read",
      output: longOutput,
      is_error: false,
    };
    const { container } = render(<ToolResultMessage event={event} />);

    // Should show first line with count
    expect(screen.getByText(/Line 1.*\+4 lines/)).toBeInTheDocument();

    // Click to expand
    fireEvent.click(screen.getByText("Read (success)"));

    // Full content should be visible in pre element
    const preElement = container.querySelector("pre");
    expect(preElement).toBeInTheDocument();
    expect(preElement?.textContent).toContain("Line 1");
    expect(preElement?.textContent).toContain("Line 5");
  });

  it("handles object output", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "WebFetch",
      output: { data: "value", count: 5 },
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("{data, count}")).toBeInTheDocument();
  });

  it("handles array output", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Glob",
      output: ["file1.ts", "file2.ts", "file3.ts"],
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Array (3 items)")).toBeInTheDocument();
  });

  it("handles empty output gracefully", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Bash",
      output: "",
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Bash (success)")).toBeInTheDocument();
  });

  it("handles null output gracefully", () => {
    const event: ToolResultEvent = {
      type: "tool_result",
      tool_name: "Bash",
      output: null,
      is_error: false,
    };
    render(<ToolResultMessage event={event} />);
    expect(screen.getByText("Bash (success)")).toBeInTheDocument();
  });
});
