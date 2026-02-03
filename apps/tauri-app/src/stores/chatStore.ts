// Chat state management with Zustand
import { create } from "zustand";

// Message roles for chat
export enum MessageRole {
  User = "user",
  Assistant = "assistant",
}

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  timestamp: Date;
}

interface ChatState {
  // Chat window visibility
  isOpen: boolean;
  setOpen: (open: boolean) => void;
  toggleChat: () => void;

  // Message history
  messages: ChatMessage[];
  addMessage: (role: MessageRole, content: string) => void;
  clearMessages: () => void;

  // Current input
  inputValue: string;
  setInputValue: (value: string) => void;

  // Loading state (for AI responses)
  isLoading: boolean;
  setLoading: (loading: boolean) => void;
}

let messageIdCounter = 0;

export const useChatStore = create<ChatState>()((set) => ({
  // Chat window visibility
  isOpen: false,
  setOpen: (open) => set({ isOpen: open }),
  toggleChat: () => set((state) => ({ isOpen: !state.isOpen })),

  // Message history
  messages: [],
  addMessage: (role, content) =>
    set((state) => ({
      messages: [
        ...state.messages,
        {
          id: `msg-${++messageIdCounter}`,
          role,
          content,
          timestamp: new Date(),
        },
      ],
    })),
  clearMessages: () => set({ messages: [] }),

  // Current input
  inputValue: "",
  setInputValue: (value) => set({ inputValue: value }),

  // Loading state
  isLoading: false,
  setLoading: (loading) => set({ isLoading: loading }),
}));
