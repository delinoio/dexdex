import { describe, it, expect, beforeEach, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { BrowserRouter } from "react-router-dom";
import { TabBar } from "../TabBar";
import { useUiStore } from "@/stores/uiStore";

// Mock useNavigate
const mockNavigate = vi.fn();
vi.mock("react-router-dom", async () => {
  const actual = await vi.importActual("react-router-dom");
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  };
});

describe("TabBar", () => {
  beforeEach(() => {
    // Reset store state before each test
    useUiStore.setState({
      tabs: [
        { id: "dashboard", title: "Dashboard", path: "/", closable: false },
      ],
      activeTabId: "dashboard",
    });
    mockNavigate.mockClear();
  });

  const renderTabBar = () => {
    return render(
      <BrowserRouter>
        <TabBar />
      </BrowserRouter>
    );
  };

  it("does not render when only one tab exists", () => {
    renderTabBar();
    expect(screen.queryByRole("tab")).toBeNull();
  });

  it("renders when multiple tabs exist", () => {
    useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    renderTabBar();
    expect(screen.getByText("Dashboard")).toBeInTheDocument();
    expect(screen.getByText("Task Details")).toBeInTheDocument();
  });

  it("switches to a tab when clicked", () => {
    useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    renderTabBar();

    // Click on Dashboard tab
    fireEvent.click(screen.getByText("Dashboard"));

    expect(useUiStore.getState().activeTabId).toBe("dashboard");
    expect(mockNavigate).toHaveBeenCalledWith("/");
  });

  it("shows close button for closable tabs", () => {
    useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    renderTabBar();

    // Close button should exist for the closable tab
    expect(screen.getByLabelText("Close Task Details")).toBeInTheDocument();
  });

  it("does not show close button for non-closable tabs", () => {
    useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    renderTabBar();

    // Close button should not exist for Dashboard
    expect(screen.queryByLabelText("Close Dashboard")).toBeNull();
  });

  it("removes tab when close button is clicked", () => {
    const tabId = useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    // Make dashboard active first
    useUiStore.getState().setActiveTab("dashboard");

    renderTabBar();

    // Click close button on the closable tab
    fireEvent.click(screen.getByLabelText("Close Task Details"));

    // Tab should be removed
    expect(
      useUiStore.getState().tabs.find((t) => t.id === tabId)
    ).toBeUndefined();
  });

  it("highlights active tab", () => {
    const tabId = useUiStore.getState().addTab({
      title: "Task Details",
      path: "/unit-tasks/123",
      closable: true,
    });

    // The new tab is active
    expect(useUiStore.getState().activeTabId).toBe(tabId);

    renderTabBar();

    // Find the task details tab (uses role="tab" instead of button)
    const taskTab = screen.getByText("Task Details").closest('[role="tab"]');
    expect(taskTab).toHaveClass("bg-[hsl(var(--muted))]");
  });
});
