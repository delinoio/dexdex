// Tab bar component for multi-tab navigation
import { useNavigate } from "react-router-dom";
import { cn } from "@/lib/utils";
import { useUiStore } from "@/stores/uiStore";

interface TabBarProps {
  className?: string;
}

export function TabBar({ className }: TabBarProps) {
  const navigate = useNavigate();
  const { tabs, activeTabId, setActiveTab, removeTab } = useUiStore();

  const handleTabClick = (tabId: string, path: string) => {
    setActiveTab(tabId);
    navigate(path);
  };

  const handleCloseTab = (
    e: React.MouseEvent,
    tabId: string,
    closable: boolean
  ) => {
    e.stopPropagation();
    if (!closable) return;

    const tabIndex = tabs.findIndex((t) => t.id === tabId);
    const isActive = activeTabId === tabId;

    removeTab(tabId);

    // If closing the active tab, navigate to the new active tab
    if (isActive) {
      const remainingTabs = tabs.filter((t) => t.id !== tabId);
      const newActiveIndex = Math.max(0, tabIndex - 1);
      const newActiveTab = remainingTabs[newActiveIndex];
      if (newActiveTab) {
        navigate(newActiveTab.path);
      }
    }
  };

  const handleMiddleClick = (
    e: React.MouseEvent,
    tabId: string,
    closable: boolean
  ) => {
    // Middle mouse button is button 1
    if (e.button === 1 && closable) {
      handleCloseTab(e, tabId, closable);
    }
  };

  if (tabs.length <= 1) {
    // Don't show tab bar if there's only one tab
    return null;
  }

  return (
    <div
      className={cn(
        "flex h-9 items-center gap-1 border-b border-[hsl(var(--border))] bg-[hsl(var(--background))] px-2",
        className
      )}
    >
      {tabs.map((tab) => (
        <div
          key={tab.id}
          role="tab"
          tabIndex={0}
          onClick={() => handleTabClick(tab.id, tab.path)}
          onMouseDown={(e) => handleMiddleClick(e, tab.id, tab.closable)}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              handleTabClick(tab.id, tab.path);
            }
          }}
          className={cn(
            "group relative flex h-7 max-w-48 cursor-pointer items-center gap-2 rounded-md px-3 text-sm transition-colors",
            activeTabId === tab.id
              ? "bg-[hsl(var(--muted))] text-[hsl(var(--foreground))]"
              : "text-[hsl(var(--muted-foreground))] hover:bg-[hsl(var(--muted))] hover:text-[hsl(var(--foreground))]"
          )}
        >
          <span className="truncate">{tab.title}</span>
          {tab.closable && (
            <button
              onClick={(e) => handleCloseTab(e, tab.id, tab.closable)}
              className={cn(
                "flex h-4 w-4 items-center justify-center rounded-sm transition-opacity",
                "opacity-0 hover:bg-[hsl(var(--destructive)/0.2)] group-hover:opacity-100",
                activeTabId === tab.id && "opacity-100"
              )}
              aria-label={`Close ${tab.title}`}
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
  );
}
