// Theme state management with Zustand
import { create } from "zustand";
import { persist } from "zustand/middleware";

/** User-selectable theme modes */
export enum ThemeMode {
  Light = "light",
  Dark = "dark",
  System = "system",
}

/** Resolved theme after evaluating system preference */
export type ResolvedTheme = "light" | "dark";

interface ThemeState {
  /** The user's selected theme mode */
  mode: ThemeMode;

  /** Set the theme mode */
  setMode: (mode: ThemeMode) => void;

  /** Cycle through theme modes: light -> dark -> system -> light */
  cycleMode: () => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      mode: ThemeMode.System,

      setMode: (mode) => set({ mode }),

      cycleMode: () =>
        set((state) => {
          const order = [ThemeMode.Light, ThemeMode.Dark, ThemeMode.System];
          const currentIndex = order.indexOf(state.mode);
          const nextIndex = (currentIndex + 1) % order.length;
          return { mode: order[nextIndex] };
        }),
    }),
    {
      name: "dexdex-theme-store",
    }
  )
);

/** Resolve the effective theme based on mode and system preference */
export function resolveTheme(mode: ThemeMode): ResolvedTheme {
  if (mode === ThemeMode.System) {
    return window.matchMedia("(prefers-color-scheme: dark)").matches
      ? "dark"
      : "light";
  }
  return mode;
}
