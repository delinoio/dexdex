import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { Card, CardHeader, CardTitle, CardDescription, CardContent, CardFooter } from "../Card";

describe("Card", () => {
  it("renders Card with children", () => {
    render(<Card>Card content</Card>);
    expect(screen.getByText("Card content")).toBeInTheDocument();
  });

  it("renders Card with all subcomponents", () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Title</CardTitle>
          <CardDescription>Description</CardDescription>
        </CardHeader>
        <CardContent>Main content</CardContent>
        <CardFooter>Footer content</CardFooter>
      </Card>
    );

    expect(screen.getByText("Title")).toBeInTheDocument();
    expect(screen.getByText("Description")).toBeInTheDocument();
    expect(screen.getByText("Main content")).toBeInTheDocument();
    expect(screen.getByText("Footer content")).toBeInTheDocument();
  });

  it("applies custom className to Card", () => {
    render(<Card className="custom-card" data-testid="test-card">Content</Card>);
    expect(screen.getByTestId("test-card")).toHaveClass("custom-card");
  });

  it("CardHeader has correct styling", () => {
    render(
      <Card>
        <CardHeader data-testid="header">Header</CardHeader>
      </Card>
    );
    expect(screen.getByTestId("header")).toHaveClass("flex", "flex-col", "space-y-1.5", "p-6");
  });

  it("CardTitle renders as h3 with correct styling", () => {
    render(
      <Card>
        <CardHeader>
          <CardTitle>Test Title</CardTitle>
        </CardHeader>
      </Card>
    );
    const title = screen.getByText("Test Title");
    expect(title.tagName).toBe("H3");
    expect(title).toHaveClass("font-semibold");
  });

  it("CardDescription renders as paragraph with muted styling", () => {
    render(
      <Card>
        <CardHeader>
          <CardDescription>Test Description</CardDescription>
        </CardHeader>
      </Card>
    );
    const desc = screen.getByText("Test Description");
    expect(desc.tagName).toBe("P");
    expect(desc).toHaveClass("text-sm", "text-[hsl(var(--muted-foreground))]");
  });

  it("CardContent has correct padding", () => {
    render(
      <Card>
        <CardContent data-testid="content">Content</CardContent>
      </Card>
    );
    expect(screen.getByTestId("content")).toHaveClass("p-6", "pt-0");
  });

  it("CardFooter has correct layout and padding", () => {
    render(
      <Card>
        <CardFooter data-testid="footer">Footer</CardFooter>
      </Card>
    );
    expect(screen.getByTestId("footer")).toHaveClass("flex", "items-center", "p-6", "pt-0");
  });

  it("forwards refs correctly", () => {
    const { container } = render(<Card ref={() => {}}>Content</Card>);
    expect(container.firstChild).toBeInTheDocument();
  });
});
