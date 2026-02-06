import { useEffect, useSyncExternalStore } from "react";
import {
  ThemeMode,
  useThemeStore,
  resolveTheme,
  type ResolvedTheme,
} from "@/stores/themeStore";

/** Subscribe to system color scheme changes */
function subscribeToSystemTheme(callback: () => void): () => void {
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", callback);
  return () => mediaQuery.removeEventListener("change", callback);
}

function getSystemThemeSnapshot(): boolean {
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

/**
 * Hook that manages the dark mode class on <html> and returns current theme info.
 * Should be called once near the root of the app (e.g., in App.tsx).
 */
export function useTheme(): {
  mode: ThemeMode;
  resolvedTheme: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
  cycleMode: () => void;
} {
  const mode = useThemeStore((s) => s.mode);
  const setMode = useThemeStore((s) => s.setMode);
  const cycleMode = useThemeStore((s) => s.cycleMode);

  // Re-render when system preference changes (only matters for ThemeMode.System)
  const systemDark = useSyncExternalStore(
    subscribeToSystemTheme,
    getSystemThemeSnapshot
  );

  const resolvedTheme = resolveTheme(mode);

  // Apply the `.dark` class to <html> whenever the resolved theme changes
  useEffect(() => {
    const root = document.documentElement;
    if (resolvedTheme === "dark") {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }
  }, [resolvedTheme, systemDark]);

  return { mode, resolvedTheme, setMode, cycleMode };
}
