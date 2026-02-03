import { type KeyboardEvent, type ChangeEvent, useRef, useEffect } from "react";
import { Button } from "@/components/ui/Button";
import { Textarea } from "@/components/ui/Textarea";
import { useChatStore, MessageRole } from "@/stores/chatStore";
import { cn } from "@/lib/utils";
import { SendIcon, MicIcon } from "@/components/ui/Icons";

interface ChatInputProps {
  className?: string;
}

export function ChatInput({ className }: ChatInputProps) {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { inputValue, setInputValue, addMessage, isLoading, isOpen } = useChatStore();

  // Auto-focus the textarea when the chat opens
  useEffect(() => {
    if (isOpen) {
      textareaRef.current?.focus();
    }
  }, [isOpen]);

  const handleChange = (e: ChangeEvent<HTMLTextAreaElement>) => {
    setInputValue(e.target.value);
  };

  const handleSend = () => {
    const trimmedValue = inputValue.trim();
    if (!trimmedValue || isLoading) return;

    // Add user message
    addMessage(MessageRole.User, trimmedValue);
    setInputValue("");

    // TODO: Integrate with AI backend to get response
    // For now, we just add the user message
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    // Send on Enter (without Shift)
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <div className={cn("flex items-end gap-2 p-3 border-t border-[hsl(var(--border))]", className)}>
      <Textarea
        ref={textareaRef}
        value={inputValue}
        onChange={handleChange}
        onKeyDown={handleKeyDown}
        placeholder="Type a message..."
        className="min-h-[40px] max-h-[120px] resize-none flex-1"
        rows={1}
        disabled={isLoading}
      />
      <Button
        variant="ghost"
        size="icon"
        className="shrink-0"
        aria-label="Voice input"
        disabled
      >
        <MicIcon className="h-5 w-5" />
      </Button>
      <Button
        variant="default"
        size="icon"
        className="shrink-0"
        onClick={handleSend}
        disabled={!inputValue.trim() || isLoading}
        aria-label="Send message"
      >
        <SendIcon className="h-5 w-5" />
      </Button>
    </div>
  );
}
