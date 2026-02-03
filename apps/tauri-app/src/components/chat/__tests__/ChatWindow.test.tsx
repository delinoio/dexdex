import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ChatWindow } from "../ChatWindow";
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

describe("ChatWindow", () => {
  const mockSetOpen = vi.fn();
  const mockClearMessages = vi.fn();
  const mockToggleChat = vi.fn();
  const mockAddMessage = vi.fn();
  const mockSetInputValue = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      isOpen: true,
      setOpen: mockSetOpen,
      toggleChat: mockToggleChat,
      messages: [],
      addMessage: mockAddMessage,
      clearMessages: mockClearMessages,
      inputValue: "",
      setInputValue: mockSetInputValue,
      isLoading: false,
      setLoading: vi.fn(),
    });
  });

  describe("Rendering", () => {
    it("renders when isOpen is true", () => {
      render(<ChatWindow />);
      expect(screen.getByText("Chat")).toBeInTheDocument();
    });

    it("does not render when isOpen is false", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: false,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: false,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);
      expect(screen.queryByText("Chat")).not.toBeInTheDocument();
    });

    it("renders empty state when no messages", () => {
      render(<ChatWindow />);
      expect(screen.getByText("Start a conversation...")).toBeInTheDocument();
    });

    it("renders input placeholder", () => {
      render(<ChatWindow />);
      expect(
        screen.getByPlaceholderText("Type a message...")
      ).toBeInTheDocument();
    });

    it("does not show clear button when no messages", () => {
      render(<ChatWindow />);
      expect(screen.queryByText("Clear")).not.toBeInTheDocument();
    });

    it("shows clear button when there are messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: true,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: false,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);
      expect(screen.getByText("Clear")).toBeInTheDocument();
    });
  });

  describe("User Interaction", () => {
    it("closes when clicking backdrop", () => {
      render(<ChatWindow />);

      // Find the backdrop element (fixed overlay)
      const backdrop = document.querySelector(".fixed.inset-0");
      if (backdrop) {
        // Simulate clicking the backdrop itself (not inside the dialog)
        fireEvent.click(backdrop);
        expect(mockSetOpen).toHaveBeenCalledWith(false);
      }
    });

    it("closes when clicking close button", () => {
      render(<ChatWindow />);

      const closeButton = screen.getByLabelText("Close chat");
      fireEvent.click(closeButton);

      expect(mockSetOpen).toHaveBeenCalledWith(false);
    });

    it("clears messages when clicking clear button", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: true,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: false,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);

      const clearButton = screen.getByText("Clear");
      fireEvent.click(clearButton);

      expect(mockClearMessages).toHaveBeenCalled();
    });
  });

  describe("Messages Display", () => {
    it("renders user messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: true,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello there", timestamp: new Date() },
        ],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: false,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);

      expect(screen.getByText("Hello there")).toBeInTheDocument();
    });

    it("renders assistant messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: true,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [
          { id: "msg-1", role: MessageRole.Assistant, content: "Hi, how can I help?", timestamp: new Date() },
        ],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: false,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);

      expect(screen.getByText("Hi, how can I help?")).toBeInTheDocument();
    });

    it("shows loading indicator when isLoading is true", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isOpen: true,
        setOpen: mockSetOpen,
        toggleChat: mockToggleChat,
        messages: [
          { id: "msg-1", role: MessageRole.User, content: "Hello", timestamp: new Date() },
        ],
        addMessage: mockAddMessage,
        clearMessages: mockClearMessages,
        inputValue: "",
        setInputValue: mockSetInputValue,
        isLoading: true,
        setLoading: vi.fn(),
      });
      render(<ChatWindow />);

      expect(screen.getByText("Thinking...")).toBeInTheDocument();
    });
  });
});
