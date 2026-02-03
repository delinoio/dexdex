// Hook for requesting notification permission on startup
import { useEffect, useRef } from "react";
import {
  isPermissionGranted,
  requestPermission,
} from "@tauri-apps/plugin-notification";

/**
 * Hook that requests notification permission on app startup.
 *
 * This hook should be called once at the root level of the app (e.g., in App.tsx)
 * to ensure the user is prompted for notification permission when the app starts.
 *
 * The permission request is only made once per app session. If the user has already
 * granted or denied permission, no prompt will be shown.
 */
export function useNotificationPermission(): void {
  const hasRequested = useRef(false);

  useEffect(() => {
    // Only request once per session
    if (hasRequested.current) {
      return;
    }
    hasRequested.current = true;

    async function checkAndRequestPermission() {
      try {
        const granted = await isPermissionGranted();

        if (granted) {
          return;
        }

        await requestPermission();
      } catch {
        // This can fail if not running in Tauri context (e.g., browser dev mode)
      }
    }

    checkAndRequestPermission();
  }, []);
}
