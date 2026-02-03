// Chat state management with Zustand
import { create } from "zustand";

export enum MessageRole {
  User = "user",
  Assistant = "assistant",
}

export interface ChatMessage {
  id: string;
  role: MessageRole;
  content: string;
  createdAt: number;
}

interface ChatState {
  // Messages
  messages: ChatMessage[];
  addMessage: (role: MessageRole, content: string) => string;
  clearMessages: () => void;

  // Chat window visibility
  isOpen: boolean;
  setIsOpen: (open: boolean) => void;
  toggleChat: () => void;

  // Input state
  inputValue: string;
  setInputValue: (value: string) => void;

  // Loading state for AI responses
  isLoading: boolean;
  setIsLoading: (loading: boolean) => void;
}

let messageIdCounter = 0;

export const useChatStore = create<ChatState>((set) => ({
  // Messages
  messages: [],

  addMessage: (role, content) => {
    const id = `msg-${++messageIdCounter}`;
    const newMessage: ChatMessage = {
      id,
      role,
      content,
      createdAt: Date.now(),
    };

    set((state) => ({
      messages: [...state.messages, newMessage],
    }));

    return id;
  },

  clearMessages: () => set({ messages: [] }),

  // Chat window visibility
  isOpen: false,
  setIsOpen: (open) => set({ isOpen: open }),
  toggleChat: () => set((state) => ({ isOpen: !state.isOpen })),

  // Input state
  inputValue: "",
  setInputValue: (value) => set({ inputValue: value }),

  // Loading state
  isLoading: false,
  setIsLoading: (loading) => set({ isLoading: loading }),
}));
