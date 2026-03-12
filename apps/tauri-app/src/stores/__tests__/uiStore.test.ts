import { describe, it, expect, beforeEach } from "vitest";
import { useUiStore } from "../uiStore";

describe("uiStore", () => {
  beforeEach(() => {
    // Reset store state before each test
    useUiStore.setState({
      sidebarCollapsed: false,
      currentWorkspaceId: null,
      tabs: [{ id: "home", type: "workspace", title: "Tasks", path: "/", closable: false }],
      activeTabId: "home",
      isCommandPaletteOpen: false,
      isTaskCreationOpen: false,
      isSettingsOpen: false,
    });
  });

  describe("sidebar", () => {
    it("toggles sidebar collapsed state", () => {
      expect(useUiStore.getState().sidebarCollapsed).toBe(false);

      useUiStore.getState().toggleSidebar();
      expect(useUiStore.getState().sidebarCollapsed).toBe(true);

      useUiStore.getState().toggleSidebar();
      expect(useUiStore.getState().sidebarCollapsed).toBe(false);
    });

    it("sets sidebar collapsed state directly", () => {
      useUiStore.getState().setSidebarCollapsed(true);
      expect(useUiStore.getState().sidebarCollapsed).toBe(true);

      useUiStore.getState().setSidebarCollapsed(false);
      expect(useUiStore.getState().sidebarCollapsed).toBe(false);
    });
  });

  describe("workspace", () => {
    it("sets current workspace ID", () => {
      expect(useUiStore.getState().currentWorkspaceId).toBeNull();

      useUiStore.getState().setCurrentWorkspaceId("workspace-123");
      expect(useUiStore.getState().currentWorkspaceId).toBe("workspace-123");
    });

    it("can clear current workspace ID", () => {
      useUiStore.getState().setCurrentWorkspaceId("workspace-123");
      useUiStore.getState().setCurrentWorkspaceId(null);
      expect(useUiStore.getState().currentWorkspaceId).toBeNull();
    });
  });

  describe("tabs", () => {
    it("adds a new tab and makes it active", () => {
      const newTabId = useUiStore.getState().addTab({
        type: "task",
        title: "New Tab",
        path: "/new",
        closable: true,
      });

      const state = useUiStore.getState();
      expect(state.tabs.length).toBe(2);
      expect(state.activeTabId).toBe(newTabId);
      expect(state.tabs.find((t) => t.id === newTabId)?.title).toBe("New Tab");
    });

    it("removes a closable tab", () => {
      const newTabId = useUiStore.getState().addTab({
        type: "task",
        title: "Closable Tab",
        path: "/closable",
        closable: true,
      });

      useUiStore.getState().removeTab(newTabId);

      const state = useUiStore.getState();
      expect(state.tabs.find((t) => t.id === newTabId)).toBeUndefined();
    });

    it("does not remove a non-closable tab", () => {
      useUiStore.getState().removeTab("home");
      expect(useUiStore.getState().tabs.find((t) => t.id === "home")).toBeDefined();
    });

    it("selects previous tab when active tab is removed", () => {
      // Add two tabs
      const tab1Id = useUiStore.getState().addTab({
        type: "task",
        title: "Tab 1",
        path: "/tab1",
        closable: true,
      });
      const tab2Id = useUiStore.getState().addTab({
        type: "task",
        title: "Tab 2",
        path: "/tab2",
        closable: true,
      });

      // Tab 2 is now active
      expect(useUiStore.getState().activeTabId).toBe(tab2Id);

      // Remove Tab 2
      useUiStore.getState().removeTab(tab2Id);

      // Tab 1 should be active now
      expect(useUiStore.getState().activeTabId).toBe(tab1Id);
    });

    it("sets active tab", () => {
      const newTabId = useUiStore.getState().addTab({
        type: "task",
        title: "New Tab",
        path: "/new",
        closable: true,
      });

      useUiStore.getState().setActiveTab("home");
      expect(useUiStore.getState().activeTabId).toBe("home");

      useUiStore.getState().setActiveTab(newTabId);
      expect(useUiStore.getState().activeTabId).toBe(newTabId);
    });

    it("updates tab title", () => {
      useUiStore.getState().updateTabTitle("home", "Tasks Home");
      expect(useUiStore.getState().tabs.find((t) => t.id === "home")?.title).toBe("Tasks Home");
    });

    it("updates tab path", () => {
      const tabId = useUiStore.getState().addTab({
        type: "task",
        title: "Test Tab",
        path: "/test",
        closable: true,
      });

      useUiStore.getState().updateTabPath(tabId, "/updated-path");
      expect(useUiStore.getState().tabs.find((t) => t.id === tabId)?.path).toBe("/updated-path");
    });

    it("updates multiple tab properties with updateTab", () => {
      const tabId = useUiStore.getState().addTab({
        type: "task",
        title: "Test Tab",
        path: "/test",
        closable: true,
      });

      useUiStore.getState().updateTab(tabId, { title: "Updated Title", path: "/new-path" });

      const tab = useUiStore.getState().tabs.find((t) => t.id === tabId);
      expect(tab?.title).toBe("Updated Title");
      expect(tab?.path).toBe("/new-path");
    });
  });

  describe("dialogs", () => {
    it("toggles command palette", () => {
      expect(useUiStore.getState().isCommandPaletteOpen).toBe(false);

      useUiStore.getState().toggleCommandPalette();
      expect(useUiStore.getState().isCommandPaletteOpen).toBe(true);

      useUiStore.getState().toggleCommandPalette();
      expect(useUiStore.getState().isCommandPaletteOpen).toBe(false);
    });

    it("sets command palette open state", () => {
      useUiStore.getState().setCommandPaletteOpen(true);
      expect(useUiStore.getState().isCommandPaletteOpen).toBe(true);

      useUiStore.getState().setCommandPaletteOpen(false);
      expect(useUiStore.getState().isCommandPaletteOpen).toBe(false);
    });

    it("sets task creation open state", () => {
      useUiStore.getState().setTaskCreationOpen(true);
      expect(useUiStore.getState().isTaskCreationOpen).toBe(true);

      useUiStore.getState().setTaskCreationOpen(false);
      expect(useUiStore.getState().isTaskCreationOpen).toBe(false);
    });

    it("sets settings open state", () => {
      useUiStore.getState().setSettingsOpen(true);
      expect(useUiStore.getState().isSettingsOpen).toBe(true);

      useUiStore.getState().setSettingsOpen(false);
      expect(useUiStore.getState().isSettingsOpen).toBe(false);
    });
  });

  // Note: selectedPanel was removed in the new entity model rewrite
});
