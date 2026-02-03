import { forwardRef, useState, type HTMLAttributes } from "react";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { DiffViewer, type DiffLine } from "./DiffViewer";
import { useReviewComments } from "@/hooks/useReviewComments";

export interface ChangedFile {
  path: string;
  status: "added" | "modified" | "deleted";
  additions: number;
  deletions: number;
  diff: DiffLine[];
}

export interface ReviewInterfaceProps extends HTMLAttributes<HTMLDivElement> {
  taskId: string;
  taskTitle?: string;
  branchName?: string;
  changedFiles: ChangedFile[];
  onApprove?: () => void;
  onRequestChanges?: (feedback: string) => void;
  onReject?: () => void;
  onCommit?: () => void;
  onCreatePR?: () => void;
  isApproving?: boolean;
  isRejecting?: boolean;
  isRequestingChanges?: boolean;
}

const ReviewInterface = forwardRef<HTMLDivElement, ReviewInterfaceProps>(
  (
    {
      taskId,
      taskTitle,
      branchName,
      changedFiles,
      onApprove,
      onRequestChanges,
      onReject,
      onCommit,
      onCreatePR,
      isApproving,
      isRejecting,
      isRequestingChanges,
      className,
      ...props
    },
    ref
  ) => {
    const [selectedFile, setSelectedFile] = useState<string | null>(
      changedFiles[0]?.path ?? null
    );
    const [viewedFiles, setViewedFiles] = useState<Set<string>>(new Set());

    const {
      comments,
      getCommentsForFile,
      getCommentCount,
      addComment,
      updateComment,
      deleteComment,
    } = useReviewComments(taskId);

    const toggleViewed = (filePath: string) => {
      setViewedFiles((prev) => {
        const newSet = new Set(prev);
        if (newSet.has(filePath)) {
          newSet.delete(filePath);
        } else {
          newSet.add(filePath);
        }
        return newSet;
      });
    };

    const viewedCount = viewedFiles.size;
    const totalFiles = changedFiles.length;

    const selectedFileData = changedFiles.find((f) => f.path === selectedFile);

    const getFileStatusIcon = (status: ChangedFile["status"]) => {
      switch (status) {
        case "added":
          return "A";
        case "modified":
          return "M";
        case "deleted":
          return "D";
      }
    };

    const getFileStatusColor = (status: ChangedFile["status"]) => {
      switch (status) {
        case "added":
          return "text-green-600";
        case "modified":
          return "text-yellow-600";
        case "deleted":
          return "text-red-600";
      }
    };

    return (
      <div
        ref={ref}
        className={cn("flex h-full flex-col", className)}
        {...props}
      >
        {/* Header */}
        <div className="border-b border-[hsl(var(--border))] px-4 py-3">
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-lg font-semibold">Code Review</h2>
              {taskTitle && (
                <p className="text-sm text-[hsl(var(--muted-foreground))]">
                  Task: {taskTitle}
                </p>
              )}
              {branchName && (
                <p className="font-mono text-xs text-[hsl(var(--muted-foreground))]">
                  {branchName}
                </p>
              )}
            </div>
            <div className="flex gap-2">
              {onApprove && (
                <Button onClick={onApprove} disabled={isApproving}>
                  {isApproving ? "Approving..." : "Approve"}
                </Button>
              )}
              {onRequestChanges && (
                <Button
                  variant="outline"
                  onClick={() => {
                    // Collect all comments as feedback
                    const feedback = comments
                      .map((c) => `${c.filePath}:${c.lineNumber}: ${c.content}`)
                      .join("\n");
                    if (feedback) {
                      onRequestChanges(feedback);
                    }
                  }}
                  disabled={isRequestingChanges || comments.length === 0}
                >
                  {isRequestingChanges ? "Requesting..." : "Request Changes"}
                </Button>
              )}
              {onReject && (
                <Button
                  variant="destructive"
                  onClick={onReject}
                  disabled={isRejecting}
                >
                  {isRejecting ? "Rejecting..." : "Reject"}
                </Button>
              )}
              {onCommit && (
                <Button variant="outline" onClick={onCommit}>
                  Commit
                </Button>
              )}
              {onCreatePR && (
                <Button variant="outline" onClick={onCreatePR}>
                  Create PR
                </Button>
              )}
            </div>
          </div>
        </div>

        {/* Main content */}
        <div className="flex flex-1 overflow-hidden">
          {/* File tree */}
          <div className="w-72 flex-shrink-0 overflow-y-auto border-r border-[hsl(var(--border))]">
            <div className="p-3">
              <div className="mb-2 flex items-center justify-between">
                <span className="text-sm font-medium">
                  Files Changed ({totalFiles})
                </span>
                <span className="text-xs text-[hsl(var(--muted-foreground))]">
                  {viewedCount}/{totalFiles} viewed
                </span>
              </div>
              <ul className="space-y-1">
                {changedFiles.map((file) => {
                  const isSelected = selectedFile === file.path;
                  const isViewed = viewedFiles.has(file.path);
                  const commentCount = getCommentCount(file.path);

                  return (
                    <li key={file.path}>
                      <button
                        className={cn(
                          "flex w-full items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm transition-colors",
                          isSelected
                            ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
                            : "hover:bg-[hsl(var(--muted))]"
                        )}
                        onClick={() => setSelectedFile(file.path)}
                      >
                        <span
                          className={cn(
                            "font-mono text-xs font-bold",
                            getFileStatusColor(file.status)
                          )}
                        >
                          {getFileStatusIcon(file.status)}
                        </span>
                        <span className="flex-1 truncate font-mono text-xs">
                          {file.path}
                        </span>
                        <div className="flex items-center gap-1">
                          {commentCount > 0 && (
                            <span className="flex h-5 w-5 items-center justify-center rounded-full bg-[hsl(var(--primary))] text-xs text-[hsl(var(--primary-foreground))]">
                              {commentCount}
                            </span>
                          )}
                          {isViewed && (
                            <span className="text-green-600">
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
                              >
                                <path d="M20 6 9 17l-5-5" />
                              </svg>
                            </span>
                          )}
                        </div>
                      </button>
                    </li>
                  );
                })}
              </ul>
            </div>
          </div>

          {/* Diff viewer */}
          <div className="flex-1 overflow-auto p-4">
            {selectedFileData ? (
              <DiffViewer
                filePath={selectedFileData.path}
                lines={selectedFileData.diff}
                comments={getCommentsForFile(selectedFileData.path)}
                onAddComment={(lineNumber, content) =>
                  addComment(selectedFileData.path, lineNumber, content)
                }
                onEditComment={updateComment}
                onDeleteComment={deleteComment}
                onMarkAsViewed={() => toggleViewed(selectedFileData.path)}
                isViewed={viewedFiles.has(selectedFileData.path)}
              />
            ) : (
              <div className="flex h-full items-center justify-center text-[hsl(var(--muted-foreground))]">
                Select a file to view
              </div>
            )}
          </div>
        </div>

        {/* Comments summary */}
        {comments.length > 0 && (
          <div className="border-t border-[hsl(var(--border))] px-4 py-3">
            <div className="flex items-center justify-between">
              <span className="text-sm">
                {comments.length} comment{comments.length !== 1 ? "s" : ""} on
                this review
              </span>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => {
                  // Could show a summary modal or expand comments panel
                }}
              >
                View all comments
              </Button>
            </div>
          </div>
        )}
      </div>
    );
  }
);
ReviewInterface.displayName = "ReviewInterface";

export { ReviewInterface };
