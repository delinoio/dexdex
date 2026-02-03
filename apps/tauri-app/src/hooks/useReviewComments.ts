import { useState, useCallback } from "react";

export interface ReviewComment {
  id: string;
  taskId: string;
  filePath: string;
  lineNumber: number;
  content: string;
  author: string;
  createdAt: string;
  updatedAt: string;
}

interface ReviewCommentsState {
  comments: ReviewComment[];
  isLoading: boolean;
  error: string | null;
}

// Generate unique ID for comments
const generateId = () => {
  return `comment_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
};

// Get current timestamp in ISO format
const getCurrentTimestamp = () => {
  return new Date().toISOString();
};

// Local storage key for persisting comments
const getStorageKey = (taskId: string) => `review_comments_${taskId}`;

// Load comments from local storage
const loadComments = (taskId: string): ReviewComment[] => {
  try {
    const stored = localStorage.getItem(getStorageKey(taskId));
    if (stored) {
      return JSON.parse(stored);
    }
  } catch (error) {
    console.error("Failed to load comments from storage:", error);
  }
  return [];
};

// Save comments to local storage
const saveComments = (taskId: string, comments: ReviewComment[]) => {
  try {
    localStorage.setItem(getStorageKey(taskId), JSON.stringify(comments));
  } catch (error) {
    console.error("Failed to save comments to storage:", error);
  }
};

export function useReviewComments(taskId: string) {
  const [state, setState] = useState<ReviewCommentsState>(() => ({
    comments: loadComments(taskId),
    isLoading: false,
    error: null,
  }));

  // Get comments for a specific file
  const getCommentsForFile = useCallback(
    (filePath: string) => {
      return state.comments.filter((c) => c.filePath === filePath);
    },
    [state.comments]
  );

  // Get comments for a specific line
  const getCommentsForLine = useCallback(
    (filePath: string, lineNumber: number) => {
      return state.comments.filter(
        (c) => c.filePath === filePath && c.lineNumber === lineNumber
      );
    },
    [state.comments]
  );

  // Get all file paths that have comments
  const getFilesWithComments = useCallback(() => {
    const files = new Set(state.comments.map((c) => c.filePath));
    return Array.from(files);
  }, [state.comments]);

  // Get comment count for a file
  const getCommentCount = useCallback(
    (filePath: string) => {
      return state.comments.filter((c) => c.filePath === filePath).length;
    },
    [state.comments]
  );

  // Add a new comment
  const addComment = useCallback(
    (filePath: string, lineNumber: number, content: string, author: string = "You") => {
      const timestamp = getCurrentTimestamp();
      const newComment: ReviewComment = {
        id: generateId(),
        taskId,
        filePath,
        lineNumber,
        content,
        author,
        createdAt: timestamp,
        updatedAt: timestamp,
      };

      setState((prev) => {
        const newComments = [...prev.comments, newComment];
        saveComments(taskId, newComments);
        return {
          ...prev,
          comments: newComments,
        };
      });

      return newComment;
    },
    [taskId]
  );

  // Update an existing comment
  const updateComment = useCallback(
    (commentId: string, content: string) => {
      setState((prev) => {
        const newComments = prev.comments.map((c) =>
          c.id === commentId
            ? { ...c, content, updatedAt: getCurrentTimestamp() }
            : c
        );
        saveComments(taskId, newComments);
        return {
          ...prev,
          comments: newComments,
        };
      });
    },
    [taskId]
  );

  // Delete a comment
  const deleteComment = useCallback(
    (commentId: string) => {
      setState((prev) => {
        const newComments = prev.comments.filter((c) => c.id !== commentId);
        saveComments(taskId, newComments);
        return {
          ...prev,
          comments: newComments,
        };
      });
    },
    [taskId]
  );

  // Clear all comments for a task
  const clearComments = useCallback(() => {
    setState((prev) => {
      saveComments(taskId, []);
      return {
        ...prev,
        comments: [],
      };
    });
  }, [taskId]);

  return {
    comments: state.comments,
    isLoading: state.isLoading,
    error: state.error,
    getCommentsForFile,
    getCommentsForLine,
    getFilesWithComments,
    getCommentCount,
    addComment,
    updateComment,
    deleteComment,
    clearComments,
  };
}
