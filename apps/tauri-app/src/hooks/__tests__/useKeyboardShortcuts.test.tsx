// useKeyboardShortcuts tests
import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useKeyboardShortcuts } from "../useKeyboardShortcuts";

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock("react-router-dom", () => ({
  useNavigate: () => mockNavigate,
}));

// Mock stores
const mockSetTaskCreationOpen = vi.fn();
const mockSetSettingsOpen = vi.fn();
const mockToggleCommandPalette = vi.fn();
const mockToggleKeyboardShortcuts = vi.fn();
const mockSetKeyboardShortcutsOpen = vi.fn();
const mockAddTab = vi.fn(() => "tab-1");
const mockRemoveTab = vi.fn();
const mockSetActiveTab = vi.fn();
const mockToggleChat = vi.fn();
const mockSetChatOpen = vi.fn();

vi.mock("@/stores/uiStore", () => ({
  useUiStore: () => ({
    setTaskCreationOpen: mockSetTaskCreationOpen,
    setSettingsOpen: mockSetSettingsOpen,
    toggleCommandPalette: mockToggleCommandPalette,
    toggleKeyboardShortcuts: mockToggleKeyboardShortcuts,
    setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
    tabs: [{ id: "tab-1", title: "Test", path: "/" }],
    activeTabId: "tab-1",
    addTab: mockAddTab,
    removeTab: mockRemoveTab,
    setActiveTab: mockSetActiveTab,
  }),
}));

vi.mock("@/stores/chatStore", () => ({
  useChatStore: () => ({
    toggleChat: mockToggleChat,
    setOpen: mockSetChatOpen,
  }),
}));

describe("useKeyboardShortcuts", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    // Default to non-Mac platform
    Object.defineProperty(navigator, "platform", {
      value: "Win32",
      writable: true,
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe("Modifier key handling", () => {
    it("should trigger 'c' shortcut only when no modifiers are pressed", () => {
      renderHook(() => useKeyboardShortcuts());

      // Press 'c' without modifiers - should trigger
      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "c",
            ctrlKey: false,
            metaKey: false,
            altKey: false,
            shiftKey: false,
          })
        );
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(true);
      expect(mockNavigate).toHaveBeenCalledWith("/tasks/new");
    });

    it("should NOT trigger 'c' shortcut when Ctrl is pressed (for copy)", () => {
      renderHook(() => useKeyboardShortcuts());

      // Press Ctrl+C - should NOT trigger 'c' shortcut
      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "c",
            ctrlKey: true,
            metaKey: false,
            altKey: false,
            shiftKey: false,
          })
        );
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
      expect(mockNavigate).not.toHaveBeenCalled();
    });

    it("should NOT trigger 'c' shortcut when both Ctrl and Meta keys are pressed", () => {
      // Test that shortcuts don't trigger with multiple modifiers pressed
      renderHook(() => useKeyboardShortcuts());

      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "c",
            ctrlKey: true,
            metaKey: true,
            altKey: false,
            shiftKey: false,
          })
        );
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
    });

    it("should NOT trigger 'c' shortcut when Alt is pressed", () => {
      renderHook(() => useKeyboardShortcuts());

      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "c",
            ctrlKey: false,
            metaKey: false,
            altKey: true,
            shiftKey: false,
          })
        );
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
    });

    it("should NOT trigger 'c' shortcut when Shift is pressed", () => {
      renderHook(() => useKeyboardShortcuts());

      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "c",
            ctrlKey: false,
            metaKey: false,
            altKey: false,
            shiftKey: true,
          })
        );
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
    });

    it("should trigger '?' shortcut with Shift pressed", () => {
      renderHook(() => useKeyboardShortcuts());

      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "?",
            ctrlKey: false,
            metaKey: false,
            altKey: false,
            shiftKey: true,
          })
        );
      });

      expect(mockToggleKeyboardShortcuts).toHaveBeenCalled();
    });

    it("should trigger Escape regardless of modifiers", () => {
      renderHook(() => useKeyboardShortcuts());

      // Escape with Ctrl pressed - should still work
      act(() => {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            key: "Escape",
            ctrlKey: true,
            metaKey: false,
            altKey: false,
            shiftKey: false,
          })
        );
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(false);
      expect(mockSetSettingsOpen).toHaveBeenCalledWith(false);
      expect(mockSetChatOpen).toHaveBeenCalledWith(false);
      expect(mockSetKeyboardShortcutsOpen).toHaveBeenCalledWith(false);
    });
  });

  describe("Input field handling", () => {
    it("should NOT trigger 'c' shortcut when typing in input field", () => {
      renderHook(() => useKeyboardShortcuts());

      // Create an input element and make it the event target
      const input = document.createElement("input");
      document.body.appendChild(input);

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "c",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
          bubbles: true,
        });

        // Override target to be the input
        Object.defineProperty(event, "target", {
          value: input,
          writable: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
      expect(mockNavigate).not.toHaveBeenCalled();

      document.body.removeChild(input);
    });

    it("should NOT trigger '?' shortcut when typing in textarea", () => {
      renderHook(() => useKeyboardShortcuts());

      const textarea = document.createElement("textarea");
      document.body.appendChild(textarea);

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "?",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: true,
          bubbles: true,
        });

        Object.defineProperty(event, "target", {
          value: textarea,
          writable: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockToggleKeyboardShortcuts).not.toHaveBeenCalled();

      document.body.removeChild(textarea);
    });

    it("should NOT trigger 'c' shortcut when typing in contentEditable element", () => {
      renderHook(() => useKeyboardShortcuts());

      const div = document.createElement("div");
      div.contentEditable = "true";
      // JSDOM doesn't properly implement isContentEditable, so we need to mock it
      Object.defineProperty(div, "isContentEditable", {
        value: true,
        writable: false,
      });
      document.body.appendChild(div);

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "c",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
          bubbles: true,
        });

        Object.defineProperty(event, "target", {
          value: div,
          writable: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();

      document.body.removeChild(div);
    });

    it("should allow Escape in input fields", () => {
      renderHook(() => useKeyboardShortcuts());

      const input = document.createElement("input");
      document.body.appendChild(input);

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "Escape",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
          bubbles: true,
        });

        Object.defineProperty(event, "target", {
          value: input,
          writable: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(false);
      expect(mockSetSettingsOpen).toHaveBeenCalledWith(false);

      document.body.removeChild(input);
    });

    it("should allow Cmd+K (Command Palette) in input fields", () => {
      renderHook(() => useKeyboardShortcuts());

      const input = document.createElement("input");
      document.body.appendChild(input);

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "k",
          ctrlKey: true,
          metaKey: false,
          altKey: false,
          shiftKey: false,
          bubbles: true,
        });

        Object.defineProperty(event, "target", {
          value: input,
          writable: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockToggleCommandPalette).toHaveBeenCalled();

      document.body.removeChild(input);
    });
  });

  describe("Cleanup", () => {
    it("should remove event listener on unmount", () => {
      const removeEventListenerSpy = vi.spyOn(window, "removeEventListener");

      const { unmount } = renderHook(() => useKeyboardShortcuts());

      unmount();

      expect(removeEventListenerSpy).toHaveBeenCalledWith(
        "keydown",
        expect.any(Function)
      );
    });
  });
});
