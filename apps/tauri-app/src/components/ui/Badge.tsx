import { type HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export interface BadgeProps extends HTMLAttributes<HTMLDivElement> {
  variant?: "default" | "secondary" | "destructive" | "outline";
}

function Badge({ className, variant = "default", ...props }: BadgeProps) {
  return (
    <div
      className={cn(
        "inline-flex items-center rounded-md border px-2.5 py-0.5 text-xs font-semibold transition-colors focus:outline-none focus:ring-2 focus:ring-[hsl(var(--ring))] focus:ring-offset-2",
        variant === "default" &&
          "border-transparent bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))] shadow hover:bg-[hsl(var(--primary))]/80",
        variant === "secondary" &&
          "border-transparent bg-[hsl(var(--secondary))] text-[hsl(var(--secondary-foreground))] hover:bg-[hsl(var(--secondary))]/80",
        variant === "destructive" &&
          "border-transparent bg-[hsl(var(--destructive))] text-[hsl(var(--destructive-foreground))] shadow hover:bg-[hsl(var(--destructive))]/80",
        variant === "outline" && "text-[hsl(var(--foreground))]",
        className
      )}
      {...props}
    />
  );
}

export { Badge };
