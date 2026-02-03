import { useEffect, useCallback } from "react";
import { useChatStore } from "@/stores/chatStore";
import { MessageList } from "./MessageList";
import { ChatInput } from "./ChatInput";
import { Button } from "@/components/ui/Button";
import { cn } from "@/lib/utils";

export function ChatWindow() {
  const { isOpen, setIsOpen } = useChatStore();

  // Close on Escape
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape" && isOpen) {
        setIsOpen(false);
      }
    },
    [isOpen, setIsOpen]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [handleKeyDown]);

  if (!isOpen) {
    return null;
  }

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={() => setIsOpen(false)}
        aria-hidden="true"
      />

      {/* Chat Panel */}
      <div
        className={cn(
          "fixed right-0 top-0 h-full w-full sm:w-[400px] bg-[hsl(var(--background))] border-l border-[hsl(var(--border))] z-50",
          "flex flex-col shadow-lg",
          "animate-in slide-in-from-right duration-200"
        )}
        role="dialog"
        aria-label="Chat"
      >
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-[hsl(var(--border))]">
          <h2 className="text-lg font-semibold">Chat</h2>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setIsOpen(false)}
            aria-label="Close chat"
          >
            <CloseIcon className="h-4 w-4" />
          </Button>
        </div>

        {/* Messages */}
        <MessageList />

        {/* Input */}
        <ChatInput />
      </div>
    </>
  );
}

function CloseIcon({ className }: { className?: string }) {
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
      <path d="M18 6 6 18" />
      <path d="m6 6 12 12" />
    </svg>
  );
}
