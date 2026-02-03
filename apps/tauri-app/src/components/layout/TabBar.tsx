import { useNavigate, useLocation } from "react-router-dom";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/uiStore";
import { useCallback, useEffect, MouseEvent } from "react";

export function TabBar() {
  const navigate = useNavigate();
  const location = useLocation();
  const tabs = useUiStore((state) => state.tabs);
  const activeTabId = useUiStore((state) => state.activeTabId);
  const setActiveTab = useUiStore((state) => state.setActiveTab);
  const removeTab = useUiStore((state) => state.removeTab);

  // Sync route with active tab
  useEffect(() => {
    const activeTab = tabs.find((t) => t.id === activeTabId);
    if (activeTab && activeTab.path !== location.pathname) {
      navigate(activeTab.path);
    }
  }, [activeTabId, tabs, navigate, location.pathname]);

  const handleTabClick = useCallback(
    (tabId: string, path: string) => {
      setActiveTab(tabId);
      navigate(path);
    },
    [setActiveTab, navigate]
  );

  const handleCloseTab = useCallback(
    (e: MouseEvent, tabId: string) => {
      e.stopPropagation();
      removeTab(tabId);
    },
    [removeTab]
  );

  const handleMiddleClick = useCallback(
    (e: MouseEvent, tabId: string, closable: boolean) => {
      if (e.button === 1 && closable) {
        e.preventDefault();
        removeTab(tabId);
      }
    },
    [removeTab]
  );

  if (tabs.length <= 1) {
    return null;
  }

  return (
    <div className="flex h-9 items-center border-b border-[hsl(var(--border))] bg-[hsl(var(--background))]">
      <div className="flex h-full items-center overflow-x-auto">
        {tabs.map((tab) => (
          <div
            key={tab.id}
            onClick={() => handleTabClick(tab.id, tab.path)}
            onMouseDown={(e) => handleMiddleClick(e, tab.id, tab.closable)}
            className={cn(
              "group flex h-full cursor-pointer items-center gap-2 border-r border-[hsl(var(--border))] px-3 text-sm transition-colors",
              tab.id === activeTabId
                ? "bg-[hsl(var(--muted))] text-[hsl(var(--foreground))]"
                : "text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--muted)/0.5)] hover:text-[hsl(var(--foreground))]"
            )}
          >
            <span className="max-w-[120px] truncate">{tab.title}</span>
            {tab.closable && (
              <button
                onClick={(e) => handleCloseTab(e, tab.id)}
                className={cn(
                  "flex h-4 w-4 items-center justify-center rounded-sm transition-colors",
                  "opacity-0 group-hover:opacity-100",
                  "hover:bg-[hsl(var(--destructive)/0.2)] hover:text-[hsl(var(--destructive))]"
                )}
                aria-label={`Close ${tab.title} tab`}
              >
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  width="12"
                  height="12"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="2"
                  strokeLinecap="round"
                  strokeLinejoin="round"
                >
                  <path d="M18 6 6 18" />
                  <path d="m6 6 12 12" />
                </svg>
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
