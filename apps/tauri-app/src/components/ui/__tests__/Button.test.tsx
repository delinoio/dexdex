import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { Button } from "../Button";

describe("Button", () => {
  it("renders children correctly", () => {
    render(<Button>Click me</Button>);
    expect(screen.getByRole("button", { name: /click me/i })).toBeInTheDocument();
  });

  it("handles click events", () => {
    const handleClick = vi.fn();
    render(<Button onClick={handleClick}>Click me</Button>);

    fireEvent.click(screen.getByRole("button"));
    expect(handleClick).toHaveBeenCalledTimes(1);
  });

  it("can be disabled", () => {
    const handleClick = vi.fn();
    render(<Button disabled onClick={handleClick}>Click me</Button>);

    const button = screen.getByRole("button");
    expect(button).toBeDisabled();

    fireEvent.click(button);
    expect(handleClick).not.toHaveBeenCalled();
  });

  it("applies variant classes correctly", () => {
    const { rerender } = render(<Button variant="default">Default</Button>);
    expect(screen.getByRole("button")).toHaveClass("bg-[hsl(var(--primary))]");

    rerender(<Button variant="destructive">Destructive</Button>);
    expect(screen.getByRole("button")).toHaveClass("bg-[hsl(var(--destructive))]");

    rerender(<Button variant="outline">Outline</Button>);
    expect(screen.getByRole("button")).toHaveClass("border");

    rerender(<Button variant="secondary">Secondary</Button>);
    expect(screen.getByRole("button")).toHaveClass("bg-[hsl(var(--secondary))]");

    rerender(<Button variant="ghost">Ghost</Button>);
    expect(screen.getByRole("button")).toHaveClass("hover:bg-[hsl(var(--accent))]");

    rerender(<Button variant="link">Link</Button>);
    expect(screen.getByRole("button")).toHaveClass("text-[hsl(var(--primary))]");
  });

  it("applies size classes correctly", () => {
    const { rerender } = render(<Button size="default">Default</Button>);
    expect(screen.getByRole("button")).toHaveClass("h-9");

    rerender(<Button size="sm">Small</Button>);
    expect(screen.getByRole("button")).toHaveClass("h-8");

    rerender(<Button size="lg">Large</Button>);
    expect(screen.getByRole("button")).toHaveClass("h-10");

    rerender(<Button size="icon">Icon</Button>);
    expect(screen.getByRole("button")).toHaveClass("h-9", "w-9");
  });

  it("forwards ref correctly", () => {
    const ref = vi.fn();
    render(<Button ref={ref}>Button with ref</Button>);
    expect(ref).toHaveBeenCalled();
  });

  it("accepts additional className", () => {
    render(<Button className="custom-class">Custom</Button>);
    expect(screen.getByRole("button")).toHaveClass("custom-class");
  });

  it("has proper focus styles", () => {
    render(<Button>Focusable</Button>);
    const button = screen.getByRole("button");
    expect(button).toHaveClass("focus-visible:outline-none", "focus-visible:ring-1");
  });
});
