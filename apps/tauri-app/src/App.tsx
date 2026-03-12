import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MainLayout } from "@/components/layout/MainLayout";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { WorkspaceHome } from "@/pages/WorkspaceHome";
import { UnitTaskDetail } from "@/pages/UnitTaskDetail";
import { Settings } from "@/pages/Settings";
import { Notifications } from "@/pages/Notifications";
import { useTheme } from "@/hooks/useTheme";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 30, // 30 seconds
      retry: 1,
    },
  },
});

function AppRoutes() {
  // Initialize theme (applies dark class to <html>)
  useTheme();

  return (
    <Routes>
      {/* Main app with sidebar */}
      <Route element={<MainLayout />}>
        <Route path="/" element={<WorkspaceHome />} />
        <Route path="/tasks/:id" element={<UnitTaskDetail />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/notifications" element={<Notifications />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <BrowserRouter>
          <AppRoutes />
        </BrowserRouter>
      </QueryClientProvider>
    </ErrorBoundary>
  );
}

export default App;
