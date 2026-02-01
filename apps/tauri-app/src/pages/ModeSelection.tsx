import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { useSetMode } from "@/hooks/useMode";
import { cn } from "@/lib/utils";

type Mode = "local" | "remote";

export function ModeSelection() {
  const [selectedMode, setSelectedMode] = useState<Mode | null>(null);
  const [serverUrl, setServerUrl] = useState("");
  const [isConnecting, setIsConnecting] = useState(false);
  const [connectionError, setConnectionError] = useState<string | null>(null);
  const navigate = useNavigate();
  const setModeMutation = useSetMode();

  const handleContinue = async () => {
    if (!selectedMode) return;

    setIsConnecting(true);
    setConnectionError(null);

    try {
      await setModeMutation.mutateAsync({
        mode: selectedMode,
        serverUrl: selectedMode === "remote" ? serverUrl : undefined,
      });
      navigate("/onboarding");
    } catch (error) {
      setConnectionError(
        error instanceof Error ? error.message : "Failed to set mode"
      );
    } finally {
      setIsConnecting(false);
    }
  };

  const testConnection = async () => {
    if (!serverUrl) return;

    setIsConnecting(true);
    setConnectionError(null);

    try {
      const response = await fetch(`${serverUrl}/health`);
      if (!response.ok) {
        throw new Error(`Server returned status ${response.status}`);
      }
      setConnectionError(null);
    } catch (error) {
      setConnectionError(
        error instanceof Error ? error.message : "Connection failed"
      );
    } finally {
      setIsConnecting(false);
    }
  };

  return (
    <div className="flex min-h-screen flex-col items-center justify-center p-8">
      <div className="w-full max-w-2xl space-y-8">
        <div className="text-center">
          <h1 className="text-3xl font-bold">Welcome to DeliDev</h1>
          <p className="mt-2 text-[hsl(var(--muted-foreground))]">
            Choose how you want to run DeliDev
          </p>
        </div>

        <div className="space-y-4">
          <Card
            className={cn(
              "cursor-pointer transition-all",
              selectedMode === "local" &&
                "border-[hsl(var(--primary))] ring-2 ring-[hsl(var(--primary))]"
            )}
            onClick={() => setSelectedMode("local")}
          >
            <CardHeader>
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-[hsl(var(--muted))] p-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect width="20" height="14" x="2" y="3" rx="2" />
                    <line x1="8" x2="16" y1="21" y2="21" />
                    <line x1="12" x2="12" y1="17" y2="21" />
                  </svg>
                </div>
                <div>
                  <CardTitle>Local Mode</CardTitle>
                  <CardDescription>
                    Run everything locally on your machine
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-[hsl(var(--muted-foreground))]">
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Full privacy - your code never leaves your machine
              </div>
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                No network latency
              </div>
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Works offline (requires local AI setup)
              </div>
            </CardContent>
          </Card>

          <Card
            className={cn(
              "cursor-pointer transition-all",
              selectedMode === "remote" &&
                "border-[hsl(var(--primary))] ring-2 ring-[hsl(var(--primary))]"
            )}
            onClick={() => setSelectedMode("remote")}
          >
            <CardHeader>
              <div className="flex items-center gap-3">
                <div className="rounded-lg bg-[hsl(var(--muted))] p-2">
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="24"
                    height="24"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    strokeWidth="2"
                    strokeLinecap="round"
                    strokeLinejoin="round"
                  >
                    <rect width="20" height="8" x="2" y="2" rx="2" ry="2" />
                    <rect width="20" height="8" x="2" y="14" rx="2" ry="2" />
                    <line x1="6" x2="6.01" y1="6" y2="6" />
                    <line x1="6" x2="6.01" y1="18" y2="18" />
                  </svg>
                </div>
                <div>
                  <CardTitle>Remote Mode</CardTitle>
                  <CardDescription>
                    Connect to a remote DeliDev server
                  </CardDescription>
                </div>
              </div>
            </CardHeader>
            <CardContent className="space-y-2 text-sm text-[hsl(var(--muted-foreground))]">
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Centralized task management
              </div>
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Team collaboration support
              </div>
              <div className="flex items-center gap-2">
                <svg
                  className="h-4 w-4 text-green-500"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
                Offload computation to server
              </div>
            </CardContent>
          </Card>
        </div>

        {selectedMode === "remote" && (
          <div className="space-y-2">
            <label className="text-sm font-medium">Server URL</label>
            <div className="flex gap-2">
              <Input
                placeholder="https://your-server.com"
                value={serverUrl}
                onChange={(e) => setServerUrl(e.target.value)}
              />
              <Button
                variant="outline"
                onClick={testConnection}
                disabled={!serverUrl || isConnecting}
              >
                Test
              </Button>
            </div>
            {connectionError && (
              <p className="text-sm text-[hsl(var(--destructive))]">
                {connectionError}
              </p>
            )}
          </div>
        )}

        <div className="flex items-center justify-between border-t border-[hsl(var(--border))] pt-4">
          <p className="text-sm text-[hsl(var(--muted-foreground))]">
            You can change this setting later in Settings
          </p>
          <Button
            onClick={handleContinue}
            disabled={!selectedMode || isConnecting}
          >
            {isConnecting ? "Connecting..." : "Continue →"}
          </Button>
        </div>
      </div>
    </div>
  );
}
