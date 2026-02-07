import { useState, useEffect, useRef, useCallback, useMemo } from "react";
import { useNavigate } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";
import { useChatStore } from "@/stores/chatStore";
import { cn } from "@/lib/utils";
import {
  HomeIcon,
  PlusIcon,
  SettingsIcon,
  FolderIcon,
  CompositeTaskIcon,
  SearchIcon,
  ChatIcon,
} from "@/components/ui/Icons";

enum CommandId {
  NewTask = "new-task",
  Dashboard = "dashboard",
  Settings = "settings",
  Repositories = "repositories",
  RepositoryGroups = "repository-groups",
  OpenChat = "open-chat",
}

interface Command {
  id: CommandId;
  label: string;
  icon: React.ReactNode;
  path?: string;
  action?: () => void;
  keywords: string[];
}

export function CommandPalette() {
  const navigate = useNavigate();
  const { isCommandPaletteOpen, setCommandPaletteOpen } = useUiStore();
  const { setOpen: setChatOpen } = useChatStore();

  const commands: Command[] = useMemo(
    () => [
      {
        id: CommandId.NewTask,
        label: "New Task",
        icon: <PlusIcon size={16} />,
        path: "/tasks/new",
        keywords: ["create", "add", "task", "new"],
      },
      {
        id: CommandId.Dashboard,
        label: "Dashboard",
        icon: <HomeIcon size={16} />,
        path: "/",
        keywords: ["home", "main", "overview", "kanban"],
      },
      {
        id: CommandId.Settings,
        label: "Settings",
        icon: <SettingsIcon size={16} />,
        path: "/settings",
        keywords: ["preferences", "config", "configuration", "options"],
      },
      {
        id: CommandId.Repositories,
        label: "Repositories",
        icon: <FolderIcon size={16} />,
        path: "/repositories",
        keywords: ["repos", "git", "code", "projects"],
      },
      {
        id: CommandId.RepositoryGroups,
        label: "Repository Groups",
        icon: <CompositeTaskIcon size={16} />,
        path: "/repository-groups",
        keywords: ["groups", "collections", "multi-repo"],
      },
      {
        id: CommandId.OpenChat,
        label: "Open Chat",
        icon: <ChatIcon size={16} />,
        action: () => setChatOpen(true),
        keywords: ["chat", "message", "ai", "assistant", "talk"],
      },
    ],
    [setChatOpen]
  );
  const [searchQuery, setSearchQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const previousActiveElement = useRef<HTMLElement | null>(null);

  const filteredCommands = useMemo(() => {
    if (!searchQuery.trim()) {
      return commands;
    }

    const query = searchQuery.toLowerCase();
    return commands.filter((command) => {
      const labelMatch = command.label.toLowerCase().includes(query);
      const keywordMatch = command.keywords.some((keyword) =>
        keyword.includes(query)
      );
      return labelMatch || keywordMatch;
    });
  }, [searchQuery, commands]);

  // Reset state when opening, manage focus trap
  useEffect(() => {
    if (isCommandPaletteOpen) {
      // Store the previously focused element to restore focus on close
      previousActiveElement.current = document.activeElement as HTMLElement;
      setSearchQuery("");
      setSelectedIndex(0);
      // Defer focus to next tick to ensure dialog is fully rendered
      const timeoutId = setTimeout(() => {
        inputRef.current?.focus();
      }, 0);
      return () => clearTimeout(timeoutId);
    } else {
      // Restore focus to previously focused element when closing
      if (previousActiveElement.current) {
        previousActiveElement.current.focus();
        previousActiveElement.current = null;
      }
    }
  }, [isCommandPaletteOpen]);

  // Reset selected index when filtered commands change
  useEffect(() => {
    setSelectedIndex(0);
  }, [filteredCommands.length]);

  // Scroll selected item into view
  useEffect(() => {
    if (listRef.current && filteredCommands.length > 0) {
      const selectedItem = listRef.current.children[selectedIndex] as HTMLElement;
      if (selectedItem?.scrollIntoView) {
        selectedItem.scrollIntoView({ block: "nearest" });
      }
    }
  }, [selectedIndex, filteredCommands.length]);

  const executeCommand = useCallback(
    (command: Command) => {
      setSearchQuery("");
      setCommandPaletteOpen(false);
      if (command.action) {
        command.action();
      } else if (command.path) {
        navigate(command.path);
      }
    },
    [navigate, setCommandPaletteOpen]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      switch (e.key) {
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex((prev) =>
            prev < filteredCommands.length - 1 ? prev + 1 : prev
          );
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex((prev) => (prev > 0 ? prev - 1 : prev));
          break;
        case "Enter":
          e.preventDefault();
          if (filteredCommands[selectedIndex]) {
            executeCommand(filteredCommands[selectedIndex]);
          }
          break;
        case "Escape":
          e.preventDefault();
          e.stopPropagation();
          setCommandPaletteOpen(false);
          break;
        case "Tab":
          // Trap focus within the dialog
          e.preventDefault();
          break;
      }
    },
    [filteredCommands, selectedIndex, executeCommand, setCommandPaletteOpen]
  );

  const handleOverlayClick = useCallback(() => {
    setCommandPaletteOpen(false);
  }, [setCommandPaletteOpen]);

  const handleContentClick = useCallback((e: React.MouseEvent) => {
    e.stopPropagation();
  }, []);

  if (!isCommandPaletteOpen) {
    return null;
  }

  const listboxId = "command-palette-listbox";
  const selectedCommandId = filteredCommands[selectedIndex]
    ? `command-${filteredCommands[selectedIndex].id}`
    : undefined;

  return (
    <div
      className="fixed inset-0 z-50 bg-black/80"
      onClick={handleOverlayClick}
      onKeyDown={handleKeyDown}
    >
      <div
        className="fixed left-[50%] top-[20%] w-full max-w-lg translate-x-[-50%] rounded-lg border border-[hsl(var(--border))] bg-[hsl(var(--background))] shadow-lg"
        onClick={handleContentClick}
        role="dialog"
        aria-modal="true"
        aria-labelledby="command-palette-title"
      >
        {/* Visually hidden heading for screen readers */}
        <h2 id="command-palette-title" className="sr-only">
          Command Palette
        </h2>

        {/* Search Input */}
        <div className="flex items-center border-b border-[hsl(var(--border))] px-3">
          <SearchIcon className="mr-2 h-4 w-4 shrink-0 opacity-50" />
          <input
            ref={inputRef}
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder="Type a command or search..."
            className="flex h-11 w-full rounded-md bg-transparent py-3 text-sm outline-none placeholder:text-[hsl(var(--muted-foreground))] disabled:cursor-not-allowed disabled:opacity-50"
            role="combobox"
            aria-autocomplete="list"
            aria-controls={listboxId}
            aria-expanded="true"
            aria-activedescendant={selectedCommandId}
          />
        </div>

        {/* Command List */}
        <div
          ref={listRef}
          id={listboxId}
          className="max-h-[300px] overflow-y-auto p-1"
          role="listbox"
        >
          {filteredCommands.length === 0 ? (
            <div className="py-6 text-center text-sm text-[hsl(var(--muted-foreground))]">
              No commands found.
            </div>
          ) : (
            filteredCommands.map((command, index) => (
              <button
                key={command.id}
                id={`command-${command.id}`}
                type="button"
                role="option"
                aria-selected={index === selectedIndex}
                onClick={() => executeCommand(command)}
                onMouseEnter={() => setSelectedIndex(index)}
                className={cn(
                  "relative flex w-full cursor-pointer select-none items-center rounded-sm px-2 py-1.5 text-sm outline-none",
                  index === selectedIndex
                    ? "bg-[hsl(var(--accent))] text-[hsl(var(--accent-foreground))]"
                    : "text-[hsl(var(--foreground))]"
                )}
              >
                <span className="mr-2 flex h-5 w-5 items-center justify-center">
                  {command.icon}
                </span>
                <span>{command.label}</span>
              </button>
            ))
          )}
        </div>

        {/* Footer with keyboard hints */}
        <div className="flex items-center justify-between border-t border-[hsl(var(--border))] px-3 py-2 text-xs text-[hsl(var(--muted-foreground))]">
          <div className="flex items-center gap-2">
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-[hsl(var(--border))] bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-xs">
                ↑↓
              </kbd>
              <span>Navigate</span>
            </span>
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-[hsl(var(--border))] bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-xs">
                ↵
              </kbd>
              <span>Select</span>
            </span>
            <span className="flex items-center gap-1">
              <kbd className="rounded border border-[hsl(var(--border))] bg-[hsl(var(--muted))] px-1.5 py-0.5 font-mono text-xs">
                Esc
              </kbd>
              <span>Close</span>
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

