// Keyboard shortcuts hook
import { useEffect, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";
import { useChatStore } from "@/stores/chatStore";

interface ShortcutHandler {
  key: string;
  mod?: boolean; // Cmd on Mac, Ctrl on Windows/Linux
  alt?: boolean;
  shift?: boolean;
  handler: () => void;
  description: string;
}

// Detect platform once at module level
const isMac = typeof navigator !== "undefined" && navigator.platform.toUpperCase().indexOf("MAC") >= 0;

export function useKeyboardShortcuts() {
  const navigate = useNavigate();
  const {
    setTaskCreationOpen,
    setSettingsOpen,
    toggleCommandPalette,
    toggleKeyboardShortcuts,
    setKeyboardShortcutsOpen,
    tabs,
    activeTabId,
    addTab,
    removeTab,
    setActiveTab,
  } = useUiStore();
  const { toggleChat, setOpen: setChatOpen } = useChatStore();

  // Memoize shortcuts array to prevent recreation on every render
  const shortcuts = useMemo<ShortcutHandler[]>(() => [
    // Navigation shortcuts
    {
      key: "n",
      mod: true,
      handler: () => {
        setTaskCreationOpen(true);
        navigate("/tasks/new");
      },
      description: "New Task",
    },
    {
      key: ",",
      mod: true,
      handler: () => {
        setSettingsOpen(true);
        navigate("/settings");
      },
      description: "Settings",
    },
    {
      key: "k",
      mod: true,
      handler: () => {
        toggleCommandPalette();
      },
      description: "Command Palette",
    },
    {
      key: "1",
      mod: true,
      handler: () => {
        navigate("/");
      },
      description: "Dashboard",
    },

    // Quick shortcuts (no modifier)
    {
      key: "?",
      shift: true,
      handler: () => {
        toggleKeyboardShortcuts();
      },
      description: "Show Keyboard Shortcuts",
    },
    {
      key: "c",
      handler: () => {
        setTaskCreationOpen(true);
        navigate("/tasks/new");
      },
      description: "Create Task",
    },

    // Tab navigation
    {
      key: "t",
      mod: true,
      handler: () => {
        const tabId = addTab({
          title: "Dashboard",
          path: "/",
          closable: true,
        });
        setActiveTab(tabId);
        navigate("/");
      },
      description: "New Tab",
    },
    {
      key: "w",
      mod: true,
      handler: () => {
        if (activeTabId) {
          removeTab(activeTabId);
        }
      },
      description: "Close Tab",
    },
    {
      key: "Tab",
      mod: true,
      handler: () => {
        const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
        const nextIndex = (currentIndex + 1) % tabs.length;
        if (tabs[nextIndex]) {
          setActiveTab(tabs[nextIndex].id);
        }
      },
      description: "Next Tab",
    },

    // Quick tab switching (Cmd/Ctrl + 1-9)
    ...Array.from({ length: 9 }, (_, i) => ({
      key: String(i + 1),
      mod: true,
      handler: () => {
        const tab = tabs[i];
        if (tab) {
          setActiveTab(tab.id);
        }
      },
      description: `Switch to Tab ${i + 1}`,
    })),

    // Chat toggle (Option+Z / Alt+Z)
    {
      key: "z",
      alt: true,
      handler: () => {
        toggleChat();
      },
      description: "Open Chat",
    },

    // Dialog close - Escape should work regardless of modifiers
    {
      key: "Escape",
      handler: () => {
        setTaskCreationOpen(false);
        setSettingsOpen(false);
        setChatOpen(false);
        setKeyboardShortcutsOpen(false);
      },
      description: "Close Dialog",
    },
  ], [
    navigate,
    setTaskCreationOpen,
    setSettingsOpen,
    toggleCommandPalette,
    toggleKeyboardShortcuts,
    setKeyboardShortcutsOpen,
    toggleChat,
    setChatOpen,
    tabs,
    activeTabId,
    addTab,
    removeTab,
    setActiveTab,
  ]);

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const modKey = isMac ? event.metaKey : event.ctrlKey;

      // Check if any shortcut matches
      for (const shortcut of shortcuts) {
        const keyMatches =
          event.key.toLowerCase() === shortcut.key.toLowerCase();
        // mod: true means Cmd on Mac, Ctrl on Windows/Linux
        // When modifier is undefined, require that modifier to NOT be pressed
        // This prevents conflicts with system shortcuts (e.g., Cmd+C for copy)
        // Exception: Escape key works regardless of modifiers for better UX
        const isEscape = shortcut.key === "Escape";
        const modMatches = isEscape || (shortcut.mod === undefined ? !modKey : (shortcut.mod ? modKey : !modKey));
        const altMatches = isEscape || (shortcut.alt === undefined ? !event.altKey : (shortcut.alt ? event.altKey : !event.altKey));
        const shiftMatches = isEscape || (shortcut.shift === undefined ? !event.shiftKey : (shortcut.shift ? event.shiftKey : !event.shiftKey));

        if (keyMatches && modMatches && altMatches && shiftMatches) {
          // Don't trigger shortcuts when typing in inputs
          const target = event.target as HTMLElement;
          if (
            target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable
          ) {
            // Allow Escape and Command Palette (Cmd/Ctrl+K) to still work
            if (shortcut.key !== "Escape" && shortcut.key !== "k") {
              return;
            }
          }

          event.preventDefault();
          shortcut.handler();
          return;
        }
      }
    },
    [shortcuts]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}

// Export shortcut definitions for display in UI
export const KEYBOARD_SHORTCUTS = {
  global: [
    { keys: ["?"], description: "Show Keyboard Shortcuts" },
    { keys: ["c"], description: "Create Task" },
    { keys: ["⌥/Alt", "Z"], description: "Open Chat" },
    { keys: ["⌘/Ctrl", "N"], description: "New Task" },
    { keys: ["⌘/Ctrl", ","], description: "Settings" },
    { keys: ["⌘/Ctrl", "K"], description: "Command Palette" },
    { keys: ["⌘/Ctrl", "1"], description: "Dashboard" },
    { keys: ["Escape"], description: "Close Dialog" },
  ],
  tabs: [
    { keys: ["⌘/Ctrl", "T"], description: "New Tab" },
    { keys: ["⌘/Ctrl", "W"], description: "Close Tab" },
    { keys: ["⌘/Ctrl", "Tab"], description: "Next Tab" },
    { keys: ["⌘/Ctrl", "1-9"], description: "Switch Tab" },
  ],
  review: [
    { keys: ["J", "K"], description: "Navigate Files" },
    { keys: ["Enter"], description: "Open File" },
    { keys: ["⌘/Ctrl", "Enter"], description: "Approve" },
  ],
  taskDetail: [
    { keys: ["A"], description: "Approve" },
    { keys: ["D"], description: "Deny" },
    { keys: ["L"], description: "Toggle Log" },
    { keys: ["S"], description: "Stop Execution" },
  ],
};
