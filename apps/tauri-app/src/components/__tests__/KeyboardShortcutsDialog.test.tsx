// KeyboardShortcutsDialog tests
import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { KeyboardShortcutsDialog } from "../KeyboardShortcutsDialog";
import { useUiStore } from "@/stores/uiStore";

// Mock the store
vi.mock("@/stores/uiStore", () => ({
  useUiStore: vi.fn(),
}));

describe("KeyboardShortcutsDialog", () => {
  const mockSetKeyboardShortcutsOpen = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      isKeyboardShortcutsOpen: false,
      setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
    });
  });

  describe("Rendering", () => {
    it("does not render when closed", () => {
      render(<KeyboardShortcutsDialog />);
      expect(screen.queryByText("Keyboard Shortcuts")).not.toBeInTheDocument();
    });

    it("renders when open", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);
      expect(screen.getByText("Keyboard Shortcuts")).toBeInTheDocument();
    });

    it("displays all shortcut sections", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);

      expect(screen.getByText("Global")).toBeInTheDocument();
      expect(screen.getByText("Tab Navigation")).toBeInTheDocument();
      expect(screen.getByText("Review Interface")).toBeInTheDocument();
      expect(screen.getByText("Task Detail")).toBeInTheDocument();
    });

    it("displays the new shortcuts", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);

      expect(screen.getByText("Show Keyboard Shortcuts")).toBeInTheDocument();
      expect(screen.getByText("Create Task")).toBeInTheDocument();
    });

    it("displays description text", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);

      expect(
        screen.getByText(
          "Available keyboard shortcuts for navigating the application."
        )
      ).toBeInTheDocument();
    });
  });

  describe("Interaction", () => {
    it("closes when escape is pressed", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);

      const dialog = screen.getByRole("dialog");
      fireEvent.keyDown(dialog, { key: "Escape" });
      expect(mockSetKeyboardShortcutsOpen).toHaveBeenCalledWith(false);
    });

    it("closes when close button is clicked", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);

      const closeButton = screen.getByLabelText("Close dialog");
      fireEvent.click(closeButton);
      expect(mockSetKeyboardShortcutsOpen).toHaveBeenCalledWith(false);
    });
  });

  describe("Accessibility", () => {
    it("has proper dialog role", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    it("has aria-modal attribute", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isKeyboardShortcutsOpen: true,
        setKeyboardShortcutsOpen: mockSetKeyboardShortcutsOpen,
      });

      render(<KeyboardShortcutsDialog />);
      expect(screen.getByRole("dialog")).toHaveAttribute("aria-modal", "true");
    });
  });
});
