import { useCallback, useRef, type KeyboardEvent, type FormEvent } from "react";
import { Button } from "@/components/ui/Button";
import { Textarea } from "@/components/ui/Textarea";
import { useChatStore, MessageRole } from "@/stores/chatStore";
import { cn } from "@/lib/utils";

export function ChatInput() {
  const { inputValue, setInputValue, addMessage, isLoading } = useChatStore();
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = useCallback(
    (e?: FormEvent) => {
      e?.preventDefault();

      const trimmedValue = inputValue.trim();
      if (!trimmedValue || isLoading) return;

      // Add user message
      addMessage(MessageRole.User, trimmedValue);

      // Clear input
      setInputValue("");

      // Focus back on textarea
      textareaRef.current?.focus();
    },
    [inputValue, isLoading, addMessage, setInputValue]
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent<HTMLTextAreaElement>) => {
      // Submit on Enter (without Shift)
      if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSubmit();
      }
    },
    [handleSubmit]
  );

  return (
    <form onSubmit={handleSubmit} className="flex gap-2 p-3 border-t border-[hsl(var(--border))]">
      <Textarea
        ref={textareaRef}
        value={inputValue}
        onChange={(e) => setInputValue(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder="Type a message..."
        disabled={isLoading}
        className={cn(
          "resize-none min-h-[40px] max-h-[120px]",
          isLoading && "opacity-50"
        )}
        rows={1}
      />
      <Button
        type="submit"
        size="icon"
        disabled={!inputValue.trim() || isLoading}
        aria-label="Send message"
      >
        <SendIcon className="h-4 w-4" />
      </Button>
    </form>
  );
}

function SendIcon({ className }: { className?: string }) {
  return (
    <svg
      xmlns="http://www.w3.org/2000/svg"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="2"
      strokeLinecap="round"
      strokeLinejoin="round"
      className={className}
    >
      <path d="m22 2-7 20-4-9-9-4Z" />
      <path d="M22 2 11 13" />
    </svg>
  );
}
