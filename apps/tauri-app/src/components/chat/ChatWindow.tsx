import { useChatStore } from "@/stores/chatStore";
import { Button } from "@/components/ui/Button";
import { CloseIcon, ChatIcon } from "@/components/ui/Icons";
import { MessageList } from "./MessageList";
import { ChatInput } from "./ChatInput";
import { cn } from "@/lib/utils";

interface ChatWindowProps {
  className?: string;
}

export function ChatWindow({ className }: ChatWindowProps) {
  const { isOpen, setOpen, clearMessages, messages } = useChatStore();

  if (!isOpen) {
    return null;
  }

  return (
    <div
      className={cn(
        "fixed inset-0 z-50 bg-black/50 flex items-center justify-center",
        className
      )}
      onClick={(e) => {
        // Close when clicking backdrop
        if (e.target === e.currentTarget) {
          setOpen(false);
        }
      }}
    >
      <div className="bg-[hsl(var(--background))] border border-[hsl(var(--border))] rounded-lg shadow-xl w-full max-w-md h-[600px] max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-4 py-3 border-b border-[hsl(var(--border))]">
          <div className="flex items-center gap-2">
            <ChatIcon className="h-5 w-5 text-[hsl(var(--muted-foreground))]" />
            <h2 className="text-lg font-semibold">Chat</h2>
          </div>
          <div className="flex items-center gap-1">
            {messages.length > 0 && (
              <Button
                variant="ghost"
                size="sm"
                onClick={clearMessages}
                className="text-xs"
              >
                Clear
              </Button>
            )}
            <Button
              variant="ghost"
              size="icon"
              onClick={() => setOpen(false)}
              aria-label="Close chat"
            >
              <CloseIcon className="h-4 w-4" />
            </Button>
          </div>
        </div>

        {/* Message List */}
        <MessageList className="flex-1" />

        {/* Input */}
        <ChatInput />
      </div>
    </div>
  );
}
