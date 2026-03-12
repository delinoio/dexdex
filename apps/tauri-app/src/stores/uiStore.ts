// UI state management with Zustand
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type TabType = 'workspace' | 'task' | 'pr' | 'settings' | 'notifications';

interface Tab {
  id: string;
  type: TabType;
  entityId?: string;
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
  addTab: (tab: Omit<Tab, 'id'>) => string;
  removeTab: (id: string) => void;
  setActiveTab: (id: string) => void;
  updateTabTitle: (id: string, title: string) => void;
  updateTabPath: (id: string, path: string) => void;
  updateTab: (id: string, updates: Partial<Omit<Tab, 'id'>>) => void;

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
          id: 'home',
          type: 'workspace',
          title: 'Tasks',
          path: '/',
          closable: false,
        },
      ],
      activeTabId: 'home',
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
          const newIndex = Math.max(0, tabIndex - 1);
          newActiveTabId = newTabs[newIndex]?.id ?? null;
        }

        set({ tabs: newTabs, activeTabId: newActiveTabId });
      },
      setActiveTab: (id) => set({ activeTabId: id }),
      updateTabTitle: (id, title) =>
        set((state) => ({
          tabs: state.tabs.map((t) => (t.id === id ? { ...t, title } : t)),
        })),
      updateTabPath: (id, path) =>
        set((state) => ({
          tabs: state.tabs.map((t) => (t.id === id ? { ...t, path } : t)),
        })),
      updateTab: (id, updates) =>
        set((state) => ({
          tabs: state.tabs.map((t) => (t.id === id ? { ...t, ...updates } : t)),
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
    }),
    {
      name: 'dexdex-ui-store',
      partialize: (state) => ({
        sidebarCollapsed: state.sidebarCollapsed,
        currentWorkspaceId: state.currentWorkspaceId,
      }),
    }
  )
);
