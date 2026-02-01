import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { KanbanBoard } from "../KanbanBoard";
import { UnitTaskStatus, CompositeTaskStatus } from "@/api/types";
import type { UnitTask, CompositeTask } from "@/api/types";

const createMockUnitTask = (overrides?: Partial<UnitTask>): UnitTask => ({
  id: "unit-1",
  repositoryGroupId: "repo-group-1",
  agentTaskId: "agent-1",
  prompt: "Test prompt",
  title: "Test Task",
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
  prompt: "Test prompt",
  title: "Test Composite",
  nodeIds: [],
  status: CompositeTaskStatus.Planning,
  createdAt: "2024-01-15T10:00:00Z",
  updatedAt: "2024-01-15T12:00:00Z",
  ...overrides,
});

describe("KanbanBoard", () => {
  it("renders all columns", () => {
    render(<KanbanBoard unitTasks={[]} compositeTasks={[]} />);

    expect(screen.getByText("In Progress")).toBeInTheDocument();
    expect(screen.getByText("In Review")).toBeInTheDocument();
    expect(screen.getByText("PR Open")).toBeInTheDocument();
    expect(screen.getByText("Done")).toBeInTheDocument();
    expect(screen.getByText("Rejected")).toBeInTheDocument();
  });

  it("places unit tasks in correct columns based on status", () => {
    const tasks: UnitTask[] = [
      createMockUnitTask({ id: "1", title: "In Progress Task", status: UnitTaskStatus.InProgress }),
      createMockUnitTask({ id: "2", title: "In Review Task", status: UnitTaskStatus.InReview }),
      createMockUnitTask({ id: "3", title: "PR Open Task", status: UnitTaskStatus.PrOpen }),
      createMockUnitTask({ id: "4", title: "Done Task", status: UnitTaskStatus.Done }),
      createMockUnitTask({ id: "5", title: "Rejected Task", status: UnitTaskStatus.Rejected }),
    ];

    render(<KanbanBoard unitTasks={tasks} compositeTasks={[]} />);

    expect(screen.getByText("In Progress Task")).toBeInTheDocument();
    expect(screen.getByText("In Review Task")).toBeInTheDocument();
    expect(screen.getByText("PR Open Task")).toBeInTheDocument();
    expect(screen.getByText("Done Task")).toBeInTheDocument();
    expect(screen.getByText("Rejected Task")).toBeInTheDocument();
  });

  it("places composite tasks in correct columns based on status", () => {
    const tasks: CompositeTask[] = [
      createMockCompositeTask({ id: "1", title: "Planning Task", status: CompositeTaskStatus.Planning }),
      createMockCompositeTask({ id: "2", title: "Pending Approval Task", status: CompositeTaskStatus.PendingApproval }),
      createMockCompositeTask({ id: "3", title: "Done Composite", status: CompositeTaskStatus.Done }),
      createMockCompositeTask({ id: "4", title: "Rejected Composite", status: CompositeTaskStatus.Rejected }),
    ];

    render(<KanbanBoard unitTasks={[]} compositeTasks={tasks} />);

    expect(screen.getByText("Planning Task")).toBeInTheDocument();
    expect(screen.getByText("Pending Approval Task")).toBeInTheDocument();
    expect(screen.getByText("Done Composite")).toBeInTheDocument();
    expect(screen.getByText("Rejected Composite")).toBeInTheDocument();
  });

  it("shows 'No tasks' message for empty columns", () => {
    render(<KanbanBoard unitTasks={[]} compositeTasks={[]} />);

    const noTasksMessages = screen.getAllByText("No tasks");
    expect(noTasksMessages.length).toBe(5); // All 5 columns should show "No tasks"
  });

  it("calls onTaskClick with correct arguments for unit task", () => {
    const onTaskClick = vi.fn();
    const task = createMockUnitTask({ id: "test-unit-id" });

    render(<KanbanBoard unitTasks={[task]} compositeTasks={[]} onTaskClick={onTaskClick} />);

    fireEvent.click(screen.getByRole("button", { name: /Test Task/i }));
    expect(onTaskClick).toHaveBeenCalledWith("test-unit-id", true);
  });

  it("calls onTaskClick with correct arguments for composite task", () => {
    const onTaskClick = vi.fn();
    const task = createMockCompositeTask({ id: "test-composite-id" });

    render(<KanbanBoard unitTasks={[]} compositeTasks={[task]} onTaskClick={onTaskClick} />);

    fireEvent.click(screen.getByRole("button", { name: /Test Composite/i }));
    expect(onTaskClick).toHaveBeenCalledWith("test-composite-id", false);
  });

  it("displays correct task counts in column headers", () => {
    const unitTasks: UnitTask[] = [
      createMockUnitTask({ id: "1", status: UnitTaskStatus.InProgress }),
      createMockUnitTask({ id: "2", status: UnitTaskStatus.InProgress }),
    ];

    render(<KanbanBoard unitTasks={unitTasks} compositeTasks={[]} />);

    // The count "2" should be displayed in the In Progress column header
    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("sorts tasks by updatedAt date descending", () => {
    const olderTask = createMockUnitTask({
      id: "1",
      title: "Older Task",
      status: UnitTaskStatus.InProgress,
      updatedAt: "2024-01-10T10:00:00Z",
    });
    const newerTask = createMockUnitTask({
      id: "2",
      title: "Newer Task",
      status: UnitTaskStatus.InProgress,
      updatedAt: "2024-01-15T10:00:00Z",
    });

    render(<KanbanBoard unitTasks={[olderTask, newerTask]} compositeTasks={[]} />);

    const taskCards = screen.getAllByRole("button");
    // Newer task should appear before older task
    const newerIndex = taskCards.findIndex((el) => el.textContent?.includes("Newer Task"));
    const olderIndex = taskCards.findIndex((el) => el.textContent?.includes("Older Task"));
    expect(newerIndex).toBeLessThan(olderIndex);
  });

  it("handles mixed unit and composite tasks in same column", () => {
    const unitTask = createMockUnitTask({
      id: "unit-1",
      title: "Unit in Progress",
      status: UnitTaskStatus.InProgress,
    });
    const compositeTask = createMockCompositeTask({
      id: "composite-1",
      title: "Composite in Progress",
      status: CompositeTaskStatus.InProgress,
    });

    render(<KanbanBoard unitTasks={[unitTask]} compositeTasks={[compositeTask]} />);

    expect(screen.getByText("Unit in Progress")).toBeInTheDocument();
    expect(screen.getByText("Composite in Progress")).toBeInTheDocument();
  });
});
