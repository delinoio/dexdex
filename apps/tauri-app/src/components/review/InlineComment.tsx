import { forwardRef, useState, type HTMLAttributes } from "react";
import { Button } from "@/components/ui/Button";
import { Textarea } from "@/components/ui/Textarea";
import { cn } from "@/lib/utils";
import type { ReviewComment } from "@/hooks/useReviewComments";

export interface InlineCommentProps extends HTMLAttributes<HTMLDivElement> {
  comment: ReviewComment;
  onEdit?: (commentId: string, newContent: string) => void;
  onDelete?: (commentId: string) => void;
  isEditable?: boolean;
}

const InlineComment = forwardRef<HTMLDivElement, InlineCommentProps>(
  ({ comment, onEdit, onDelete, isEditable = true, className, ...props }, ref) => {
    const [isEditing, setIsEditing] = useState(false);
    const [editContent, setEditContent] = useState(comment.content);

    const handleSaveEdit = () => {
      if (editContent.trim() && onEdit) {
        onEdit(comment.id, editContent.trim());
        setIsEditing(false);
      }
    };

    const handleCancelEdit = () => {
      setEditContent(comment.content);
      setIsEditing(false);
    };

    const formatTimestamp = (timestamp: string) => {
      const date = new Date(timestamp);
      return date.toLocaleString();
    };

    return (
      <div
        ref={ref}
        className={cn(
          "rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-3",
          className
        )}
        {...props}
      >
        <div className="mb-2 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <span className="font-medium text-sm">{comment.author}</span>
            <span className="text-xs text-[hsl(var(--muted-foreground))]">
              {formatTimestamp(comment.createdAt)}
            </span>
            {comment.updatedAt !== comment.createdAt && (
              <span className="text-xs text-[hsl(var(--muted-foreground))]">
                (edited)
              </span>
            )}
          </div>
          {isEditable && !isEditing && (
            <div className="flex gap-1">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setIsEditing(true)}
                className="h-6 px-2 text-xs"
              >
                Edit
              </Button>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => onDelete?.(comment.id)}
                className="h-6 px-2 text-xs text-[hsl(var(--destructive))]"
              >
                Delete
              </Button>
            </div>
          )}
        </div>

        {isEditing ? (
          <div className="space-y-2">
            <Textarea
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              rows={3}
              className="text-sm"
            />
            <div className="flex gap-2">
              <Button size="sm" onClick={handleSaveEdit}>
                Save
              </Button>
              <Button variant="outline" size="sm" onClick={handleCancelEdit}>
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <p className="whitespace-pre-wrap text-sm">{comment.content}</p>
        )}
      </div>
    );
  }
);
InlineComment.displayName = "InlineComment";

export interface CommentFormProps extends Omit<HTMLAttributes<HTMLDivElement>, "onSubmit"> {
  onSubmit: (content: string) => void;
  onCancel?: () => void;
  placeholder?: string;
}

const CommentForm = forwardRef<HTMLDivElement, CommentFormProps>(
  ({ onSubmit, onCancel, placeholder = "Add a comment...", className, ...props }, ref) => {
    const [content, setContent] = useState("");

    const handleSubmit = () => {
      if (content.trim()) {
        onSubmit(content.trim());
        setContent("");
      }
    };

    return (
      <div
        ref={ref}
        className={cn(
          "rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-3",
          className
        )}
        {...props}
      >
        <Textarea
          value={content}
          onChange={(e) => setContent(e.target.value)}
          placeholder={placeholder}
          rows={3}
          className="mb-2 text-sm"
        />
        <div className="flex gap-2">
          <Button size="sm" onClick={handleSubmit} disabled={!content.trim()}>
            Add Comment
          </Button>
          {onCancel && (
            <Button variant="outline" size="sm" onClick={onCancel}>
              Cancel
            </Button>
          )}
        </div>
      </div>
    );
  }
);
CommentForm.displayName = "CommentForm";

export { InlineComment, CommentForm };
