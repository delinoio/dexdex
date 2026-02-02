import { renderHook, act } from "@testing-library/react";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { useIsMobileViewport, useSwipeGesture } from "../useMobile";

describe("useMobile hooks", () => {
  describe("useIsMobileViewport", () => {
    const originalInnerWidth = window.innerWidth;

    beforeEach(() => {
      vi.spyOn(window, "addEventListener");
      vi.spyOn(window, "removeEventListener");
    });

    afterEach(() => {
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        value: originalInnerWidth,
      });
      vi.restoreAllMocks();
    });

    it("returns true when viewport is less than 768px", () => {
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        value: 375,
      });

      const { result } = renderHook(() => useIsMobileViewport());
      expect(result.current).toBe(true);
    });

    it("returns false when viewport is 768px or more", () => {
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        value: 1024,
      });

      const { result } = renderHook(() => useIsMobileViewport());
      expect(result.current).toBe(false);
    });

    it("updates when window is resized", () => {
      Object.defineProperty(window, "innerWidth", {
        writable: true,
        value: 1024,
      });

      const { result } = renderHook(() => useIsMobileViewport());
      expect(result.current).toBe(false);

      // Simulate resize to mobile
      act(() => {
        Object.defineProperty(window, "innerWidth", {
          writable: true,
          value: 375,
        });
        window.dispatchEvent(new Event("resize"));
      });

      expect(result.current).toBe(true);
    });

    it("adds and removes resize event listener", () => {
      const { unmount } = renderHook(() => useIsMobileViewport());

      expect(window.addEventListener).toHaveBeenCalledWith(
        "resize",
        expect.any(Function)
      );

      unmount();

      expect(window.removeEventListener).toHaveBeenCalledWith(
        "resize",
        expect.any(Function)
      );
    });
  });

  describe("useSwipeGesture", () => {
    it("calls onSwipeLeft when swiping left beyond threshold", () => {
      const onSwipeLeft = vi.fn();
      const onSwipeRight = vi.fn();

      const { result } = renderHook(() =>
        useSwipeGesture(onSwipeLeft, onSwipeRight, 50)
      );

      // Simulate touch start
      act(() => {
        result.current.onTouchStart({
          targetTouches: [{ clientX: 200 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch move (swipe left)
      act(() => {
        result.current.onTouchMove({
          targetTouches: [{ clientX: 100 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch end
      act(() => {
        result.current.onTouchEnd();
      });

      expect(onSwipeLeft).toHaveBeenCalled();
      expect(onSwipeRight).not.toHaveBeenCalled();
    });

    it("calls onSwipeRight when swiping right beyond threshold", () => {
      const onSwipeLeft = vi.fn();
      const onSwipeRight = vi.fn();

      const { result } = renderHook(() =>
        useSwipeGesture(onSwipeLeft, onSwipeRight, 50)
      );

      // Simulate touch start
      act(() => {
        result.current.onTouchStart({
          targetTouches: [{ clientX: 100 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch move (swipe right)
      act(() => {
        result.current.onTouchMove({
          targetTouches: [{ clientX: 200 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch end
      act(() => {
        result.current.onTouchEnd();
      });

      expect(onSwipeRight).toHaveBeenCalled();
      expect(onSwipeLeft).not.toHaveBeenCalled();
    });

    it("does not call callbacks when swipe is below threshold", () => {
      const onSwipeLeft = vi.fn();
      const onSwipeRight = vi.fn();

      const { result } = renderHook(() =>
        useSwipeGesture(onSwipeLeft, onSwipeRight, 50)
      );

      // Simulate touch start
      act(() => {
        result.current.onTouchStart({
          targetTouches: [{ clientX: 100 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch move (small movement)
      act(() => {
        result.current.onTouchMove({
          targetTouches: [{ clientX: 120 }],
        } as unknown as React.TouchEvent);
      });

      // Simulate touch end
      act(() => {
        result.current.onTouchEnd();
      });

      expect(onSwipeLeft).not.toHaveBeenCalled();
      expect(onSwipeRight).not.toHaveBeenCalled();
    });

    it("uses default threshold of 50", () => {
      const onSwipeLeft = vi.fn();

      const { result } = renderHook(() => useSwipeGesture(onSwipeLeft));

      // Simulate swipe of exactly 50px (should trigger)
      act(() => {
        result.current.onTouchStart({
          targetTouches: [{ clientX: 150 }],
        } as unknown as React.TouchEvent);
      });

      act(() => {
        result.current.onTouchMove({
          targetTouches: [{ clientX: 99 }],
        } as unknown as React.TouchEvent);
      });

      act(() => {
        result.current.onTouchEnd();
      });

      expect(onSwipeLeft).toHaveBeenCalled();
    });
  });
});
