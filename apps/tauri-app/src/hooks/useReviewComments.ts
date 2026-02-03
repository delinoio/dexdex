// Hook for managing inline review comments
// Uses local state management (no backend API required)

import { useState, useCallback, useMemo } from "react";

export interface ReviewComment {
  id: string;
  filePath: string;
  lineNumber: number;
  content: string;
  author: string;
  createdAt: string;
  updatedAt: string;
}

export interface UseReviewCommentsOptions {
  taskId: string;
}

export interface UseReviewCommentsReturn {
  comments: ReviewComment[];
  addComment: (filePath: string, lineNumber: number, content: string) => void;
  updateComment: (commentId: string, content: string) => void;
  deleteComment: (commentId: string) => void;
  getCommentsForFile: (filePath: string) => ReviewComment[];
  getCommentsForLine: (filePath: string, lineNumber: number) => ReviewComment[];
  hasCommentsForLine: (filePath: string, lineNumber: number) => boolean;
  commentCount: number;
}

function generateId(): string {
  return `comment-${Date.now()}-${Math.random().toString(36).substring(2, 9)}`;
}

function getCurrentUser(): string {
  // In a real implementation, this would come from auth context
  return "You";
}

export function useReviewComments({
  taskId,
}: UseReviewCommentsOptions): UseReviewCommentsReturn {
  const [comments, setComments] = useState<ReviewComment[]>([]);

  const addComment = useCallback(
    (filePath: string, lineNumber: number, content: string) => {
      const now = new Date().toISOString();
      const newComment: ReviewComment = {
        id: generateId(),
        filePath,
        lineNumber,
        content,
        author: getCurrentUser(),
        createdAt: now,
        updatedAt: now,
      };
      setComments((prev) => [...prev, newComment]);
    },
    [taskId]
  );

  const updateComment = useCallback((commentId: string, content: string) => {
    setComments((prev) =>
      prev.map((comment) =>
        comment.id === commentId
          ? { ...comment, content, updatedAt: new Date().toISOString() }
          : comment
      )
    );
  }, []);

  const deleteComment = useCallback((commentId: string) => {
    setComments((prev) => prev.filter((comment) => comment.id !== commentId));
  }, []);

  const getCommentsForFile = useCallback(
    (filePath: string): ReviewComment[] => {
      return comments.filter((comment) => comment.filePath === filePath);
    },
    [comments]
  );

  const getCommentsForLine = useCallback(
    (filePath: string, lineNumber: number): ReviewComment[] => {
      return comments.filter(
        (comment) =>
          comment.filePath === filePath && comment.lineNumber === lineNumber
      );
    },
    [comments]
  );

  const hasCommentsForLine = useCallback(
    (filePath: string, lineNumber: number): boolean => {
      return comments.some(
        (comment) =>
          comment.filePath === filePath && comment.lineNumber === lineNumber
      );
    },
    [comments]
  );

  const commentCount = useMemo(() => comments.length, [comments]);

  return {
    comments,
    addComment,
    updateComment,
    deleteComment,
    getCommentsForFile,
    getCommentsForLine,
    hasCommentsForLine,
    commentCount,
  };
}
