import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { ChatInput } from "../ChatInput";
import { useChatStore, MessageRole } from "@/stores/chatStore";

// Mock zustand store
vi.mock("@/stores/chatStore", () => ({
  useChatStore: vi.fn(),
  MessageRole: {
    User: "user",
    Assistant: "assistant",
  },
}));

describe("ChatInput", () => {
  const mockSetInputValue = vi.fn();
  const mockAddMessage = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      inputValue: "",
      setInputValue: mockSetInputValue,
      addMessage: mockAddMessage,
      isLoading: false,
    });
  });

  describe("Rendering", () => {
    it("renders textarea with placeholder", () => {
      render(<ChatInput />);
      expect(
        screen.getByPlaceholderText("Type a message...")
      ).toBeInTheDocument();
    });

    it("renders send button", () => {
      render(<ChatInput />);
      expect(screen.getByLabelText("Send message")).toBeInTheDocument();
    });

    it("renders voice input button (disabled)", () => {
      render(<ChatInput />);
      const micButton = screen.getByLabelText("Voice input");
      expect(micButton).toBeInTheDocument();
      expect(micButton).toBeDisabled();
    });

    it("send button is disabled when input is empty", () => {
      render(<ChatInput />);
      const sendButton = screen.getByLabelText("Send message");
      expect(sendButton).toBeDisabled();
    });

    it("send button is enabled when input has text", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: false,
      });
      render(<ChatInput />);
      const sendButton = screen.getByLabelText("Send message");
      expect(sendButton).not.toBeDisabled();
    });
  });

  describe("User Interaction", () => {
    it("updates input value on change", () => {
      render(<ChatInput />);

      const textarea = screen.getByPlaceholderText("Type a message...");
      fireEvent.change(textarea, { target: { value: "Hello" } });

      expect(mockSetInputValue).toHaveBeenCalledWith("Hello");
    });

    it("sends message on button click", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello there",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: false,
      });
      render(<ChatInput />);

      const sendButton = screen.getByLabelText("Send message");
      fireEvent.click(sendButton);

      expect(mockAddMessage).toHaveBeenCalledWith(MessageRole.User, "Hello there");
      expect(mockSetInputValue).toHaveBeenCalledWith("");
    });

    it("sends message on Enter key", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello there",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: false,
      });
      render(<ChatInput />);

      const textarea = screen.getByPlaceholderText("Type a message...");
      fireEvent.keyDown(textarea, { key: "Enter", shiftKey: false });

      expect(mockAddMessage).toHaveBeenCalledWith(MessageRole.User, "Hello there");
      expect(mockSetInputValue).toHaveBeenCalledWith("");
    });

    it("does not send on Shift+Enter (allows multiline)", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello there",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: false,
      });
      render(<ChatInput />);

      const textarea = screen.getByPlaceholderText("Type a message...");
      fireEvent.keyDown(textarea, { key: "Enter", shiftKey: true });

      expect(mockAddMessage).not.toHaveBeenCalled();
    });

    it("does not send empty or whitespace-only messages", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "   ",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: false,
      });
      render(<ChatInput />);

      const sendButton = screen.getByLabelText("Send message");
      // Button should be disabled for whitespace-only input
      expect(sendButton).toBeDisabled();
    });
  });

  describe("Loading State", () => {
    it("disables input when loading", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: true,
      });
      render(<ChatInput />);

      const textarea = screen.getByPlaceholderText("Type a message...");
      expect(textarea).toBeDisabled();
    });

    it("disables send button when loading", () => {
      (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        inputValue: "Hello",
        setInputValue: mockSetInputValue,
        addMessage: mockAddMessage,
        isLoading: true,
      });
      render(<ChatInput />);

      const sendButton = screen.getByLabelText("Send message");
      expect(sendButton).toBeDisabled();
    });
  });
});
