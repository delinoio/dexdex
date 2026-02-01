import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/Tabs";
import { useMode, useSetMode } from "@/hooks/useMode";
import { AiAgentType } from "@/api/types";

export function Settings() {
  const navigate = useNavigate();
  const { data: currentMode } = useMode();
  const setModeMutation = useSetMode();

  const [mode, setMode] = useState<"local" | "remote">(
    (currentMode as "local" | "remote") ?? "local"
  );
  const [serverUrl, setServerUrl] = useState("");
  const [hotkey, setHotkey] = useState("Option+Z");
  const [planningAgent, setPlanningAgent] = useState(AiAgentType.ClaudeCode);
  const [executionAgent, setExecutionAgent] = useState(AiAgentType.ClaudeCode);
  const [chatAgent, setChatAgent] = useState(AiAgentType.ClaudeCode);

  const handleSave = async () => {
    try {
      await setModeMutation.mutateAsync({
        mode,
        serverUrl: mode === "remote" ? serverUrl : undefined,
      });
    } catch (error) {
      console.error("Failed to save settings:", error);
    }
  };

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <h1 className="text-2xl font-bold">Settings</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl">
          <Tabs defaultValue="global">
            <TabsList className="mb-6">
              <TabsTrigger value="global">Global</TabsTrigger>
              <TabsTrigger value="workspace">Workspace</TabsTrigger>
              <TabsTrigger value="connection">Connection</TabsTrigger>
            </TabsList>

            <TabsContent value="global" className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Hotkey</CardTitle>
                  <CardDescription>
                    Configure keyboard shortcuts
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Open Chat</label>
                    <Input
                      value={hotkey}
                      onChange={(e) => setHotkey(e.target.value)}
                      placeholder="Option+Z"
                    />
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Agent - Planning</CardTitle>
                  <CardDescription>
                    Configure the AI agent used for task planning
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Agent Type</label>
                    <Select
                      value={planningAgent}
                      onChange={(e) =>
                        setPlanningAgent(e.target.value as AiAgentType)
                      }
                    >
                      <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                      <option value={AiAgentType.OpenCode}>OpenCode</option>
                      <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                    </Select>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Agent - Execution</CardTitle>
                  <CardDescription>
                    Configure the AI agent used for task execution
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Agent Type</label>
                    <Select
                      value={executionAgent}
                      onChange={(e) =>
                        setExecutionAgent(e.target.value as AiAgentType)
                      }
                    >
                      <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                      <option value={AiAgentType.OpenCode}>OpenCode</option>
                      <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                      <option value={AiAgentType.Aider}>Aider</option>
                      <option value={AiAgentType.Amp}>Amp</option>
                    </Select>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Agent - Chat</CardTitle>
                  <CardDescription>
                    Configure the AI agent used for chat interactions
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="space-y-2">
                    <label className="text-sm font-medium">Agent Type</label>
                    <Select
                      value={chatAgent}
                      onChange={(e) =>
                        setChatAgent(e.target.value as AiAgentType)
                      }
                    >
                      <option value={AiAgentType.ClaudeCode}>Claude Code</option>
                      <option value={AiAgentType.OpenCode}>OpenCode</option>
                      <option value={AiAgentType.GeminiCli}>Gemini CLI</option>
                    </Select>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="workspace" className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Branch Template</CardTitle>
                  <CardDescription>
                    Template for auto-generated branch names
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <Input
                    defaultValue="feature/${taskId}-${slug}"
                    placeholder="feature/${taskId}-${slug}"
                  />
                  <p className="mt-2 text-xs text-[hsl(var(--muted-foreground))]">
                    Available variables: {"{taskId}"}, {"{slug}"}
                  </p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle>Automation</CardTitle>
                  <CardDescription>
                    Configure automatic task behaviors
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium">Auto-fix review comments</p>
                      <p className="text-xs text-[hsl(var(--muted-foreground))]">
                        Automatically create tasks to address review feedback
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      defaultChecked
                      className="h-4 w-4 rounded border-[hsl(var(--input))]"
                    />
                  </div>

                  <div className="flex items-center justify-between">
                    <div>
                      <p className="text-sm font-medium">Auto-fix CI failures</p>
                      <p className="text-xs text-[hsl(var(--muted-foreground))]">
                        Automatically create tasks to fix CI issues
                      </p>
                    </div>
                    <input
                      type="checkbox"
                      defaultChecked
                      className="h-4 w-4 rounded border-[hsl(var(--input))]"
                    />
                  </div>

                  <div className="space-y-2">
                    <label className="text-sm font-medium">
                      Max auto-fix attempts
                    </label>
                    <Input type="number" defaultValue={3} min={1} max={10} />
                  </div>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="connection" className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Mode</CardTitle>
                  <CardDescription>
                    Choose between local and remote execution
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="flex gap-4">
                    <label className="flex items-center gap-2">
                      <input
                        type="radio"
                        name="mode"
                        checked={mode === "local"}
                        onChange={() => setMode("local")}
                        className="h-4 w-4"
                      />
                      <span className="text-sm font-medium">Local Mode</span>
                    </label>
                    <label className="flex items-center gap-2">
                      <input
                        type="radio"
                        name="mode"
                        checked={mode === "remote"}
                        onChange={() => setMode("remote")}
                        className="h-4 w-4"
                      />
                      <span className="text-sm font-medium">Remote Mode</span>
                    </label>
                  </div>

                  {mode === "remote" && (
                    <div className="space-y-2">
                      <label className="text-sm font-medium">Server URL</label>
                      <div className="flex gap-2">
                        <Input
                          value={serverUrl}
                          onChange={(e) => setServerUrl(e.target.value)}
                          placeholder="https://your-server.com"
                        />
                        <Button variant="outline">Test Connection</Button>
                      </div>
                    </div>
                  )}

                  <p className="text-xs text-[hsl(var(--muted-foreground))]">
                    Note: Changing mode requires restarting the application.
                  </p>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>

          <div className="mt-6 flex justify-end gap-2">
            <Button variant="outline" onClick={() => navigate(-1)}>
              Cancel
            </Button>
            <Button onClick={handleSave} disabled={setModeMutation.isPending}>
              {setModeMutation.isPending ? "Saving..." : "Save"}
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
