import { useCallback, useEffect } from "react";
import { useNavigate, useLocation } from "react-router-dom";
import { useUiStore } from "@/stores/uiStore";

interface OpenInTabOptions {
  title?: string;
  closable?: boolean;
}

/**
 * Hook for tab navigation functionality.
 * Syncs router state with tab state and provides utilities for opening links in new tabs.
 */
export function useTabNavigation() {
  const navigate = useNavigate();
  const location = useLocation();
  const tabs = useUiStore((state) => state.tabs);
  const activeTabId = useUiStore((state) => state.activeTabId);
  const addTab = useUiStore((state) => state.addTab);
  const setActiveTab = useUiStore((state) => state.setActiveTab);
  const updateTabTitle = useUiStore((state) => state.updateTabTitle);

  // Update the active tab's path when route changes
  useEffect(() => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab && activeTab.path !== location.pathname) {
      // Find if there's already a tab for this path
      const existingTab = tabs.find((t) => t.path === location.pathname);
      if (existingTab) {
        setActiveTab(existingTab.id);
      }
    }
  }, [location.pathname, tabs, activeTabId, setActiveTab]);

  /**
   * Opens a path in a new tab.
   */
  const openInNewTab = useCallback(
    (path: string, options: OpenInTabOptions = {}) => {
      const { title = "New Tab", closable = true } = options;

      // Check if path is already open in a tab
      const existingTab = tabs.find((t) => t.path === path);
      if (existingTab) {
        setActiveTab(existingTab.id);
        navigate(path);
        return existingTab.id;
      }

      const tabId = addTab({ title, path, closable });
      navigate(path);
      return tabId;
    },
    [tabs, addTab, setActiveTab, navigate]
  );

  /**
   * Navigate with optional new tab support.
   * Opens in new tab when Ctrl/Cmd+Click is used.
   */
  const navigateWithTab = useCallback(
    (
      path: string,
      event?: React.MouseEvent,
      options: OpenInTabOptions = {}
    ) => {
      const shouldOpenNewTab =
        event && (event.metaKey || event.ctrlKey || event.button === 1);

      if (shouldOpenNewTab) {
        event.preventDefault();
        openInNewTab(path, options);
      } else {
        navigate(path);
      }
    },
    [navigate, openInNewTab]
  );

  /**
   * Updates the current active tab's title.
   */
  const setCurrentTabTitle = useCallback(
    (title: string) => {
      if (activeTabId) {
        updateTabTitle(activeTabId, title);
      }
    },
    [activeTabId, updateTabTitle]
  );

  return {
    openInNewTab,
    navigateWithTab,
    setCurrentTabTitle,
    tabs,
    activeTabId,
  };
}
