import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { InlineComment, CommentInputForm, LineComments } from "../InlineComment";
import type { ReviewComment } from "@/hooks/useReviewComments";

const mockComment: ReviewComment = {
  id: "comment-1",
  filePath: "src/test.ts",
  lineNumber: 10,
  content: "This is a test comment",
  author: "Test User",
  createdAt: "2026-01-01T10:00:00Z",
  updatedAt: "2026-01-01T10:00:00Z",
};

describe("InlineComment", () => {
  it("renders comment content correctly", () => {
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText("This is a test comment")).toBeInTheDocument();
    expect(screen.getByText("Test User")).toBeInTheDocument();
  });

  it("shows edited indicator when comment was updated", () => {
    const editedComment = {
      ...mockComment,
      updatedAt: "2026-01-01T11:00:00Z",
    };

    render(
      <InlineComment
        comment={editedComment}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText("(edited)")).toBeInTheDocument();
  });

  it("shows confirmation dialog when delete button is clicked", () => {
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /delete/i }));
    expect(screen.getByText("Delete comment?")).toBeInTheDocument();
    expect(screen.getByText(/This action cannot be undone/)).toBeInTheDocument();
  });

  it("calls onDelete when delete is confirmed", () => {
    const handleDelete = vi.fn();
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={handleDelete}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /delete/i }));
    fireEvent.click(screen.getByTestId("confirm-delete-button"));
    expect(handleDelete).toHaveBeenCalledWith("comment-1");
  });

  it("does not call onDelete when delete is cancelled", () => {
    const handleDelete = vi.fn();
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={handleDelete}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /delete/i }));
    // Click the cancel button in the dialog
    const cancelButtons = screen.getAllByRole("button", { name: /cancel/i });
    fireEvent.click(cancelButtons[cancelButtons.length - 1]);
    expect(handleDelete).not.toHaveBeenCalled();
  });

  it("enters edit mode when edit button is clicked", () => {
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    expect(screen.getByTestId("inline-comment-edit-textarea")).toBeInTheDocument();
  });

  it("calls onEdit when save button is clicked in edit mode", () => {
    const handleEdit = vi.fn();
    render(
      <InlineComment
        comment={mockComment}
        onEdit={handleEdit}
        onDelete={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /edit/i }));

    const textarea = screen.getByTestId("inline-comment-edit-textarea");
    fireEvent.change(textarea, { target: { value: "Updated comment" } });

    fireEvent.click(screen.getByRole("button", { name: /save/i }));
    expect(handleEdit).toHaveBeenCalledWith("comment-1", "Updated comment");
  });

  it("cancels edit mode when cancel button is clicked", () => {
    render(
      <InlineComment
        comment={mockComment}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /edit/i }));
    expect(screen.getByTestId("inline-comment-edit-textarea")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(screen.queryByTestId("inline-comment-edit-textarea")).not.toBeInTheDocument();
    expect(screen.getByText("This is a test comment")).toBeInTheDocument();
  });
});

describe("CommentInputForm", () => {
  it("renders with placeholder text", () => {
    render(
      <CommentInputForm
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
        placeholder="Custom placeholder"
      />
    );

    expect(screen.getByPlaceholderText("Custom placeholder")).toBeInTheDocument();
  });

  it("calls onSubmit with content when form is submitted", () => {
    const handleSubmit = vi.fn();
    render(
      <CommentInputForm
        onSubmit={handleSubmit}
        onCancel={vi.fn()}
      />
    );

    const textarea = screen.getByTestId("comment-input-textarea");
    fireEvent.change(textarea, { target: { value: "New comment" } });

    fireEvent.click(screen.getByRole("button", { name: /add comment/i }));
    expect(handleSubmit).toHaveBeenCalledWith("New comment");
  });

  it("disables submit button when content is empty", () => {
    render(
      <CommentInputForm
        onSubmit={vi.fn()}
        onCancel={vi.fn()}
      />
    );

    expect(screen.getByRole("button", { name: /add comment/i })).toBeDisabled();
  });

  it("calls onCancel when cancel button is clicked", () => {
    const handleCancel = vi.fn();
    render(
      <CommentInputForm
        onSubmit={vi.fn()}
        onCancel={handleCancel}
      />
    );

    fireEvent.click(screen.getByRole("button", { name: /cancel/i }));
    expect(handleCancel).toHaveBeenCalled();
  });
});

describe("LineComments", () => {
  const mockComments: ReviewComment[] = [
    mockComment,
    {
      ...mockComment,
      id: "comment-2",
      content: "Second comment",
    },
  ];

  it("renders all comments", () => {
    render(
      <LineComments
        comments={mockComments}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
      />
    );

    expect(screen.getByText("This is a test comment")).toBeInTheDocument();
    expect(screen.getByText("Second comment")).toBeInTheDocument();
  });

  it("shows add comment button when enabled", () => {
    const handleAdd = vi.fn();
    render(
      <LineComments
        comments={mockComments}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        onAddComment={handleAdd}
        showAddButton={true}
      />
    );

    const addButton = screen.getByRole("button", { name: /add comment/i });
    expect(addButton).toBeInTheDocument();

    fireEvent.click(addButton);
    expect(handleAdd).toHaveBeenCalled();
  });

  it("hides add comment button when disabled", () => {
    render(
      <LineComments
        comments={mockComments}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        showAddButton={false}
      />
    );

    expect(screen.queryByRole("button", { name: /add comment/i })).not.toBeInTheDocument();
  });

  it("returns null when no comments and add button disabled", () => {
    const { container } = render(
      <LineComments
        comments={[]}
        onEdit={vi.fn()}
        onDelete={vi.fn()}
        showAddButton={false}
      />
    );

    expect(container.firstChild).toBeNull();
  });
});
