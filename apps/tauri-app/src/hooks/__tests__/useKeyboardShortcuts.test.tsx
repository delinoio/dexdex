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

const mockToggleNotifications = vi.fn();
const mockSetNotificationsOpen = vi.fn();

vi.mock("@/stores/notificationCenterStore", () => ({
  useNotificationCenterStore: () => ({
    toggleOpen: mockToggleNotifications,
    setOpen: mockSetNotificationsOpen,
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

  describe("Keyboard layout awareness", () => {
    it("should trigger 'c' shortcut via event.code when event.key is a non-Latin character (e.g., Korean)", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate pressing physical 'C' key on a Korean keyboard layout
      // event.key would be 'ㅊ' but event.code would be 'KeyC'
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "ㅊ",
          code: "KeyC",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(true);
      expect(mockNavigate).toHaveBeenCalledWith("/tasks/new");
    });

    it("should trigger 'n' shortcut via event.code when event.key is a non-Latin character (e.g., Russian)", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate pressing physical 'N' key on a Russian keyboard layout
      // event.key would be 'т' but event.code would be 'KeyN'
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "т",
          code: "KeyN",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockToggleNotifications).toHaveBeenCalled();
    });

    it("should trigger Ctrl+K shortcut via event.code on non-Latin layout", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate Ctrl+K on a Korean layout where event.key may be 'ㅏ'
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "ㅏ",
          code: "KeyK",
          ctrlKey: true,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockToggleCommandPalette).toHaveBeenCalled();
    });

    it("should trigger Alt+Z shortcut via event.code on non-Latin layout", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate Alt+Z on a Korean layout
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "ㅋ",
          code: "KeyZ",
          altKey: true,
          ctrlKey: false,
          metaKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockToggleChat).toHaveBeenCalled();
    });

    it("should trigger Ctrl+N shortcut via event.code on non-Latin layout", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate Ctrl+N on a Russian layout where 'N' key produces 'т'
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "т",
          code: "KeyN",
          ctrlKey: true,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(true);
      expect(mockNavigate).toHaveBeenCalledWith("/tasks/new");
    });

    it("should still work with English layout (event.key matches directly)", () => {
      renderHook(() => useKeyboardShortcuts());

      // Normal English layout - event.key is 'c' and event.code is 'KeyC'
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "c",
          code: "KeyC",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).toHaveBeenCalledWith(true);
      expect(mockNavigate).toHaveBeenCalledWith("/tasks/new");
    });

    it("should NOT trigger shortcut when event.code is unknown and event.key doesn't match", () => {
      renderHook(() => useKeyboardShortcuts());

      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "ㅊ",
          code: "UnknownKey",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
      expect(mockNavigate).not.toHaveBeenCalled();
    });

    it("should handle shifted keys on non-Latin layouts", () => {
      renderHook(() => useKeyboardShortcuts());

      // Shift+KeyC on Russian layout produces uppercase Cyrillic 'С'
      // event.code is still 'KeyC', so it should fall back to physical key
      // But since shift is pressed and 'c' shortcut requires no shift, it should NOT trigger
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "С",
          code: "KeyC",
          ctrlKey: false,
          metaKey: false,
          altKey: false,
          shiftKey: true,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetTaskCreationOpen).not.toHaveBeenCalled();
    });

    it("should trigger Ctrl+, (Settings) shortcut via event.code on non-Latin layout", () => {
      renderHook(() => useKeyboardShortcuts());

      // Simulate Ctrl+, on a non-Latin layout
      // The Comma key code maps to ','
      act(() => {
        const event = new KeyboardEvent("keydown", {
          key: "б",
          code: "Comma",
          ctrlKey: true,
          metaKey: false,
          altKey: false,
          shiftKey: false,
        });

        window.dispatchEvent(event);
      });

      expect(mockSetSettingsOpen).toHaveBeenCalledWith(true);
      expect(mockNavigate).toHaveBeenCalledWith("/settings");
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
