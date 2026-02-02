// Swipeable card component for mobile task lists
import { useState, useRef, type ReactNode } from "react";
import { cn } from "@/lib/utils";

interface SwipeAction {
  label: string;
  icon?: ReactNode;
  color: "destructive" | "success" | "warning" | "primary";
  onClick: () => void;
}

interface SwipeableCardProps {
  children: ReactNode;
  leftAction?: SwipeAction;
  rightAction?: SwipeAction;
  className?: string;
}

const colorClasses = {
  destructive: "bg-[hsl(var(--destructive))]",
  success: "bg-green-500",
  warning: "bg-yellow-500",
  primary: "bg-[hsl(var(--primary))]",
};

export function SwipeableCard({
  children,
  leftAction,
  rightAction,
  className,
}: SwipeableCardProps) {
  const [offset, setOffset] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const startXRef = useRef(0);
  const containerRef = useRef<HTMLDivElement>(null);

  const threshold = 80; // Minimum swipe distance to trigger action
  const maxSwipe = 100; // Maximum swipe distance

  const handleTouchStart = (e: React.TouchEvent) => {
    startXRef.current = e.touches[0].clientX;
    setIsDragging(true);
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDragging) return;

    const currentX = e.touches[0].clientX;
    const diff = currentX - startXRef.current;

    // Limit swipe distance
    const limitedDiff = Math.max(-maxSwipe, Math.min(maxSwipe, diff));

    // Only allow swipe if there's an action for that direction
    if (diff > 0 && !leftAction) return;
    if (diff < 0 && !rightAction) return;

    setOffset(limitedDiff);
  };

  const handleTouchEnd = () => {
    setIsDragging(false);

    if (offset > threshold && leftAction) {
      leftAction.onClick();
    } else if (offset < -threshold && rightAction) {
      rightAction.onClick();
    }

    // Reset position with animation
    setOffset(0);
  };

  return (
    <div className={cn("relative overflow-hidden rounded-lg", className)}>
      {/* Left action background */}
      {leftAction && (
        <div
          className={cn(
            "absolute inset-y-0 left-0 flex w-24 items-center justify-center",
            colorClasses[leftAction.color]
          )}
        >
          <div className="flex flex-col items-center gap-1 text-white">
            {leftAction.icon}
            <span className="text-xs font-medium">{leftAction.label}</span>
          </div>
        </div>
      )}

      {/* Right action background */}
      {rightAction && (
        <div
          className={cn(
            "absolute inset-y-0 right-0 flex w-24 items-center justify-center",
            colorClasses[rightAction.color]
          )}
        >
          <div className="flex flex-col items-center gap-1 text-white">
            {rightAction.icon}
            <span className="text-xs font-medium">{rightAction.label}</span>
          </div>
        </div>
      )}

      {/* Swipeable content */}
      <div
        ref={containerRef}
        className={cn(
          "relative bg-[hsl(var(--card))] touch-manipulation",
          !isDragging && "transition-transform duration-200 ease-out"
        )}
        style={{ transform: `translateX(${offset}px)` }}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
      >
        {children}
      </div>
    </div>
  );
}
