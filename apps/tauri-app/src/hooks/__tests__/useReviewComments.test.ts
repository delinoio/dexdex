import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useReviewComments } from "../useReviewComments";

describe("useReviewComments", () => {
  it("initializes with empty comments", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    expect(result.current.comments).toEqual([]);
    expect(result.current.commentCount).toBe(0);
  });

  it("adds a comment", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Test comment");
    });

    expect(result.current.comments).toHaveLength(1);
    expect(result.current.comments[0].filePath).toBe("src/test.ts");
    expect(result.current.comments[0].lineNumber).toBe(10);
    expect(result.current.comments[0].content).toBe("Test comment");
    expect(result.current.comments[0].author).toBe("You");
    expect(result.current.commentCount).toBe(1);
  });

  it("updates a comment", async () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Original comment");
    });

    const commentId = result.current.comments[0].id;
    const originalCreatedAt = result.current.comments[0].createdAt;

    // Wait to ensure different timestamp
    await new Promise((resolve) => setTimeout(resolve, 10));

    act(() => {
      result.current.updateComment(commentId, "Updated comment");
    });

    expect(result.current.comments[0].content).toBe("Updated comment");
    expect(result.current.comments[0].updatedAt).not.toBe(originalCreatedAt);
  });

  it("deletes a comment", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Test comment");
    });

    const commentId = result.current.comments[0].id;

    act(() => {
      result.current.deleteComment(commentId);
    });

    expect(result.current.comments).toHaveLength(0);
    expect(result.current.commentCount).toBe(0);
  });

  it("gets comments for a specific file", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Comment 1");
      result.current.addComment("src/test.ts", 20, "Comment 2");
      result.current.addComment("src/other.ts", 5, "Comment 3");
    });

    const fileComments = result.current.getCommentsForFile("src/test.ts");
    expect(fileComments).toHaveLength(2);
    expect(fileComments[0].content).toBe("Comment 1");
    expect(fileComments[1].content).toBe("Comment 2");
  });

  it("gets comments for a specific line", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Comment 1");
      result.current.addComment("src/test.ts", 10, "Comment 2");
      result.current.addComment("src/test.ts", 20, "Comment 3");
    });

    const lineComments = result.current.getCommentsForLine("src/test.ts", 10);
    expect(lineComments).toHaveLength(2);
    expect(lineComments[0].content).toBe("Comment 1");
    expect(lineComments[1].content).toBe("Comment 2");
  });

  it("checks if line has comments", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Test comment");
    });

    expect(result.current.hasCommentsForLine("src/test.ts", 10)).toBe(true);
    expect(result.current.hasCommentsForLine("src/test.ts", 20)).toBe(false);
    expect(result.current.hasCommentsForLine("src/other.ts", 10)).toBe(false);
  });

  it("handles multiple comments on the same line", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "First comment");
      result.current.addComment("src/test.ts", 10, "Second comment");
      result.current.addComment("src/test.ts", 10, "Third comment");
    });

    const lineComments = result.current.getCommentsForLine("src/test.ts", 10);
    expect(lineComments).toHaveLength(3);
    expect(result.current.hasCommentsForLine("src/test.ts", 10)).toBe(true);
  });

  it("clears all comments", () => {
    const { result } = renderHook(() => useReviewComments({ taskId: "task-1" }));

    act(() => {
      result.current.addComment("src/test.ts", 10, "Comment 1");
      result.current.addComment("src/other.ts", 20, "Comment 2");
      result.current.addComment("src/third.ts", 30, "Comment 3");
    });

    expect(result.current.comments).toHaveLength(3);
    expect(result.current.commentCount).toBe(3);

    act(() => {
      result.current.clearAll();
    });

    expect(result.current.comments).toHaveLength(0);
    expect(result.current.commentCount).toBe(0);
  });

  it("clears comments when taskId changes", () => {
    let taskId = "task-1";
    const { result, rerender } = renderHook(() =>
      useReviewComments({ taskId })
    );

    act(() => {
      result.current.addComment("src/test.ts", 10, "Comment 1");
      result.current.addComment("src/other.ts", 20, "Comment 2");
    });

    expect(result.current.comments).toHaveLength(2);

    // Simulate navigating to a different task
    taskId = "task-2";
    rerender();

    expect(result.current.comments).toHaveLength(0);
    expect(result.current.commentCount).toBe(0);
  });
});
