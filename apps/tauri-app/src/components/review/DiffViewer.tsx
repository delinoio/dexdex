// Diff viewer component with inline comment support
// Displays file changes with line-by-line commenting capability

import { useState, useCallback, useMemo } from "react";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { InlineComment, CommentInputForm } from "./InlineComment";
import type { ReviewComment } from "@/hooks/useReviewComments";

export enum DiffLineType {
  Context = "context",
  Addition = "addition",
  Deletion = "deletion",
  Header = "header",
}

export interface DiffLine {
  type: DiffLineType;
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffFile {
  filePath: string;
  oldPath?: string;
  status: "added" | "modified" | "deleted" | "renamed";
  lines: DiffLine[];
}

export interface DiffViewerProps {
  file: DiffFile;
  comments: ReviewComment[];
  onAddComment: (filePath: string, lineNumber: number, content: string) => void;
  onEditComment: (commentId: string, content: string) => void;
  onDeleteComment: (commentId: string) => void;
  onMarkAsViewed?: () => void;
  isViewed?: boolean;
  className?: string;
}

export function DiffViewer({
  file,
  comments,
  onAddComment,
  onEditComment,
  onDeleteComment,
  onMarkAsViewed,
  isViewed = false,
  className,
}: DiffViewerProps) {
  const [activeCommentLine, setActiveCommentLine] = useState<number | null>(
    null
  );
  const [expandedCommentLines, setExpandedCommentLines] = useState<Set<number>>(
    new Set()
  );

  // Get comments for a specific line
  const getCommentsForLine = useCallback(
    (lineNumber: number): ReviewComment[] => {
      return comments.filter(
        (c) => c.filePath === file.filePath && c.lineNumber === lineNumber
      );
    },
    [comments, file.filePath]
  );

  // Get count of comments for display in file list
  const fileCommentCount = useMemo(() => {
    return comments.filter((c) => c.filePath === file.filePath).length;
  }, [comments, file.filePath]);

  // Handle line click to add comment
  const handleLineClick = (lineNumber: number) => {
    if (activeCommentLine === lineNumber) {
      setActiveCommentLine(null);
    } else {
      setActiveCommentLine(lineNumber);
      setExpandedCommentLines((prev) => new Set([...prev, lineNumber]));
    }
  };

  // Toggle comment visibility for a line
  const toggleCommentLine = (lineNumber: number) => {
    setExpandedCommentLines((prev) => {
      const next = new Set(prev);
      if (next.has(lineNumber)) {
        next.delete(lineNumber);
      } else {
        next.add(lineNumber);
      }
      return next;
    });
  };

  // Handle adding a comment
  const handleAddComment = (content: string) => {
    if (activeCommentLine !== null) {
      onAddComment(file.filePath, activeCommentLine, content);
      setActiveCommentLine(null);
    }
  };

  // Cancel adding a comment
  const handleCancelComment = () => {
    setActiveCommentLine(null);
  };

  // Get status badge color
  const getStatusColor = (status: DiffFile["status"]) => {
    switch (status) {
      case "added":
        return "text-green-500";
      case "deleted":
        return "text-red-500";
      case "modified":
        return "text-yellow-500";
      case "renamed":
        return "text-blue-500";
      default:
        return "text-[hsl(var(--muted-foreground))]";
    }
  };

  // Get line background color based on type
  const getLineBackground = (type: DiffLineType) => {
    switch (type) {
      case DiffLineType.Addition:
        return "bg-green-500/10";
      case DiffLineType.Deletion:
        return "bg-red-500/10";
      case DiffLineType.Header:
        return "bg-[hsl(var(--muted))]/50";
      default:
        return "";
    }
  };

  // Get line text color based on type
  const getLineColor = (type: DiffLineType) => {
    switch (type) {
      case DiffLineType.Addition:
        return "text-green-700 dark:text-green-400";
      case DiffLineType.Deletion:
        return "text-red-700 dark:text-red-400";
      case DiffLineType.Header:
        return "text-[hsl(var(--muted-foreground))]";
      default:
        return "text-[hsl(var(--foreground))]";
    }
  };

  // Get line prefix character
  const getLinePrefix = (type: DiffLineType) => {
    switch (type) {
      case DiffLineType.Addition:
        return "+";
      case DiffLineType.Deletion:
        return "-";
      default:
        return " ";
    }
  };

  return (
    <div
      className={cn(
        "rounded-lg border border-[hsl(var(--border))] overflow-hidden",
        className
      )}
      data-testid="diff-viewer"
    >
      {/* File header */}
      <div className="flex items-center justify-between border-b border-[hsl(var(--border))] bg-[hsl(var(--muted))]/50 px-4 py-2">
        <div className="flex items-center gap-2">
          <span className={cn("text-xs font-medium", getStatusColor(file.status))}>
            {file.status.toUpperCase()}
          </span>
          <span className="font-mono text-sm">{file.filePath}</span>
          {file.oldPath && file.oldPath !== file.filePath && (
            <span className="text-xs text-[hsl(var(--muted-foreground))]">
              (was: {file.oldPath})
            </span>
          )}
          {fileCommentCount > 0 && (
            <span className="rounded-full bg-[hsl(var(--primary))] px-2 py-0.5 text-xs text-[hsl(var(--primary-foreground))]">
              {fileCommentCount} comment{fileCommentCount !== 1 ? "s" : ""}
            </span>
          )}
        </div>
        <div className="flex items-center gap-2">
          {onMarkAsViewed && (
            <Button
              variant={isViewed ? "secondary" : "outline"}
              size="sm"
              onClick={onMarkAsViewed}
            >
              {isViewed ? "Viewed" : "Mark as viewed"}
            </Button>
          )}
        </div>
      </div>

      {/* Diff content */}
      <div className="overflow-x-auto">
        <table className="w-full font-mono text-sm">
          <tbody>
            {file.lines.map((line, index) => {
              const lineNumber = line.newLineNumber ?? line.oldLineNumber ?? 0;
              const lineComments = getCommentsForLine(lineNumber);
              const hasComments = lineComments.length > 0;
              const isClickable = line.type !== DiffLineType.Header;

              return (
                <tr key={`line-${index}`} className="group">
                  {/* Line numbers */}
                  <td
                    className={cn(
                      "w-12 select-none border-r border-[hsl(var(--border))] px-2 text-right text-xs text-[hsl(var(--muted-foreground))]",
                      getLineBackground(line.type),
                      isClickable && "cursor-pointer hover:bg-[hsl(var(--accent))]",
                      hasComments && "bg-yellow-500/20"
                    )}
                    onClick={() => isClickable && handleLineClick(lineNumber)}
                    data-testid={`diff-line-${index}`}
                  >
                    {line.oldLineNumber ?? ""}
                  </td>
                  <td
                    className={cn(
                      "w-12 select-none border-r border-[hsl(var(--border))] px-2 text-right text-xs text-[hsl(var(--muted-foreground))]",
                      getLineBackground(line.type),
                      isClickable && "cursor-pointer hover:bg-[hsl(var(--accent))]",
                      hasComments && "bg-yellow-500/20"
                    )}
                    onClick={() => isClickable && handleLineClick(lineNumber)}
                  >
                    {line.newLineNumber ?? ""}
                  </td>

                  {/* Comment indicator */}
                  <td
                    className={cn(
                      "w-6 select-none border-r border-[hsl(var(--border))] text-center",
                      getLineBackground(line.type),
                      isClickable && "cursor-pointer"
                    )}
                    onClick={() => isClickable && hasComments && toggleCommentLine(lineNumber)}
                  >
                    {hasComments && (
                      <button
                        className="text-[hsl(var(--primary))] hover:text-[hsl(var(--primary))]/80"
                        title={`${lineComments.length} comment${lineComments.length !== 1 ? "s" : ""}`}
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="currentColor"
                          className="mx-auto"
                        >
                          <path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z" />
                        </svg>
                      </button>
                    )}
                    {!hasComments && isClickable && (
                      <button
                        className="invisible text-[hsl(var(--muted-foreground))] group-hover:visible hover:text-[hsl(var(--primary))]"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleLineClick(lineNumber);
                        }}
                        title="Add comment"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="2"
                          strokeLinecap="round"
                          strokeLinejoin="round"
                          className="mx-auto"
                        >
                          <path d="M12 5v14M5 12h14" />
                        </svg>
                      </button>
                    )}
                  </td>

                  {/* Line content */}
                  <td
                    className={cn(
                      "whitespace-pre px-4 py-0.5",
                      getLineBackground(line.type),
                      getLineColor(line.type)
                    )}
                  >
                    <span className="select-none opacity-50">
                      {getLinePrefix(line.type)}
                    </span>
                    {line.content}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>

        {/* Inline comments section - rendered separately to avoid table layout issues */}
        {file.lines.map((line, index) => {
          const lineNumber = line.newLineNumber ?? line.oldLineNumber ?? 0;
          const lineComments = getCommentsForLine(lineNumber);
          const isCommentsExpanded = expandedCommentLines.has(lineNumber);
          const isAddingComment = activeCommentLine === lineNumber;
          const showComments = (lineComments.length > 0 && isCommentsExpanded) || isAddingComment;

          // Only show comment section for the first occurrence of each line number
          // to avoid duplicates when both old and new line numbers are the same
          const isFirstOccurrence = file.lines.findIndex(
            (l) => (l.newLineNumber ?? l.oldLineNumber ?? 0) === lineNumber
          ) === index;

          if (!showComments || !isFirstOccurrence || lineNumber === 0) return null;

          return (
            <div
              key={`comments-${index}`}
              className="border-t border-[hsl(var(--border))] bg-[hsl(var(--muted))]/30 px-4 py-2"
              data-testid={`line-comments-${lineNumber}`}
            >
              <div className="flex items-center gap-2 mb-2 text-xs text-[hsl(var(--muted-foreground))]">
                <span>Line {lineNumber}</span>
              </div>
              {lineComments.map((comment) => (
                <InlineComment
                  key={comment.id}
                  comment={comment}
                  onEdit={onEditComment}
                  onDelete={onDeleteComment}
                  className="mb-2"
                />
              ))}
              {isAddingComment && (
                <CommentInputForm
                  onSubmit={handleAddComment}
                  onCancel={handleCancelComment}
                  placeholder={`Add a comment on line ${lineNumber}...`}
                />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

// File list component for the sidebar
export interface DiffFileListProps {
  files: DiffFile[];
  selectedFilePath?: string;
  onSelectFile: (filePath: string) => void;
  viewedFiles: Set<string>;
  commentCounts: Record<string, number>;
  className?: string;
}

export function DiffFileList({
  files,
  selectedFilePath,
  onSelectFile,
  viewedFiles,
  commentCounts,
  className,
}: DiffFileListProps) {
  // Get status icon
  const getStatusIcon = (status: DiffFile["status"]) => {
    switch (status) {
      case "added":
        return (
          <span className="text-green-500" title="Added">
            +
          </span>
        );
      case "deleted":
        return (
          <span className="text-red-500" title="Deleted">
            -
          </span>
        );
      case "modified":
        return (
          <span className="text-yellow-500" title="Modified">
            ~
          </span>
        );
      case "renamed":
        return (
          <span className="text-blue-500" title="Renamed">
            R
          </span>
        );
      default:
        return null;
    }
  };

  const viewedCount = files.filter((f) => viewedFiles.has(f.filePath)).length;

  return (
    <div className={cn("space-y-1", className)} data-testid="diff-file-list">
      <div className="mb-2 text-xs text-[hsl(var(--muted-foreground))]">
        {viewedCount}/{files.length} viewed
      </div>
      {files.map((file) => {
        const isViewed = viewedFiles.has(file.filePath);
        const commentCount = commentCounts[file.filePath] ?? 0;
        const isSelected = selectedFilePath === file.filePath;

        return (
          <button
            key={file.filePath}
            className={cn(
              "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors",
              isSelected
                ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
                : "hover:bg-[hsl(var(--accent))]/50",
              isViewed && "opacity-60"
            )}
            onClick={() => onSelectFile(file.filePath)}
          >
            <span className="w-4 text-center font-mono text-xs">
              {getStatusIcon(file.status)}
            </span>
            <span className="flex-1 truncate font-mono text-xs">
              {file.filePath.split("/").pop()}
            </span>
            {isViewed && (
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="14"
                height="14"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
                className="text-green-500"
              >
                <polyline points="20 6 9 17 4 12" />
              </svg>
            )}
            {commentCount > 0 && (
              <span className="rounded-full bg-[hsl(var(--primary))] px-1.5 py-0.5 text-xs text-[hsl(var(--primary-foreground))]">
                {commentCount}
              </span>
            )}
          </button>
        );
      })}
    </div>
  );
}
