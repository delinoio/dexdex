import { useEffect, useRef } from "react";
import { useChatStore, MessageRole, type ChatMessage } from "@/stores/chatStore";
import { cn } from "@/lib/utils";

export function MessageList() {
  const { messages, isLoading } = useChatStore();
  const containerRef = useRef<HTMLDivElement>(null);

  // Auto-scroll to bottom when new messages arrive
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight;
    }
  }, [messages]);

  if (messages.length === 0) {
    return (
      <div className="flex-1 flex items-center justify-center p-4 text-[hsl(var(--muted-foreground))]">
        <p>Start a conversation...</p>
      </div>
    );
  }

  return (
    <div ref={containerRef} className="flex-1 overflow-y-auto p-4 space-y-4">
      {messages.map((message) => (
        <MessageBubble key={message.id} message={message} />
      ))}
      {isLoading && (
        <div className="flex justify-start">
          <div className="bg-[hsl(var(--muted))] rounded-lg px-4 py-2">
            <LoadingDots />
          </div>
        </div>
      )}
    </div>
  );
}

interface MessageBubbleProps {
  message: ChatMessage;
}

function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === MessageRole.User;

  return (
    <div className={cn("flex", isUser ? "justify-end" : "justify-start")}>
      <div
        className={cn(
          "max-w-[80%] rounded-lg px-4 py-2 whitespace-pre-wrap break-words",
          isUser
            ? "bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]"
            : "bg-[hsl(var(--muted))] text-[hsl(var(--foreground))]"
        )}
      >
        {message.content}
      </div>
    </div>
  );
}

function LoadingDots() {
  return (
    <div className="flex gap-1">
      <span className="w-2 h-2 bg-[hsl(var(--muted-foreground))] rounded-full animate-bounce [animation-delay:-0.3s]" />
      <span className="w-2 h-2 bg-[hsl(var(--muted-foreground))] rounded-full animate-bounce [animation-delay:-0.15s]" />
      <span className="w-2 h-2 bg-[hsl(var(--muted-foreground))] rounded-full animate-bounce" />
    </div>
  );
}
