import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import {
  PlusIcon,
  UnitTaskIcon,
  CompositeTaskIcon,
  CloseIcon,
  RefreshIcon,
  AlertCircleIcon,
  HomeIcon,
  SettingsIcon,
  FolderIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  ChevronLeftIcon,
  CheckIcon,
  MenuIcon,
  LoaderIcon,
  TrashIcon,
  EditIcon,
  WorkspaceIcon,
} from "../Icons";

describe("Icons", () => {
  const icons = [
    { name: "PlusIcon", Component: PlusIcon },
    { name: "UnitTaskIcon", Component: UnitTaskIcon },
    { name: "CompositeTaskIcon", Component: CompositeTaskIcon },
    { name: "CloseIcon", Component: CloseIcon },
    { name: "RefreshIcon", Component: RefreshIcon },
    { name: "AlertCircleIcon", Component: AlertCircleIcon },
    { name: "HomeIcon", Component: HomeIcon },
    { name: "SettingsIcon", Component: SettingsIcon },
    { name: "FolderIcon", Component: FolderIcon },
    { name: "ChevronDownIcon", Component: ChevronDownIcon },
    { name: "ChevronRightIcon", Component: ChevronRightIcon },
    { name: "ChevronLeftIcon", Component: ChevronLeftIcon },
    { name: "CheckIcon", Component: CheckIcon },
    { name: "MenuIcon", Component: MenuIcon },
    { name: "LoaderIcon", Component: LoaderIcon },
    { name: "TrashIcon", Component: TrashIcon },
    { name: "EditIcon", Component: EditIcon },
    { name: "WorkspaceIcon", Component: WorkspaceIcon },
  ];

  icons.forEach(({ name, Component }) => {
    describe(name, () => {
      it("renders SVG element", () => {
        const { container } = render(<Component data-testid={name} />);
        const svg = container.querySelector("svg");
        expect(svg).toBeInTheDocument();
      });

      it("applies default size of 24", () => {
        const { container } = render(<Component />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveAttribute("width", "24");
        expect(svg).toHaveAttribute("height", "24");
      });

      it("accepts custom size prop", () => {
        const { container } = render(<Component size={16} />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveAttribute("width", "16");
        expect(svg).toHaveAttribute("height", "16");
      });

      it("accepts custom className", () => {
        const { container } = render(<Component className="custom-class" />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveClass("custom-class");
      });

      it("has aria-hidden attribute for accessibility", () => {
        const { container } = render(<Component />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveAttribute("aria-hidden", "true");
      });

      it("has shrink-0 class to prevent shrinking", () => {
        const { container } = render(<Component />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveClass("shrink-0");
      });

      it("uses currentColor for stroke", () => {
        const { container } = render(<Component />);
        const svg = container.querySelector("svg");
        expect(svg).toHaveAttribute("stroke", "currentColor");
      });
    });
  });
});
