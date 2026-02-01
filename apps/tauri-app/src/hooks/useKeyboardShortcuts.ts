// Keyboard shortcuts hook
import { useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";

interface ShortcutHandler {
  key: string;
  ctrl?: boolean;
  meta?: boolean;
  alt?: boolean;
  shift?: boolean;
  handler: () => void;
  description: string;
}

export function useKeyboardShortcuts() {
  const navigate = useNavigate();
  const {
    setTaskCreationOpen,
    setSettingsOpen,
    toggleCommandPalette,
    tabs,
    activeTabId,
    addTab,
    removeTab,
    setActiveTab,
  } = useUiStore();

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      const isMac = navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const modKey = isMac ? event.metaKey : event.ctrlKey;

      // Define shortcuts
      const shortcuts: ShortcutHandler[] = [
        // Navigation shortcuts
        {
          key: "n",
          ctrl: true,
          meta: true,
          handler: () => {
            setTaskCreationOpen(true);
            navigate("/tasks/new");
          },
          description: "New Task",
        },
        {
          key: ",",
          ctrl: true,
          meta: true,
          handler: () => {
            setSettingsOpen(true);
            navigate("/settings");
          },
          description: "Settings",
        },
        {
          key: "k",
          ctrl: true,
          meta: true,
          handler: () => {
            toggleCommandPalette();
          },
          description: "Command Palette",
        },
        {
          key: "1",
          ctrl: true,
          meta: true,
          handler: () => {
            navigate("/");
          },
          description: "Dashboard",
        },

        // Tab navigation
        {
          key: "t",
          ctrl: true,
          meta: true,
          handler: () => {
            const tabId = addTab({
              title: "New Tab",
              path: "/",
              closable: true,
            });
            setActiveTab(tabId);
          },
          description: "New Tab",
        },
        {
          key: "w",
          ctrl: true,
          meta: true,
          handler: () => {
            if (activeTabId) {
              removeTab(activeTabId);
            }
          },
          description: "Close Tab",
        },
        {
          key: "Tab",
          ctrl: true,
          meta: true,
          handler: () => {
            const currentIndex = tabs.findIndex((t) => t.id === activeTabId);
            const nextIndex = (currentIndex + 1) % tabs.length;
            setActiveTab(tabs[nextIndex].id);
          },
          description: "Next Tab",
        },

        // Quick tab switching (Cmd/Ctrl + 1-9)
        ...Array.from({ length: 9 }, (_, i) => ({
          key: String(i + 1),
          ctrl: true,
          meta: true,
          handler: () => {
            const tab = tabs[i];
            if (tab) {
              setActiveTab(tab.id);
            }
          },
          description: `Switch to Tab ${i + 1}`,
        })),

        // Dialog close
        {
          key: "Escape",
          handler: () => {
            setTaskCreationOpen(false);
            setSettingsOpen(false);
          },
          description: "Close Dialog",
        },
      ];

      // Check if any shortcut matches
      for (const shortcut of shortcuts) {
        const keyMatches =
          event.key.toLowerCase() === shortcut.key.toLowerCase();
        const modMatches =
          shortcut.ctrl || shortcut.meta ? modKey : !modKey;
        const altMatches = shortcut.alt ? event.altKey : !event.altKey;
        const shiftMatches = shortcut.shift ? event.shiftKey : !event.shiftKey;

        if (keyMatches && modMatches && altMatches && shiftMatches) {
          // Don't trigger shortcuts when typing in inputs
          const target = event.target as HTMLElement;
          if (
            target.tagName === "INPUT" ||
            target.tagName === "TEXTAREA" ||
            target.isContentEditable
          ) {
            // Allow Escape to still work
            if (shortcut.key !== "Escape") {
              return;
            }
          }

          event.preventDefault();
          shortcut.handler();
          return;
        }
      }
    },
    [
      navigate,
      setTaskCreationOpen,
      setSettingsOpen,
      toggleCommandPalette,
      tabs,
      activeTabId,
      addTab,
      removeTab,
      setActiveTab,
    ]
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
