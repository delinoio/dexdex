import type { TokenUsage } from "@/api/types";

interface TokenUsageDisplayProps {
  tokenUsage: TokenUsage | undefined;
  className?: string;
}

// Format large numbers with thousands separators
function formatNumber(num: number): string {
  return num.toLocaleString();
}

export function TokenUsageDisplay({ tokenUsage, className = "" }: TokenUsageDisplayProps) {
  if (!tokenUsage) {
    return null;
  }

  const totalTokens = tokenUsage.inputTokens + tokenUsage.outputTokens;

  return (
    <div className={`rounded-md border border-[hsl(var(--border))] bg-[hsl(var(--muted))] p-3 ${className}`}>
      <div className="mb-2 text-sm font-medium text-[hsl(var(--foreground))]">
        Token Usage
      </div>
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="flex justify-between">
          <span className="text-[hsl(var(--muted-foreground))]">Input:</span>
          <span className="font-mono">{formatNumber(tokenUsage.inputTokens)}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-[hsl(var(--muted-foreground))]">Output:</span>
          <span className="font-mono">{formatNumber(tokenUsage.outputTokens)}</span>
        </div>
        {tokenUsage.cacheReadTokens !== undefined && tokenUsage.cacheReadTokens > 0 && (
          <div className="flex justify-between">
            <span className="text-[hsl(var(--muted-foreground))]">Cache Read:</span>
            <span className="font-mono">{formatNumber(tokenUsage.cacheReadTokens)}</span>
          </div>
        )}
        {tokenUsage.cacheCreationTokens !== undefined && tokenUsage.cacheCreationTokens > 0 && (
          <div className="flex justify-between">
            <span className="text-[hsl(var(--muted-foreground))]">Cache Write:</span>
            <span className="font-mono">{formatNumber(tokenUsage.cacheCreationTokens)}</span>
          </div>
        )}
        <div className="col-span-2 mt-1 border-t border-[hsl(var(--border))] pt-1">
          <div className="flex justify-between font-medium">
            <span className="text-[hsl(var(--muted-foreground))]">Total:</span>
            <span className="font-mono">{formatNumber(totalTokens)}</span>
          </div>
        </div>
      </div>
    </div>
  );
}
