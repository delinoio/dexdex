import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import { MessageList } from "../MessageList";
import { useChatStore, MessageRole } from "@/stores/chatStore";

// Mock zustand store
vi.mock("@/stores/chatStore", () => ({
  useChatStore: vi.fn(),
  MessageRole: {
    User: "user",
    Assistant: "assistant",
  },
}));

// Mock scrollIntoView
Element.prototype.scrollIntoView = vi.fn();

describe("MessageList", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Empty State", () => {
    it("shows empty state when no messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [],
        isLoading: false,
      });
      render(<MessageList />);

      expect(screen.getByText("Start a conversation...")).toBeInTheDocument();
    });
  });

  describe("Messages Display", () => {
    it("renders user messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello!", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      expect(screen.getByText("Hello!")).toBeInTheDocument();
    });

    it("renders assistant messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.Assistant, content: "Hi there!", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      expect(screen.getByText("Hi there!")).toBeInTheDocument();
    });

    it("renders multiple messages in order", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "First message", timestamp: new Date() },
          { id: "msg-2", role: MessageRole.Assistant, content: "Second message", timestamp: new Date() },
          { id: "msg-3", role: MessageRole.User, content: "Third message", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      expect(screen.getByText("First message")).toBeInTheDocument();
      expect(screen.getByText("Second message")).toBeInTheDocument();
      expect(screen.getByText("Third message")).toBeInTheDocument();
    });

    it("preserves whitespace in messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Line 1\nLine 2", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      const message = screen.getByText(/Line 1/);
      expect(message).toHaveClass("whitespace-pre-wrap");
    });
  });

  describe("Loading State", () => {
    it("shows loading indicator when isLoading is true", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        isLoading: true,
      });
      render(<MessageList />);

      expect(screen.getByText("Thinking...")).toBeInTheDocument();
    });

    it("does not show loading indicator when isLoading is false", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      expect(screen.queryByText("Thinking...")).not.toBeInTheDocument();
    });

    it("does not show loading in empty state", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [],
        isLoading: true,
      });
      render(<MessageList />);

      // Empty state should show, not loading
      expect(screen.getByText("Start a conversation...")).toBeInTheDocument();
      expect(screen.queryByText("Thinking...")).not.toBeInTheDocument();
    });
  });

  describe("Auto-scroll", () => {
    it("calls scrollIntoView when messages change", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      expect(Element.prototype.scrollIntoView).toHaveBeenCalled();
    });
  });

  describe("Message Styling", () => {
    it("applies different styling for user vs assistant messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "User message", timestamp: new Date() },
          { id: "msg-2", role: MessageRole.Assistant, content: "Assistant message", timestamp: new Date() },
        ],
        isLoading: false,
      });
      render(<MessageList />);

      const userMessage = screen.getByText("User message");
      const assistantMessage = screen.getByText("Assistant message");

      // User messages should have primary styling
      expect(userMessage.parentElement).toHaveClass("bg-[hsl(var(--primary))]");

      // Assistant messages should have muted styling
      expect(assistantMessage.parentElement).toHaveClass("bg-[hsl(var(--muted))]");
    });
  });
});
