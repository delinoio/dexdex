import type { FileChangeEvent, FileChangeType } from "@/api/types";
import { cn } from "@/lib/utils";

interface FileChangeMessageProps {
  event: FileChangeEvent;
}

export function FileChangeMessage({ event }: FileChangeMessageProps) {
  const { changeType, displayType, fromPath } = parseChangeType(event.change_type);

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <span
        className={cn("font-medium", getChangeTypeColor(changeType))}
      >
        {displayType}
      </span>
      <code className="text-foreground bg-muted/50 px-1.5 py-0.5 rounded text-sm">
        {event.path}
      </code>
      {fromPath && (
        <>
          <span className="text-muted-foreground">from</span>
          <code className="text-foreground bg-muted/50 px-1.5 py-0.5 rounded text-sm">
            {fromPath}
          </code>
        </>
      )}
    </div>
  );
}

interface ParsedChangeType {
  changeType: FileChangeType | "rename";
  displayType: string;
  fromPath?: string;
}

function parseChangeType(
  changeType: FileChangeType | { rename: { from: string } }
): ParsedChangeType {
  if (typeof changeType === "string") {
    return {
      changeType,
      displayType: capitalizeFirst(changeType),
    };
  }

  // Handle rename object
  if (typeof changeType === "object" && "rename" in changeType) {
    return {
      changeType: "rename",
      displayType: "Rename",
      fromPath: changeType.rename.from,
    };
  }

  return {
    changeType: "modify" as FileChangeType,
    displayType: "Change",
  };
}

function getChangeTypeColor(changeType: FileChangeType | "rename"): string {
  switch (changeType) {
    case "create":
      return "text-green-500";
    case "modify":
      return "text-yellow-500";
    case "delete":
      return "text-destructive";
    case "rename":
      return "text-blue-500";
    default:
      return "text-muted-foreground";
  }
}

function capitalizeFirst(str: string): string {
  return str.charAt(0).toUpperCase() + str.slice(1);
}
