// UI state management with Zustand
import { create } from "zustand";
import { persist } from "zustand/middleware";

interface Tab {
  id: string;
  title: string;
  path: string;
  closable: boolean;
}

interface UiState {
  // Sidebar state
  sidebarCollapsed: boolean;
  setSidebarCollapsed: (collapsed: boolean) => void;
  toggleSidebar: () => void;

  // Current workspace
  currentWorkspaceId: string | null;
  setCurrentWorkspaceId: (id: string | null) => void;

  // Tab management
  tabs: Tab[];
  activeTabId: string | null;
  addTab: (tab: Omit<Tab, "id">) => string;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabTitle: (id: string, title: string) => void;

  // Dialog state
  isCommandPaletteOpen: boolean;
  setCommandPaletteOpen: (open: boolean) => void;
  toggleCommandPalette: () => void;

  // Task creation dialog
  isTaskCreationOpen: boolean;
  setTaskCreationOpen: (open: boolean) => void;

  // Settings dialog
  isSettingsOpen: boolean;
  setSettingsOpen: (open: boolean) => void;

  // Selected panel
  selectedPanel: "dashboard" | "repositories" | "settings";
  setSelectedPanel: (panel: "dashboard" | "repositories" | "settings") => void;
}

let tabIdCounter = 0;

export const useUiStore = create<UiState>()(
  persist(
    (set, get) => ({
      // Sidebar
      sidebarCollapsed: false,
      setSidebarCollapsed: (collapsed) => set({ sidebarCollapsed: collapsed }),
      toggleSidebar: () =>
        set((state) => ({ sidebarCollapsed: !state.sidebarCollapsed })),

      // Current workspace
      currentWorkspaceId: null,
      setCurrentWorkspaceId: (id) => set({ currentWorkspaceId: id }),

      // Tabs
      tabs: [
    {
      id: "dashboard",
      title: "Dashboard",
      path: "/",
      closable: false,
    },
  ],
  activeTabId: "dashboard",
  addTab: (tab) => {
    const id = `tab-${++tabIdCounter}`;
    set((state) => ({
      tabs: [...state.tabs, { ...tab, id }],
      activeTabId: id,
    }));
    return id;
  },
  removeTab: (id) => {
    const state = get();
    const tabIndex = state.tabs.findIndex((t) => t.id === id);
    if (tabIndex === -1 || !state.tabs[tabIndex].closable) return;

    const newTabs = state.tabs.filter((t) => t.id !== id);
    let newActiveTabId = state.activeTabId;

    if (state.activeTabId === id) {
      // Select the previous tab, or the next one if this was the first
      const newIndex = Math.max(0, tabIndex - 1);
      newActiveTabId = newTabs[newIndex]?.id || null;
    }

    set({ tabs: newTabs, activeTabId: newActiveTabId });
  },
  setActiveTab: (id) => set({ activeTabId: id }),
  updateTabTitle: (id, title) =>
    set((state) => ({
      tabs: state.tabs.map((t) => (t.id === id ? { ...t, title } : t)),
    })),

  // Command palette
  isCommandPaletteOpen: false,
  setCommandPaletteOpen: (open) => set({ isCommandPaletteOpen: open }),
  toggleCommandPalette: () =>
    set((state) => ({ isCommandPaletteOpen: !state.isCommandPaletteOpen })),

  // Task creation
  isTaskCreationOpen: false,
  setTaskCreationOpen: (open) => set({ isTaskCreationOpen: open }),

  // Settings
  isSettingsOpen: false,
  setSettingsOpen: (open) => set({ isSettingsOpen: open }),

      // Panel selection
      selectedPanel: "dashboard",
      setSelectedPanel: (panel) => set({ selectedPanel: panel }),
    }),
    {
      name: "delidev-ui-store",
      partialize: (state) => ({
        sidebarCollapsed: state.sidebarCollapsed,
        currentWorkspaceId: state.currentWorkspaceId,
      }),
    }
  )
);
