import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TaskCard } from "../TaskCard";
import { UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import type { UnitTask, CompositeTask } from "@/api/types";

const createMockUnitTask = (overrides?: Partial<UnitTask>): UnitTask => ({
  id: "unit-1",
  repositoryGroupId: "repo-group-1",
  agentTaskId: "agent-1",
  prompt: "Fix the login bug",
  title: "Login Bug Fix",
  status: UnitTaskStatus.InProgress,
  autoFixTaskIds: [],
  createdAt: "2024-01-15T10:00:00Z",
  updatedAt: "2024-01-15T12:00:00Z",
  ...overrides,
});

const createMockCompositeTask = (overrides?: Partial<CompositeTask>): CompositeTask => ({
  id: "composite-1",
  repositoryGroupId: "repo-group-1",
  planningTaskId: "planning-1",
  prompt: "Implement user authentication",
  title: "User Auth Implementation",
  nodeIds: [],
  status: CompositeTaskStatus.Planning,
  createdAt: "2024-01-15T10:00:00Z",
  updatedAt: "2024-01-15T12:00:00Z",
  ...overrides,
});

describe("TaskCard", () => {
  it("renders unit task correctly", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} />);

    expect(screen.getByText("Login Bug Fix")).toBeInTheDocument();
    expect(screen.getByText("In Progress")).toBeInTheDocument();
    expect(screen.getByText("Unit")).toBeInTheDocument();
  });

  it("renders composite task correctly", () => {
    const task = createMockCompositeTask();
    render(<TaskCard task={task} />);

    expect(screen.getByText("User Auth Implementation")).toBeInTheDocument();
    expect(screen.getByText("Planning")).toBeInTheDocument();
    expect(screen.getByText("Composite")).toBeInTheDocument();
  });

  it("uses truncated prompt as title when title is not provided", () => {
    const task = createMockUnitTask({
      title: undefined,
      prompt: "This is a very long prompt that should be truncated when displayed",
    });
    render(<TaskCard task={task} />);

    // Both title area (truncated) and content area (full) show the prompt
    // Using getAllByText since it appears in both places
    const matches = screen.getAllByText(/This is a very long prompt/);
    expect(matches.length).toBeGreaterThanOrEqual(1);
  });

  it("handles click events", () => {
    const onClick = vi.fn();
    const task = createMockUnitTask();
    render(<TaskCard task={task} onClick={onClick} />);

    fireEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("is keyboard accessible - Enter key", () => {
    const onClick = vi.fn();
    const task = createMockUnitTask();
    render(<TaskCard task={task} onClick={onClick} />);

    const card = screen.getByRole("button");
    fireEvent.keyDown(card, { key: "Enter" });
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("is keyboard accessible - Space key", () => {
    const onClick = vi.fn();
    const task = createMockUnitTask();
    render(<TaskCard task={task} onClick={onClick} />);

    const card = screen.getByRole("button");
    fireEvent.keyDown(card, { key: " " });
    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it("has correct aria-label for accessibility", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} />);

    const card = screen.getByRole("button");
    expect(card).toHaveAttribute(
      "aria-label",
      "Unit task: Login Bug Fix. Status: In Progress"
    );
  });

  it("renders different status badges correctly", () => {
    const statuses: { status: UnitTaskStatus; label: string }[] = [
      { status: UnitTaskStatus.InProgress, label: "In Progress" },
      { status: UnitTaskStatus.InReview, label: "In Review" },
      { status: UnitTaskStatus.PrOpen, label: "PR Open" },
      { status: UnitTaskStatus.Done, label: "Done" },
      { status: UnitTaskStatus.Rejected, label: "Rejected" },
      { status: UnitTaskStatus.Approved, label: "Approved" },
    ];

    statuses.forEach(({ status, label }) => {
      const task = createMockUnitTask({ status });
      const { unmount } = render(<TaskCard task={task} />);
      expect(screen.getByText(label)).toBeInTheDocument();
      unmount();
    });
  });

  it("displays creation date correctly", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} />);

    // The date format depends on locale, but it should be present
    expect(screen.getByText(/2024/)).toBeInTheDocument();
  });

  it("displays task prompt in content area", () => {
    const task = createMockUnitTask({ prompt: "Test prompt content" });
    render(<TaskCard task={task} />);

    expect(screen.getByText("Test prompt content")).toBeInTheDocument();
  });

  it("has tabIndex 0 for keyboard focus", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} />);

    expect(screen.getByRole("button")).toHaveAttribute("tabIndex", "0");
  });

  it("applies hover styles class when onClick is provided", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} onClick={() => {}} />);

    const card = screen.getByRole("button");
    expect(card).toHaveClass("hover:border-[hsl(var(--primary))]");
  });

  it("shows dropdown menu trigger when onDelete is provided", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} onDelete={vi.fn()} />);

    expect(screen.getByLabelText("Task actions")).toBeInTheDocument();
  });

  it("does not show dropdown menu trigger when onDelete is not provided", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} />);

    expect(screen.queryByLabelText("Task actions")).not.toBeInTheDocument();
  });

  it("opens dropdown menu and shows delete option on trigger click", () => {
    const task = createMockUnitTask();
    render(<TaskCard task={task} onDelete={vi.fn()} />);

    const trigger = screen.getByLabelText("Task actions");
    fireEvent.click(trigger);

    expect(screen.getByRole("menuitem", { name: /Delete/i })).toBeInTheDocument();
  });

  it("calls onDelete with task id when delete is clicked", () => {
    const onDelete = vi.fn();
    const task = createMockUnitTask({ id: "task-123" });
    render(<TaskCard task={task} onDelete={onDelete} />);

    const trigger = screen.getByLabelText("Task actions");
    fireEvent.click(trigger);

    const deleteItem = screen.getByRole("menuitem", { name: /Delete/i });
    fireEvent.click(deleteItem);

    expect(onDelete).toHaveBeenCalledWith("task-123");
  });

  it("calls onDelete for composite tasks", () => {
    const onDelete = vi.fn();
    const task = createMockCompositeTask({ id: "composite-456" });
    render(<TaskCard task={task} onDelete={onDelete} />);

    const trigger = screen.getByLabelText("Task actions");
    fireEvent.click(trigger);

    const deleteItem = screen.getByRole("menuitem", { name: /Delete/i });
    fireEvent.click(deleteItem);

    expect(onDelete).toHaveBeenCalledWith("composite-456");
  });
});
