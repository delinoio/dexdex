import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { CommandPalette } from "../CommandPalette";
import { useUiStore } from "@/stores/uiStore";
import { useChatStore } from "@/stores/chatStore";

// Mock react-router-dom
const mockNavigate = vi.fn();
vi.mock("react-router-dom", () => ({
  useNavigate: () => mockNavigate,
}));

// Mock zustand stores
vi.mock("@/stores/uiStore", () => ({
  useUiStore: vi.fn(),
}));

const mockSetChatOpen = vi.fn();
vi.mock("@/stores/chatStore", () => ({
  useChatStore: vi.fn(() => ({
    setOpen: mockSetChatOpen,
  })),
}));

describe("CommandPalette", () => {
  const mockSetCommandPaletteOpen = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
    (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      isCommandPaletteOpen: true,
      setCommandPaletteOpen: mockSetCommandPaletteOpen,
    });
    (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
      setOpen: mockSetChatOpen,
    });
  });

  describe("Rendering", () => {
    it("renders when isCommandPaletteOpen is true", () => {
      render(<CommandPalette />);
      expect(screen.getByRole("dialog")).toBeInTheDocument();
    });

    it("does not render when isCommandPaletteOpen is false", () => {
      (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
        isCommandPaletteOpen: false,
        setCommandPaletteOpen: mockSetCommandPaletteOpen,
      });
      render(<CommandPalette />);
      expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
    });

    it("renders all default commands", () => {
      render(<CommandPalette />);
      expect(screen.getByText("New Task")).toBeInTheDocument();
      expect(screen.getByText("Dashboard")).toBeInTheDocument();
      expect(screen.getByText("Settings")).toBeInTheDocument();
      expect(screen.getByText("Repositories")).toBeInTheDocument();
      expect(screen.getByText("Repository Groups")).toBeInTheDocument();
      expect(screen.getByText("Open Chat")).toBeInTheDocument();
    });

    it("renders search input with placeholder", () => {
      render(<CommandPalette />);
      expect(
        screen.getByPlaceholderText("Type a command or search...")
      ).toBeInTheDocument();
    });

    it("renders keyboard hints in footer", () => {
      render(<CommandPalette />);
      expect(screen.getByText("Navigate")).toBeInTheDocument();
      expect(screen.getByText("Select")).toBeInTheDocument();
      expect(screen.getByText("Close")).toBeInTheDocument();
    });
  });

  describe("Accessibility", () => {
    it("has proper ARIA attributes on dialog", () => {
      render(<CommandPalette />);
      const dialog = screen.getByRole("dialog");
      expect(dialog).toHaveAttribute("aria-modal", "true");
      expect(dialog).toHaveAttribute("aria-labelledby", "command-palette-title");
    });

    it("has visually hidden heading for screen readers", () => {
      render(<CommandPalette />);
      const heading = screen.getByText("Command Palette");
      expect(heading).toHaveClass("sr-only");
    });

    it("has proper combobox ARIA attributes on input", () => {
      render(<CommandPalette />);
      const input = screen.getByRole("combobox");
      expect(input).toHaveAttribute("aria-autocomplete", "list");
      expect(input).toHaveAttribute("aria-controls", "command-palette-listbox");
      expect(input).toHaveAttribute("aria-expanded", "true");
    });

    it("has proper listbox role on command list", () => {
      render(<CommandPalette />);
      expect(screen.getByRole("listbox")).toBeInTheDocument();
    });

    it("command options have proper option role and aria-selected", () => {
      render(<CommandPalette />);
      const options = screen.getAllByRole("option");
      expect(options.length).toBe(6);
      expect(options[0]).toHaveAttribute("aria-selected", "true");
      expect(options[1]).toHaveAttribute("aria-selected", "false");
    });
  });

  describe("Filtering", () => {
    it("filters commands by label", () => {
      render(<CommandPalette />);

      const input = screen.getByRole("combobox");
      fireEvent.change(input, { target: { value: "dash" } });

      expect(screen.getByText("Dashboard")).toBeInTheDocument();
      expect(screen.queryByText("New Task")).not.toBeInTheDocument();
      expect(screen.queryByText("Settings")).not.toBeInTheDocument();
    });

    it("filters commands by keyword", () => {
      render(<CommandPalette />);

      const input = screen.getByRole("combobox");
      fireEvent.change(input, { target: { value: "git" } });

      expect(screen.getByText("Repositories")).toBeInTheDocument();
      expect(screen.queryByText("Dashboard")).not.toBeInTheDocument();
    });

    it("filters chat command by keyword", () => {
      render(<CommandPalette />);

      const input = screen.getByRole("combobox");
      fireEvent.change(input, { target: { value: "chat" } });

      expect(screen.getByText("Open Chat")).toBeInTheDocument();
      expect(screen.queryByText("Dashboard")).not.toBeInTheDocument();
    });

    it("shows 'No commands found' when no matches", () => {
      render(<CommandPalette />);

      const input = screen.getByRole("combobox");
      fireEvent.change(input, { target: { value: "xyz123nonexistent" } });

      expect(screen.getByText("No commands found.")).toBeInTheDocument();
    });

    it("resets filter when search is cleared", () => {
      render(<CommandPalette />);

      const input = screen.getByRole("combobox");
      fireEvent.change(input, { target: { value: "dash" } });
      expect(screen.queryByText("Settings")).not.toBeInTheDocument();

      fireEvent.change(input, { target: { value: "" } });
      expect(screen.getByText("Settings")).toBeInTheDocument();
    });
  });

  describe("Keyboard Navigation", () => {
    it("navigates down with ArrowDown", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      fireEvent.keyDown(overlay, { key: "ArrowDown" });

      const options = screen.getAllByRole("option");
      expect(options[1]).toHaveAttribute("aria-selected", "true");
    });

    it("navigates up with ArrowUp", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      fireEvent.keyDown(overlay, { key: "ArrowDown" });
      fireEvent.keyDown(overlay, { key: "ArrowDown" });
      fireEvent.keyDown(overlay, { key: "ArrowUp" });

      const options = screen.getAllByRole("option");
      expect(options[1]).toHaveAttribute("aria-selected", "true");
    });

    it("does not go above first item", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      fireEvent.keyDown(overlay, { key: "ArrowUp" });

      const options = screen.getAllByRole("option");
      expect(options[0]).toHaveAttribute("aria-selected", "true");
    });

    it("does not go below last item", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      for (let i = 0; i < 10; i++) {
        fireEvent.keyDown(overlay, { key: "ArrowDown" });
      }

      const options = screen.getAllByRole("option");
      expect(options[5]).toHaveAttribute("aria-selected", "true");
    });

    it("selects command with Enter", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      fireEvent.keyDown(overlay, { key: "Enter" });

      expect(mockNavigate).toHaveBeenCalledWith("/tasks/new");
      expect(mockSetCommandPaletteOpen).toHaveBeenCalledWith(false);
    });

    it("closes with Escape", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      fireEvent.keyDown(overlay, { key: "Escape" });

      expect(mockSetCommandPaletteOpen).toHaveBeenCalledWith(false);
    });

    it("traps Tab key within dialog", () => {
      render(<CommandPalette />);

      const overlay = screen.getByRole("dialog").parentElement!;
      const input = screen.getByRole("combobox");
      input.focus();

      const tabEvent = new KeyboardEvent("keydown", {
        key: "Tab",
        bubbles: true,
        cancelable: true,
      });
      const preventDefaultSpy = vi.spyOn(tabEvent, "preventDefault");
      overlay.dispatchEvent(tabEvent);

      expect(preventDefaultSpy).toHaveBeenCalled();
    });
  });

  describe("Command Execution", () => {
    it("executes command on click", () => {
      render(<CommandPalette />);

      fireEvent.click(screen.getByText("Dashboard"));

      expect(mockNavigate).toHaveBeenCalledWith("/");
      expect(mockSetCommandPaletteOpen).toHaveBeenCalledWith(false);
    });

    it("navigates to correct path for each command", () => {
      const testCases = [
        { label: "New Task", path: "/tasks/new" },
        { label: "Dashboard", path: "/" },
        { label: "Settings", path: "/settings" },
        { label: "Repositories", path: "/repositories" },
        { label: "Repository Groups", path: "/repository-groups" },
      ];

      for (const { label, path } of testCases) {
        vi.clearAllMocks();
        (useUiStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
          isCommandPaletteOpen: true,
          setCommandPaletteOpen: mockSetCommandPaletteOpen,
        });
        (useChatStore as unknown as ReturnType<typeof vi.fn>).mockReturnValue({
          setOpen: mockSetChatOpen,
        });
        const { unmount } = render(<CommandPalette />);
        fireEvent.click(screen.getByText(label));
        expect(mockNavigate).toHaveBeenCalledWith(path);
        unmount();
      }
    });

    it("opens chat when Open Chat command is clicked", () => {
      render(<CommandPalette />);

      fireEvent.click(screen.getByText("Open Chat"));

      expect(mockSetChatOpen).toHaveBeenCalledWith(true);
      expect(mockSetCommandPaletteOpen).toHaveBeenCalledWith(false);
      expect(mockNavigate).not.toHaveBeenCalled();
    });
  });

  describe("Mouse Interaction", () => {
    it("selects command on mouse hover", () => {
      render(<CommandPalette />);

      const options = screen.getAllByRole("option");
      fireEvent.mouseEnter(options[2]);

      expect(options[2]).toHaveAttribute("aria-selected", "true");
    });

    it("closes on overlay click", () => {
      render(<CommandPalette />);

      // Click on overlay (outside dialog content)
      const overlay = screen.getByRole("dialog").parentElement;
      if (overlay) {
        fireEvent.click(overlay);
        expect(mockSetCommandPaletteOpen).toHaveBeenCalledWith(false);
      }
    });

    it("does not close on dialog content click", () => {
      render(<CommandPalette />);

      const dialog = screen.getByRole("dialog");
      fireEvent.click(dialog);

      // Should not close when clicking inside dialog
      expect(mockSetCommandPaletteOpen).not.toHaveBeenCalledWith(false);
    });
  });

  describe("Focus Management", () => {
    it("focuses input when opened", async () => {
      render(<CommandPalette />);

      await waitFor(() => {
        expect(screen.getByRole("combobox")).toHaveFocus();
      });
    });
  });
});
