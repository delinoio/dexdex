// Mobile platform detection and utilities
import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

/**
 * Platform information from Tauri backend.
 */
export interface PlatformInfo {
  isMobile: boolean;
  mobilePlatform: "ios" | "android" | null;
  supportsLocalMode: boolean;
  biometricAvailable: boolean;
  deviceModel: string | null;
  osVersion: string | null;
}

/**
 * Biometric authentication status.
 */
export interface BiometricStatus {
  available: boolean;
  biometricType: "face" | "fingerprint" | "iris" | "none";
  enrolled: boolean;
  description: string;
}

/**
 * Push notification registration status.
 */
export interface PushRegistrationStatus {
  supported: boolean;
  permissionGranted: boolean;
  deviceToken: string | null;
  service: "apns" | "fcm" | null;
  error: string | null;
}

// Breakpoints matching Tailwind defaults
const MOBILE_BREAKPOINT = 768; // md breakpoint

/**
 * Hook to detect if the current viewport is mobile-sized.
 * This is for responsive design, not platform detection.
 */
export function useIsMobileViewport(): boolean {
  const [isMobile, setIsMobile] = useState(
    typeof window !== "undefined" ? window.innerWidth < MOBILE_BREAKPOINT : false
  );

  useEffect(() => {
    let timeoutId: ReturnType<typeof setTimeout>;

    const checkMobile = () => {
      setIsMobile(window.innerWidth < MOBILE_BREAKPOINT);
    };

    const debouncedCheckMobile = () => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(checkMobile, 150);
    };

    window.addEventListener("resize", debouncedCheckMobile);
    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener("resize", debouncedCheckMobile);
    };
  }, []);

  return isMobile;
}

/**
 * Hook to get platform information from Tauri backend.
 */
export function usePlatformInfo() {
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function fetchPlatformInfo() {
      try {
        const info = await invoke<PlatformInfo>("get_platform_info");
        setPlatformInfo(info);
      } catch (e) {
        setError(e instanceof Error ? e.message : "Failed to get platform info");
        // Fallback for when running in browser
        setPlatformInfo({
          isMobile: false,
          mobilePlatform: null,
          supportsLocalMode: true,
          biometricAvailable: false,
          deviceModel: null,
          osVersion: null,
        });
      } finally {
        setIsLoading(false);
      }
    }

    fetchPlatformInfo();
  }, []);

  return { platformInfo, isLoading, error };
}

/**
 * Hook to check if running on a mobile platform (iOS/Android).
 */
export function useIsMobilePlatform(): boolean {
  const { platformInfo } = usePlatformInfo();
  return platformInfo?.isMobile ?? false;
}

/**
 * Hook for biometric authentication.
 */
export function useBiometricAuth() {
  const [status, setStatus] = useState<BiometricStatus | null>(null);
  const [isAuthenticating, setIsAuthenticating] = useState(false);

  useEffect(() => {
    async function checkAvailability() {
      try {
        const result = await invoke<BiometricStatus>("check_biometric_availability");
        setStatus(result);
      } catch {
        setStatus({
          available: false,
          biometricType: "none",
          enrolled: false,
          description: "Biometric authentication not available",
        });
      }
    }

    checkAvailability();
  }, []);

  const authenticate = useCallback(async (reason: string): Promise<boolean> => {
    if (!status?.available) {
      return false;
    }

    setIsAuthenticating(true);
    try {
      const result = await invoke<boolean>("authenticate_biometric", { reason });
      return result;
    } catch {
      return false;
    } finally {
      setIsAuthenticating(false);
    }
  }, [status?.available]);

  return {
    status,
    isAuthenticating,
    authenticate,
  };
}

/**
 * Hook for push notifications.
 */
export function usePushNotifications() {
  const [status, setStatus] = useState<PushRegistrationStatus | null>(null);
  const [isRegistering, setIsRegistering] = useState(false);

  const requestPermission = useCallback(async (): Promise<boolean> => {
    try {
      return await invoke<boolean>("request_push_permission");
    } catch {
      return false;
    }
  }, []);

  const register = useCallback(async (): Promise<PushRegistrationStatus | null> => {
    setIsRegistering(true);
    try {
      const result = await invoke<PushRegistrationStatus>("register_push_notifications");
      setStatus(result);
      return result;
    } catch {
      return null;
    } finally {
      setIsRegistering(false);
    }
  }, []);

  const unregister = useCallback(async (): Promise<boolean> => {
    try {
      await invoke("unregister_push_notifications");
      setStatus(null);
      return true;
    } catch {
      return false;
    }
  }, []);

  return {
    status,
    isRegistering,
    requestPermission,
    register,
    unregister,
  };
}

/**
 * Hook for touch gestures (swipe detection).
 */
export function useSwipeGesture(
  onSwipeLeft?: () => void,
  onSwipeRight?: () => void,
  threshold = 50
) {
  const [touchStart, setTouchStart] = useState<number | null>(null);
  const [touchEnd, setTouchEnd] = useState<number | null>(null);

  const onTouchStart = useCallback((e: React.TouchEvent) => {
    setTouchEnd(null);
    setTouchStart(e.targetTouches[0].clientX);
  }, []);

  const onTouchMove = useCallback((e: React.TouchEvent) => {
    setTouchEnd(e.targetTouches[0].clientX);
  }, []);

  const onTouchEnd = useCallback(() => {
    if (!touchStart || !touchEnd) return;

    const distance = touchStart - touchEnd;
    const isLeftSwipe = distance > threshold;
    const isRightSwipe = distance < -threshold;

    if (isLeftSwipe && onSwipeLeft) {
      onSwipeLeft();
    }

    if (isRightSwipe && onSwipeRight) {
      onSwipeRight();
    }
  }, [touchStart, touchEnd, threshold, onSwipeLeft, onSwipeRight]);

  return {
    onTouchStart,
    onTouchMove,
    onTouchEnd,
  };
}
