import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { MobileHeader } from "../MobileHeader";

// Mock react-router-dom useNavigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe("MobileHeader", () => {
  const renderWithRouter = (component: React.ReactElement) => {
    return render(<BrowserRouter>{component}</BrowserRouter>);
  };

  beforeEach(() => {
    mockNavigate.mockClear();
  });

  it("renders title", () => {
    renderWithRouter(<MobileHeader title="Test Title" />);

    expect(screen.getByText("Test Title")).toBeInTheDocument();
  });

  it("shows back button when showBack is true", () => {
    renderWithRouter(<MobileHeader title="Test" showBack />);

    expect(screen.getByLabelText("Go back")).toBeInTheDocument();
  });

  it("hides back button when showBack is false", () => {
    renderWithRouter(<MobileHeader title="Test" />);

    expect(screen.queryByLabelText("Go back")).not.toBeInTheDocument();
  });

  it("navigates back when back button is clicked", () => {
    renderWithRouter(<MobileHeader title="Test" showBack />);

    fireEvent.click(screen.getByLabelText("Go back"));

    expect(mockNavigate).toHaveBeenCalledWith(-1);
  });

  it("renders right action when provided", () => {
    renderWithRouter(
      <MobileHeader
        title="Test"
        rightAction={<button>Action</button>}
      />
    );

    expect(screen.getByText("Action")).toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = renderWithRouter(
      <MobileHeader title="Test" className="custom-class" />
    );

    expect(container.querySelector(".custom-class")).toBeInTheDocument();
  });
});
