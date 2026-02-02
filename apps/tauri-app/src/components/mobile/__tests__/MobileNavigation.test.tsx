import { render, screen } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { describe, it, expect } from "vitest";
import { MobileNavigation } from "../MobileNavigation";

describe("MobileNavigation", () => {
  const renderWithRouter = (component: React.ReactElement) => {
    return render(<BrowserRouter>{component}</BrowserRouter>);
  };

  it("renders all navigation items", () => {
    renderWithRouter(<MobileNavigation />);

    expect(screen.getByText("Home")).toBeInTheDocument();
    expect(screen.getByText("New Task")).toBeInTheDocument();
    expect(screen.getByText("Repos")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });

  it("has correct navigation links", () => {
    renderWithRouter(<MobileNavigation />);

    const homeLink = screen.getByText("Home").closest("a");
    const newTaskLink = screen.getByText("New Task").closest("a");
    const reposLink = screen.getByText("Repos").closest("a");
    const settingsLink = screen.getByText("Settings").closest("a");

    expect(homeLink).toHaveAttribute("href", "/");
    expect(newTaskLink).toHaveAttribute("href", "/tasks/new");
    expect(reposLink).toHaveAttribute("href", "/repositories");
    expect(settingsLink).toHaveAttribute("href", "/settings");
  });

  it("has proper accessibility attributes", () => {
    renderWithRouter(<MobileNavigation />);

    const nav = screen.getByRole("navigation");
    expect(nav).toHaveAttribute("aria-label", "Main navigation");
  });

  it("applies touch-manipulation class for touch optimization", () => {
    renderWithRouter(<MobileNavigation />);

    const links = screen.getAllByRole("link");
    links.forEach((link) => {
      expect(link).toHaveClass("touch-manipulation");
    });
  });
});
