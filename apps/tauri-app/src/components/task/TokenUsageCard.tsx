import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import type { TokenUsage } from "@/api/types";

interface TokenUsageCardProps {
  tokenUsage: TokenUsage | null | undefined;
  className?: string;
  title?: string;
  description?: string;
}

function formatNumber(n: number): string {
  return n.toLocaleString();
}

function formatCost(usd: number): string {
  if (usd < 0.01) {
    return `$${usd.toFixed(4)}`;
  }
  return `$${usd.toFixed(2)}`;
}

function formatDuration(ms: number): string {
  if (ms < 1000) {
    return `${ms}ms`;
  }
  const seconds = ms / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = Math.round(seconds % 60);
  return `${minutes}m ${remainingSeconds}s`;
}

export function TokenUsageCard({ tokenUsage, className, title = "Token Usage", description }: TokenUsageCardProps) {
  if (!tokenUsage) return null;

  const hasCacheTokens =
    tokenUsage.cacheReadInputTokens > 0 ||
    tokenUsage.cacheCreationInputTokens > 0;

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="text-base">{title}</CardTitle>
        {description && <CardDescription>{description}</CardDescription>}
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
          <div>
            <div className="text-[hsl(var(--muted-foreground))]">Cost</div>
            <div className="font-medium text-[hsl(var(--primary))]">
              {formatCost(tokenUsage.totalCostUsd)}
            </div>
          </div>
          <div>
            <div className="text-[hsl(var(--muted-foreground))]">Duration</div>
            <div className="font-medium">
              {formatDuration(tokenUsage.durationMs)}
            </div>
          </div>
          <div>
            <div className="text-[hsl(var(--muted-foreground))]">
              Input Tokens
            </div>
            <div className="font-medium">
              {formatNumber(tokenUsage.inputTokens)}
            </div>
          </div>
          <div>
            <div className="text-[hsl(var(--muted-foreground))]">
              Output Tokens
            </div>
            <div className="font-medium">
              {formatNumber(tokenUsage.outputTokens)}
            </div>
          </div>
        </div>

        {hasCacheTokens && (
          <div className="mt-4 grid grid-cols-2 gap-4 text-sm sm:grid-cols-4">
            <div>
              <div className="text-[hsl(var(--muted-foreground))]">
                Cache Read
              </div>
              <div className="font-medium text-[hsl(var(--success))]">
                {formatNumber(tokenUsage.cacheReadInputTokens)}
              </div>
            </div>
            <div>
              <div className="text-[hsl(var(--muted-foreground))]">
                Cache Write
              </div>
              <div className="font-medium">
                {formatNumber(tokenUsage.cacheCreationInputTokens)}
              </div>
            </div>
            <div>
              <div className="text-[hsl(var(--muted-foreground))]">Turns</div>
              <div className="font-medium">{tokenUsage.numTurns}</div>
            </div>
          </div>
        )}

        {!hasCacheTokens && tokenUsage.numTurns > 0 && (
          <div className="mt-4 text-sm">
            <span className="text-[hsl(var(--muted-foreground))]">Turns: </span>
            <span className="font-medium">{tokenUsage.numTurns}</span>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * Aggregates multiple TokenUsage objects into a single total.
 */
export function aggregateTokenUsage(
  usages: (TokenUsage | null | undefined)[]
): TokenUsage | null {
  const validUsages = usages.filter(
    (u): u is TokenUsage => u !== null && u !== undefined
  );
  if (validUsages.length === 0) return null;

  return validUsages.reduce(
    (acc, usage) => ({
      inputTokens: acc.inputTokens + usage.inputTokens,
      outputTokens: acc.outputTokens + usage.outputTokens,
      cacheReadInputTokens:
        acc.cacheReadInputTokens + usage.cacheReadInputTokens,
      cacheCreationInputTokens:
        acc.cacheCreationInputTokens + usage.cacheCreationInputTokens,
      totalCostUsd: acc.totalCostUsd + usage.totalCostUsd,
      durationMs: acc.durationMs + usage.durationMs,
      numTurns: acc.numTurns + usage.numTurns,
    }),
    {
      inputTokens: 0,
      outputTokens: 0,
      cacheReadInputTokens: 0,
      cacheCreationInputTokens: 0,
      totalCostUsd: 0,
      durationMs: 0,
      numTurns: 0,
    }
  );
}
