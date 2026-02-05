// Keyboard shortcuts for review and task detail pages
import { useEffect, useCallback } from "react";
import { getEffectiveKey } from "@/lib/keyboardUtils";

// Detect platform once at module level
const isMac =
  typeof navigator !== "undefined" &&
  navigator.platform.toUpperCase().indexOf("MAC") >= 0;

interface UseTaskDetailShortcutsOptions {
  /** Callback when 'A' is pressed to approve */
  onApprove?: () => void;
  /** Callback when 'D' is pressed to deny/reject */
  onDeny?: () => void;
  /** Callback when 'L' is pressed to toggle log visibility */
  onToggleLog?: () => void;
  /** Callback when 'S' is pressed to stop execution */
  onStop?: () => void;
  /** Whether shortcuts are enabled (default: true) */
  enabled?: boolean;
}

interface UseReviewShortcutsOptions {
  /** Callback when 'J' is pressed to navigate to next file */
  onNextFile?: () => void;
  /** Callback when 'K' is pressed to navigate to previous file */
  onPrevFile?: () => void;
  /** Callback when 'Enter' is pressed to open file */
  onOpenFile?: () => void;
  /** Callback when 'Cmd/Ctrl+Enter' is pressed to approve */
  onApprove?: () => void;
  /** Whether shortcuts are enabled (default: true) */
  enabled?: boolean;
}

/**
 * Hook for task detail page keyboard shortcuts
 *
 * Shortcuts:
 * - A: Approve
 * - D: Deny/Reject
 * - L: Toggle Log visibility
 * - S: Stop Execution
 */
export function useTaskDetailShortcuts(options: UseTaskDetailShortcutsOptions) {
  const { onApprove, onDeny, onToggleLog, onStop, enabled = true } = options;

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      // Don't trigger shortcuts when typing in inputs
      const target = event.target as HTMLElement;
      if (
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable
      ) {
        return;
      }

      // Skip if any modifier keys are pressed (these shortcuts are plain keys)
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      // Use event.key first, then fall back to physical key code
      // for keyboard layout independence (e.g., Korean, Russian layouts)
      const key = getEffectiveKey(event);

      switch (key) {
        case "a":
          if (onApprove) {
            event.preventDefault();
            onApprove();
          }
          break;
        case "d":
          if (onDeny) {
            event.preventDefault();
            onDeny();
          }
          break;
        case "l":
          if (onToggleLog) {
            event.preventDefault();
            onToggleLog();
          }
          break;
        case "s":
          if (onStop) {
            event.preventDefault();
            onStop();
          }
          break;
      }
    },
    [enabled, onApprove, onDeny, onToggleLog, onStop]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}

/**
 * Hook for review interface keyboard shortcuts
 *
 * Shortcuts:
 * - J: Navigate to next file
 * - K: Navigate to previous file
 * - Enter: Open file
 * - Cmd/Ctrl+Enter: Approve
 */
export function useReviewShortcuts(options: UseReviewShortcutsOptions) {
  const {
    onNextFile,
    onPrevFile,
    onOpenFile,
    onApprove,
    enabled = true,
  } = options;

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (!enabled) return;

      // Don't trigger shortcuts when typing in inputs (except for Cmd/Ctrl+Enter)
      const target = event.target as HTMLElement;
      const isInputElement =
        target.tagName === "INPUT" ||
        target.tagName === "TEXTAREA" ||
        target.isContentEditable;

      const modKey = isMac ? event.metaKey : event.ctrlKey;
      // Use event.key first, then fall back to physical key code
      // for keyboard layout independence (e.g., Korean, Russian layouts)
      const key = getEffectiveKey(event);

      // Cmd/Ctrl+Enter for approve (works even in inputs)
      if (key === "enter" && modKey && !event.altKey && !event.shiftKey) {
        if (onApprove) {
          event.preventDefault();
          onApprove();
        }
        return;
      }

      // Skip remaining shortcuts if in input
      if (isInputElement) {
        return;
      }

      // Plain key shortcuts (no modifiers)
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }

      switch (key) {
        case "j":
          if (onNextFile) {
            event.preventDefault();
            onNextFile();
          }
          break;
        case "k":
          if (onPrevFile) {
            event.preventDefault();
            onPrevFile();
          }
          break;
        case "enter":
          if (onOpenFile) {
            event.preventDefault();
            onOpenFile();
          }
          break;
      }
    },
    [enabled, onNextFile, onPrevFile, onOpenFile, onApprove]
  );

  useEffect(() => {
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [handleKeyDown]);
}
