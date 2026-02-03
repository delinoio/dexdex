// Tab navigation hook for syncing router with tab state
import { useEffect, useCallback } from "react";
import { useLocation, useNavigate } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";

// Map paths to default tab titles
const getDefaultTitle = (path: string): string => {
  if (path === "/") return "Dashboard";
  if (path === "/repositories") return "Repositories";
  if (path === "/repository-groups") return "Groups";
  if (path === "/settings") return "Settings";
  if (path === "/tasks/new") return "New Task";
  if (path.startsWith("/unit-tasks/")) return "Task";
  if (path.startsWith("/composite-tasks/")) return "Composite Task";
  return "New Tab";
};

export function useTabNavigation() {
  const location = useLocation();
  const navigate = useNavigate();
  const { tabs, activeTabId, addTab, setActiveTab, updateTab } = useUiStore();

  // Sync current path with active tab
  useEffect(() => {
    const currentPath = location.pathname;

    // Find if there's already a tab with this path
    const existingTab = tabs.find((t) => t.path === currentPath);

    if (existingTab) {
      // Activate the existing tab if not already active
      if (activeTabId !== existingTab.id) {
        setActiveTab(existingTab.id);
      }
    } else {
      // Check if the active tab should be updated instead of creating a new one
      const activeTab = tabs.find((t) => t.id === activeTabId);

      // Only update the active tab path if it's a closable tab navigating to a new location
      // This handles normal navigation within the app
      if (activeTab && activeTab.closable) {
        // Update the active tab's path and title
        const newTitle = getDefaultTitle(currentPath);
        updateTab(activeTab.id, { path: currentPath, title: newTitle });
      }
    }
  }, [location.pathname, tabs, activeTabId, setActiveTab, updateTab]);

  // Open a link in a new tab (for Ctrl/Cmd+Click)
  const openInNewTab = useCallback(
    (path: string, title?: string) => {
      const tabTitle = title || getDefaultTitle(path);

      // Check if tab with this path already exists
      const existingTab = tabs.find((t) => t.path === path);
      if (existingTab) {
        setActiveTab(existingTab.id);
        navigate(path);
        return;
      }

      // Add new tab and navigate
      const newTabId = addTab({
        title: tabTitle,
        path,
        closable: true,
      });
      setActiveTab(newTabId);
      navigate(path);
    },
    [tabs, addTab, setActiveTab, navigate]
  );

  // Handle link click with modifier key detection
  const handleLinkClick = useCallback(
    (
      e: React.MouseEvent<HTMLElement>,
      path: string,
      title?: string
    ): boolean => {
      const isMac =
        typeof navigator !== "undefined" &&
        navigator.platform.toUpperCase().indexOf("MAC") >= 0;
      const modKey = isMac ? e.metaKey : e.ctrlKey;

      if (modKey) {
        e.preventDefault();
        openInNewTab(path, title);
        return true;
      }

      return false;
    },
    [openInNewTab]
  );

  return {
    openInNewTab,
    handleLinkClick,
  };
}

// Hook to update the current tab's title
export function useTabTitle(title: string) {
  const { activeTabId, updateTabTitle } = useUiStore();

  useEffect(() => {
    if (activeTabId && title) {
      updateTabTitle(activeTabId, title);
    }
  }, [activeTabId, title, updateTabTitle]);
}
