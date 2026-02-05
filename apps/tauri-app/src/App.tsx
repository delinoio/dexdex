import { BrowserRouter, Routes, Route } from "react-router-dom";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MainLayout } from "@/components/layout/MainLayout";
import { ErrorBoundary } from "@/components/ErrorBoundary";
import { CommandPalette } from "@/components/CommandPalette";
import { KeyboardShortcutsDialog } from "@/components/KeyboardShortcutsDialog";
import { ChatWindow } from "@/components/chat";
import {
  CompositeTaskDetail,
  Dashboard,
  ModeSelection,
  Onboarding,
  Repositories,
  RepositoryGroups,
  Settings,
  TaskCreation,
  UnitTaskDetail,
} from "@/pages";
import { useKeyboardShortcuts } from "@/hooks/useKeyboardShortcuts";
import { useNotificationPermission } from "@/hooks/useNotificationPermission";
import { useTaskEvents } from "@/hooks/useTaskEvents";

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
  // Request notification permission on startup
  useNotificationPermission();
  // Listen for task status/completion events and invalidate query caches
  useTaskEvents();

  return (
    <>
      <CommandPalette />
      <KeyboardShortcutsDialog />
      <ChatWindow />
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
          <Route path="/repository-groups" element={<RepositoryGroups />} />
          <Route path="/settings" element={<Settings />} />
        </Route>
      </Routes>
    </>
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
