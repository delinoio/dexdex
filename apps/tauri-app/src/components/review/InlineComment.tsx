// Inline comment component for code review
// Displays a single comment with edit/delete actions

import { useState, type FormEvent } from "react";
import { Button } from "@/components/ui/Button";
import { Textarea } from "@/components/ui/Textarea";
import { FormattedDateTime } from "@/components/ui/FormattedDateTime";
import { cn } from "@/lib/utils";
import type { ReviewComment } from "@/hooks/useReviewComments";

export interface InlineCommentProps {
  comment: ReviewComment;
  onEdit: (commentId: string, content: string) => void;
  onDelete: (commentId: string) => void;
  className?: string;
}

export function InlineComment({
  comment,
  onEdit,
  onDelete,
  className,
}: InlineCommentProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(comment.content);

  const handleSubmitEdit = (e: FormEvent) => {
    e.preventDefault();
    if (editContent.trim()) {
      onEdit(comment.id, editContent.trim());
      setIsEditing(false);
    }
  };

  const handleCancelEdit = () => {
    setEditContent(comment.content);
    setIsEditing(false);
  };

  const handleDelete = () => {
    onDelete(comment.id);
  };

  return (
    <div
      className={cn(
        "rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-3",
        className
      )}
      data-testid="inline-comment"
    >
      <div className="mb-2 flex items-center justify-between">
        <div className="flex items-center gap-2 text-sm">
          <span className="font-medium text-[hsl(var(--foreground))]">
            {comment.author}
          </span>
          <span className="text-[hsl(var(--muted-foreground))]">
            <FormattedDateTime date={comment.createdAt} />
          </span>
          {comment.updatedAt !== comment.createdAt && (
            <span className="text-xs text-[hsl(var(--muted-foreground))]">
              (edited)
            </span>
          )}
        </div>
        {!isEditing && (
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
              onClick={handleDelete}
              className="h-6 px-2 text-xs text-[hsl(var(--destructive))] hover:text-[hsl(var(--destructive))]"
            >
              Delete
            </Button>
          </div>
        )}
      </div>

      {isEditing ? (
        <form onSubmit={handleSubmitEdit} className="space-y-2">
          <Textarea
            value={editContent}
            onChange={(e) => setEditContent(e.target.value)}
            placeholder="Write a comment..."
            rows={3}
            autoFocus
            data-testid="inline-comment-edit-textarea"
          />
          <div className="flex gap-2">
            <Button type="submit" size="sm" disabled={!editContent.trim()}>
              Save
            </Button>
            <Button
              type="button"
              variant="outline"
              size="sm"
              onClick={handleCancelEdit}
            >
              Cancel
            </Button>
          </div>
        </form>
      ) : (
        <p className="whitespace-pre-wrap text-sm text-[hsl(var(--foreground))]">
          {comment.content}
        </p>
      )}
    </div>
  );
}

// Comment input form for adding new comments
export interface CommentInputFormProps {
  onSubmit: (content: string) => void;
  onCancel: () => void;
  placeholder?: string;
  className?: string;
}

export function CommentInputForm({
  onSubmit,
  onCancel,
  placeholder = "Write a comment...",
  className,
}: CommentInputFormProps) {
  const [content, setContent] = useState("");

  const handleSubmit = (e: FormEvent) => {
    e.preventDefault();
    if (content.trim()) {
      onSubmit(content.trim());
      setContent("");
    }
  };

  return (
    <form
      onSubmit={handleSubmit}
      className={cn(
        "rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--card))] p-3",
        className
      )}
      data-testid="comment-input-form"
    >
      <Textarea
        value={content}
        onChange={(e) => setContent(e.target.value)}
        placeholder={placeholder}
        rows={3}
        autoFocus
        data-testid="comment-input-textarea"
      />
      <div className="mt-2 flex gap-2">
        <Button type="submit" size="sm" disabled={!content.trim()}>
          Add Comment
        </Button>
        <Button type="button" variant="outline" size="sm" onClick={onCancel}>
          Cancel
        </Button>
      </div>
    </form>
  );
}

// Line comments container - displays all comments for a specific line
export interface LineCommentsProps {
  comments: ReviewComment[];
  onEdit: (commentId: string, content: string) => void;
  onDelete: (commentId: string) => void;
  onAddComment?: () => void;
  showAddButton?: boolean;
  className?: string;
}

export function LineComments({
  comments,
  onEdit,
  onDelete,
  onAddComment,
  showAddButton = true,
  className,
}: LineCommentsProps) {
  if (comments.length === 0 && !showAddButton) {
    return null;
  }

  return (
    <div
      className={cn("space-y-2 py-2", className)}
      data-testid="line-comments"
    >
      {comments.map((comment) => (
        <InlineComment
          key={comment.id}
          comment={comment}
          onEdit={onEdit}
          onDelete={onDelete}
        />
      ))}
      {showAddButton && onAddComment && (
        <Button
          variant="ghost"
          size="sm"
          onClick={onAddComment}
          className="text-xs text-[hsl(var(--muted-foreground))] hover:text-[hsl(var(--foreground))]"
        >
          + Add comment
        </Button>
      )}
    </div>
  );
}
