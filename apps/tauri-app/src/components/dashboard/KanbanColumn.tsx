import type { ReactNode } from "react";
import { cn } from "@/lib/utils";

interface KanbanColumnProps {
  title: string;
  count: number;
  children: ReactNode;
  className?: string;
}

export function KanbanColumn({
  title,
  count,
  children,
  className,
}: KanbanColumnProps) {
  return (
    <div className={cn("flex h-full flex-col rounded-lg", className)}>
      <div className="mb-3 flex items-center justify-between">
        <h3 className="text-sm font-semibold text-[hsl(var(--foreground))]">
          {title}
        </h3>
        <span className="rounded-full bg-[hsl(var(--muted))] px-2 py-0.5 text-xs font-medium text-[hsl(var(--muted-foreground))]">
          {count}
        </span>
      </div>
      <div className="flex-1 space-y-2 overflow-y-auto">{children}</div>
    </div>
  );
}
