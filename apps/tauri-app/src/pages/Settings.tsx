import { useState } from "react";
import { Button } from "@/components/ui/Button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/Card";
import { Input } from "@/components/ui/Input";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/Tabs";
import { useTheme } from "@/hooks/useTheme";
import { ThemeMode } from "@/stores/themeStore";
import { useWorkspaces, useCreateWorkspace, useUpdateWorkspace } from "@/api/hooks/useWorkspaces";
import { useUiStore } from "@/stores/uiStore";

export function Settings() {
  const { data: workspacesData, isLoading: workspacesLoading } = useWorkspaces();
  const createWorkspace = useCreateWorkspace();
  const updateWorkspace = useUpdateWorkspace();
  const currentWorkspaceId = useUiStore((s) => s.currentWorkspaceId);
  const setCurrentWorkspaceId = useUiStore((s) => s.setCurrentWorkspaceId);
  const { mode: themeMode, setMode: setThemeMode, resolvedTheme } = useTheme();

  const [newWorkspaceName, setNewWorkspaceName] = useState("");
  const [newEndpointUrl, setNewEndpointUrl] = useState("");
  const [editingWorkspaceId, setEditingWorkspaceId] = useState<string | null>(null);
  const [editName, setEditName] = useState("");
  const [editEndpointUrl, setEditEndpointUrl] = useState("");

  const workspaces = workspacesData?.workspaces ?? [];

  const handleCreateWorkspace = async () => {
    if (!newWorkspaceName.trim()) return;
    const result = await createWorkspace.mutateAsync({
      name: newWorkspaceName.trim(),
      endpointUrl: newEndpointUrl.trim() || undefined,
    });
    setNewWorkspaceName("");
    setNewEndpointUrl("");
    if (result.workspace && !currentWorkspaceId) {
      setCurrentWorkspaceId(result.workspace.id);
    }
  };

  const handleSaveWorkspace = async () => {
    if (!editingWorkspaceId) return;
    await updateWorkspace.mutateAsync({
      workspaceId: editingWorkspaceId,
      name: editName.trim() || undefined,
      endpointUrl: editEndpointUrl.trim() || undefined,
    });
    setEditingWorkspaceId(null);
  };

  return (
    <div className="flex h-full flex-col">
      <div className="border-b border-[hsl(var(--border))] px-6 py-4">
        <h1 className="text-2xl font-bold">Settings</h1>
      </div>

      <div className="flex-1 overflow-y-auto p-6">
        <div className="mx-auto max-w-2xl space-y-6">
          <Tabs defaultValue="appearance">
            <TabsList className="mb-6">
              <TabsTrigger value="appearance">Appearance</TabsTrigger>
              <TabsTrigger value="workspaces">Workspaces</TabsTrigger>
            </TabsList>

            <TabsContent value="appearance" className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Theme</CardTitle>
                  <CardDescription>
                    Choose your preferred color theme.
                  </CardDescription>
                </CardHeader>
                <CardContent>
                  <div className="flex gap-3">
                    {[
                      { value: ThemeMode.Light, label: "Light" },
                      { value: ThemeMode.Dark, label: "Dark" },
                      { value: ThemeMode.System, label: "System" },
                    ].map((option) => (
                      <button
                        key={option.value}
                        onClick={() => setThemeMode(option.value)}
                        className={`rounded-md border px-4 py-2 text-sm font-medium transition-colors ${
                          themeMode === option.value
                            ? "border-[hsl(var(--primary))] bg-[hsl(var(--primary))] text-[hsl(var(--primary-foreground))]"
                            : "border-[hsl(var(--border))] bg-[hsl(var(--background))] text-[hsl(var(--foreground))] hover:bg-[hsl(var(--muted))]"
                        }`}
                      >
                        {option.label}
                      </button>
                    ))}
                  </div>
                  <p className="mt-2 text-xs text-[hsl(var(--muted-foreground))]">
                    Currently using {resolvedTheme} theme
                    {themeMode === ThemeMode.System ? " (based on system preference)" : ""}
                  </p>
                </CardContent>
              </Card>
            </TabsContent>

            <TabsContent value="workspaces" className="space-y-6">
              <Card>
                <CardHeader>
                  <CardTitle>Workspaces</CardTitle>
                  <CardDescription>
                    Manage workspaces and server connections.
                  </CardDescription>
                </CardHeader>
                <CardContent className="space-y-4">
                  {workspacesLoading ? (
                    <p className="text-sm text-[hsl(var(--muted-foreground))]">Loading...</p>
                  ) : (
                    <div className="space-y-2">
                      {workspaces.map((ws) =>
                        editingWorkspaceId === ws.id ? (
                          <div
                            key={ws.id}
                            className="rounded-md border border-[hsl(var(--border))] p-3 space-y-2"
                          >
                            <Input
                              placeholder="Workspace name"
                              value={editName}
                              onChange={(e) => setEditName(e.target.value)}
                            />
                            <Input
                              placeholder="Endpoint URL (e.g. http://localhost:3000)"
                              value={editEndpointUrl}
                              onChange={(e) => setEditEndpointUrl(e.target.value)}
                            />
                            <div className="flex gap-2">
                              <Button
                                size="sm"
                                onClick={handleSaveWorkspace}
                                disabled={updateWorkspace.isPending}
                              >
                                {updateWorkspace.isPending ? "Saving..." : "Save"}
                              </Button>
                              <Button
                                size="sm"
                                variant="outline"
                                onClick={() => setEditingWorkspaceId(null)}
                              >
                                Cancel
                              </Button>
                            </div>
                          </div>
                        ) : (
                          <div
                            key={ws.id}
                            className="flex items-center justify-between rounded-md border border-[hsl(var(--border))] p-3"
                          >
                            <div>
                              <p className="text-sm font-medium">{ws.name}</p>
                              {ws.endpointUrl && (
                                <p className="text-xs text-[hsl(var(--muted-foreground))]">
                                  {ws.endpointUrl}
                                </p>
                              )}
                            </div>
                            <div className="flex items-center gap-2">
                              {currentWorkspaceId === ws.id ? (
                                <span className="text-xs text-[hsl(var(--primary))]">Active</span>
                              ) : (
                                <Button
                                  size="sm"
                                  variant="outline"
                                  onClick={() => setCurrentWorkspaceId(ws.id)}
                                >
                                  Select
                                </Button>
                              )}
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => {
                                  setEditingWorkspaceId(ws.id);
                                  setEditName(ws.name);
                                  setEditEndpointUrl(ws.endpointUrl ?? "");
                                }}
                              >
                                Edit
                              </Button>
                            </div>
                          </div>
                        )
                      )}
                    </div>
                  )}

                  <div className="space-y-2 rounded-md border border-dashed border-[hsl(var(--border))] p-3">
                    <p className="text-sm font-medium">Add Workspace</p>
                    <Input
                      placeholder="Workspace name"
                      value={newWorkspaceName}
                      onChange={(e) => setNewWorkspaceName(e.target.value)}
                    />
                    <Input
                      placeholder="Endpoint URL (e.g. http://localhost:3000)"
                      value={newEndpointUrl}
                      onChange={(e) => setNewEndpointUrl(e.target.value)}
                    />
                    <Button
                      size="sm"
                      onClick={handleCreateWorkspace}
                      disabled={!newWorkspaceName.trim() || createWorkspace.isPending}
                    >
                      {createWorkspace.isPending ? "Creating..." : "Add Workspace"}
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </TabsContent>
          </Tabs>
        </div>
      </div>
    </div>
  );
}
