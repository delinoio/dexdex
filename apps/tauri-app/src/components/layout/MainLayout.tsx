import { Outlet } from "react-router-dom";
import { Sidebar } from "./Sidebar";
import { TabBar } from "./TabBar";
import { MobileNavigation } from "@/components/mobile";
import { useIsMobileViewport } from "@/hooks/useMobile";

export function MainLayout() {
  const isMobile = useIsMobileViewport();

  return (
    <div className="flex h-screen overflow-hidden">
      {/* Desktop sidebar - hidden on mobile */}
      {!isMobile && <Sidebar />}

      {/* Main content area with tab bar */}
      <div className="flex flex-1 flex-col overflow-hidden">
        {/* Tab bar - desktop only */}
        {!isMobile && <TabBar />}

        {/* Page content */}
        <main className="flex-1 overflow-hidden">
          {/* Add padding-bottom on mobile for the bottom navigation */}
          <div className={isMobile ? "h-full pb-20" : "h-full"}>
            <Outlet />
          </div>
        </main>
      </div>

      {/* Mobile bottom navigation - only shown on mobile */}
      {isMobile && <MobileNavigation />}
    </div>
  );
}
