import { useRef, useEffect, useState } from "react";
import { useChatStore, MessageRole, type ChatMessage } from "@/stores/chatStore";
import { cn } from "@/lib/utils";

interface MessageBubbleProps {
  message: ChatMessage;
}

function MessageBubble({ message }: MessageBubbleProps) {
  const isUser = message.role === MessageRole.User;

  return (
    <div
      className={cn(
        "flex w-full",
        isUser ? "justify-end" : "justify-start"
      )}
    >
      <div
        className={cn(
          "max-w-[80%] rounded-lg px-4 py-2 text-sm",
          isUser
            ? "bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]"
            : "bg-[hsl(var(--muted))] text-[hsl(var(--foreground))]"
        )}
      >
        <p className="whitespace-pre-wrap break-words">{message.content}</p>
      </div>
    </div>
  );
}

interface MessageListProps {
  className?: string;
}

export function MessageList({ className }: MessageListProps) {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const { messages, isLoading } = useChatStore();
  const [prevMessageCount, setPrevMessageCount] = useState(messages.length);

  // Auto-scroll to bottom only when new messages are added
  useEffect(() => {
    if (messages.length > prevMessageCount) {
      messagesEndRef.current?.scrollIntoView({ behavior: "smooth" });
    }
    setPrevMessageCount(messages.length);
  }, [messages.length, prevMessageCount]);

  return (
    <div className={cn("flex-1 overflow-y-auto p-4 space-y-3", className)}>
      {messages.length === 0 ? (
        <div className="flex items-center justify-center h-full text-[hsl(var(--muted-foreground))] text-sm">
          <p>Start a conversation...</p>
        </div>
      ) : (
        <>
          {messages.map((message) => (
            <MessageBubble key={message.id} message={message} />
          ))}
          {isLoading && (
            <div className="flex justify-start">
              <div className="bg-[hsl(var(--muted))] rounded-lg px-4 py-2 text-sm">
                <span className="animate-pulse">Thinking...</span>
              </div>
            </div>
          )}
        </>
      )}
      <div ref={messagesEndRef} />
    </div>
  );
}
