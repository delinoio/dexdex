import { forwardRef, useState, type HTMLAttributes } from "react";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";
import { InlineComment, CommentForm } from "./InlineComment";
import type { ReviewComment } from "@/hooks/useReviewComments";

export interface DiffLine {
  lineNumber: number;
  type: "added" | "removed" | "unchanged" | "header";
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface DiffViewerProps extends HTMLAttributes<HTMLDivElement> {
  filePath: string;
  lines: DiffLine[];
  comments: ReviewComment[];
  onAddComment: (lineNumber: number, content: string) => void;
  onEditComment: (commentId: string, content: string) => void;
  onDeleteComment: (commentId: string) => void;
  onMarkAsViewed?: () => void;
  onOpenInEditor?: () => void;
  isViewed?: boolean;
}

const DiffViewer = forwardRef<HTMLDivElement, DiffViewerProps>(
  (
    {
      filePath,
      lines,
      comments,
      onAddComment,
      onEditComment,
      onDeleteComment,
      onMarkAsViewed,
      onOpenInEditor,
      isViewed = false,
      className,
      ...props
    },
    ref
  ) => {
    const [activeCommentLine, setActiveCommentLine] = useState<number | null>(null);

    const getLineComments = (lineNumber: number) => {
      return comments.filter((c) => c.lineNumber === lineNumber);
    };

    const handleLineClick = (lineNumber: number) => {
      if (activeCommentLine === lineNumber) {
        setActiveCommentLine(null);
      } else {
        setActiveCommentLine(lineNumber);
      }
    };

    const handleAddComment = (content: string) => {
      if (activeCommentLine !== null) {
        onAddComment(activeCommentLine, content);
        setActiveCommentLine(null);
      }
    };

    const getLineClassName = (type: DiffLine["type"]) => {
      switch (type) {
        case "added":
          return "bg-green-500/10 hover:bg-green-500/20";
        case "removed":
          return "bg-red-500/10 hover:bg-red-500/20";
        case "header":
          return "bg-[hsl(var(--muted))] text-[hsl(var(--muted-foreground))]";
        default:
          return "hover:bg-[hsl(var(--muted))]/50";
      }
    };

    const getLinePrefix = (type: DiffLine["type"]) => {
      switch (type) {
        case "added":
          return "+";
        case "removed":
          return "-";
        default:
          return " ";
      }
    };

    return (
      <div
        ref={ref}
        className={cn(
          "rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--card))]",
          className
        )}
        {...props}
      >
        {/* File header */}
        <div className="flex items-center justify-between border-b border-[hsl(var(--border))] px-4 py-2">
          <div className="flex items-center gap-2">
            <span className="font-mono text-sm font-medium">{filePath}</span>
            {isViewed && (
              <span className="text-xs text-[hsl(var(--muted-foreground))]">
                (viewed)
              </span>
            )}
          </div>
          <div className="flex gap-2">
            {onMarkAsViewed && (
              <Button variant="outline" size="sm" onClick={onMarkAsViewed}>
                {isViewed ? "Mark as unviewed" : "Mark as viewed"}
              </Button>
            )}
            {onOpenInEditor && (
              <Button variant="outline" size="sm" onClick={onOpenInEditor}>
                Open in Editor
              </Button>
            )}
          </div>
        </div>

        {/* Diff content */}
        <div className="overflow-x-auto">
          <table className="w-full font-mono text-sm">
            <tbody>
              {lines.map((line, index) => {
                const lineComments = getLineComments(line.lineNumber);
                const hasComments = lineComments.length > 0;
                const isCommentFormActive = activeCommentLine === line.lineNumber;

                return (
                  <>
                    <tr
                      key={`line-${index}`}
                      className={cn(
                        "group cursor-pointer",
                        getLineClassName(line.type),
                        hasComments && "border-l-2 border-l-[hsl(var(--primary))]"
                      )}
                      onClick={() => handleLineClick(line.lineNumber)}
                    >
                      {/* Line numbers */}
                      <td className="w-12 select-none border-r border-[hsl(var(--border))] px-2 py-0.5 text-right text-[hsl(var(--muted-foreground))]">
                        {line.type !== "added" && line.oldLineNumber}
                      </td>
                      <td className="w-12 select-none border-r border-[hsl(var(--border))] px-2 py-0.5 text-right text-[hsl(var(--muted-foreground))]">
                        {line.type !== "removed" && line.newLineNumber}
                      </td>

                      {/* Comment indicator */}
                      <td className="w-6 select-none text-center">
                        {hasComments ? (
                          <span className="inline-flex h-5 w-5 items-center justify-center rounded-full bg-[hsl(var(--primary))] text-xs text-[hsl(var(--primary-foreground))]">
                            {lineComments.length}
                          </span>
                        ) : (
                          <span className="invisible inline-flex h-5 w-5 items-center justify-center rounded-full bg-[hsl(var(--primary))] text-xs text-[hsl(var(--primary-foreground))] group-hover:visible">
                            +
                          </span>
                        )}
                      </td>

                      {/* Line content */}
                      <td className="whitespace-pre px-2 py-0.5">
                        <span
                          className={cn(
                            line.type === "added" && "text-green-600",
                            line.type === "removed" && "text-red-600"
                          )}
                        >
                          {getLinePrefix(line.type)}
                          {line.content}
                        </span>
                      </td>
                    </tr>

                    {/* Inline comments */}
                    {(hasComments || isCommentFormActive) && (
                      <tr key={`comments-${index}`}>
                        <td colSpan={4} className="bg-[hsl(var(--muted))]/30 p-3">
                          <div className="space-y-2 pl-6">
                            {lineComments.map((comment) => (
                              <InlineComment
                                key={comment.id}
                                comment={comment}
                                onEdit={onEditComment}
                                onDelete={onDeleteComment}
                              />
                            ))}
                            {isCommentFormActive && (
                              <CommentForm
                                onSubmit={handleAddComment}
                                onCancel={() => setActiveCommentLine(null)}
                                placeholder={`Add comment on line ${line.lineNumber}...`}
                              />
                            )}
                          </div>
                        </td>
                      </tr>
                    )}
                  </>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>
    );
  }
);
DiffViewer.displayName = "DiffViewer";

export { DiffViewer };
