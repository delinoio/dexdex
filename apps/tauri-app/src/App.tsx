import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MainLayout } from "@/components/layout/MainLayout";
import {
  CompositeTaskDetail,
  Dashboard,
  ModeSelection,
  Onboarding,
  Repositories,
  Settings,
  TaskCreation,
  UnitTaskDetail,
} from "@/pages";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 1000 * 60, // 1 minute
      retry: 1,
    },
  },
});

function AppRoutes() {
  // Initialize keyboard shortcuts
  useKeyboardShortcuts();

  return (
    <Routes>
      {/* Standalone pages (no sidebar) */}
      <Route path="/mode-select" element={<ModeSelection />} />
      <Route path="/onboarding" element={<Onboarding />} />

      {/* Main app with sidebar */}
      <Route element={<MainLayout />}>
        <Route path="/" element={<Dashboard />} />
        <Route path="/tasks/new" element={<TaskCreation />} />
        <Route path="/unit-tasks/:id" element={<UnitTaskDetail />} />
        <Route path="/composite-tasks/:id" element={<CompositeTaskDetail />} />
        <Route path="/repositories" element={<Repositories />} />
        <Route path="/settings" element={<Settings />} />
      </Route>
    </Routes>
  );
}

function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </QueryClientProvider>
  );
}

export default App;
