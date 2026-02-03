import { type HTMLAttributes } from "react";
import { cn } from "@/lib/utils";

export interface FormattedDateTimeProps extends HTMLAttributes<HTMLTimeElement> {
  /** ISO date string or Date object to format */
  date: string | Date;
  /** Whether to include the time (hour:minute). Defaults to true */
  includeTime?: boolean;
}

/**
 * Formats a date string or Date object to display date with hour:minute.
 * Uses the browser's locale for formatting.
 */
export function formatDateTime(date: string | Date, includeTime = true): string {
  const dateObj = typeof date === "string" ? new Date(date) : date;

  if (includeTime) {
    return dateObj.toLocaleString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
      hour: "2-digit",
      minute: "2-digit",
    });
  }

  return dateObj.toLocaleDateString(undefined, {
    year: "numeric",
    month: "short",
    day: "numeric",
  });
}

/**
 * A shared component for displaying formatted date and time.
 * Shows date with hour:minute using the browser's locale.
 */
function FormattedDateTime({
  date,
  includeTime = true,
  className,
  ...props
}: FormattedDateTimeProps) {
  const dateObj = typeof date === "string" ? new Date(date) : date;
  const formatted = formatDateTime(date, includeTime);

  return (
    <time
      dateTime={dateObj.toISOString()}
      className={cn(className)}
      {...props}
    >
      {formatted}
    </time>
  );
}

export { FormattedDateTime };
