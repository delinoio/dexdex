import { useMemo } from "react";
import {
  ReactFlow,
  Controls,
  MiniMap,
  Background,
  useNodesState,
  useEdgesState,
  type Node,
  type Edge,
  type NodeTypes,
  Handle,
  Position,
  BackgroundVariant,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { cn } from "@/lib/utils";
import type {
  CompositeTaskNodeWithUnitTask,
  UnitTaskStatus,
} from "@/api/types";

// Layout configuration constants
const LAYOUT_CONFIG = {
  /** Horizontal spacing between nodes at different levels */
  HORIZONTAL_SPACING: 280,
  /** Vertical spacing between nodes at the same level */
  VERTICAL_SPACING: 120,
  /** Minimum width for task nodes */
  NODE_MIN_WIDTH: 160,
  /** Maximum width for task nodes */
  NODE_MAX_WIDTH: 220,
} as const;

// Node status colors based on issue requirements
const STATUS_COLORS: Record<
  UnitTaskStatus | "pending",
  { bg: string; border: string; text: string }
> = {
  pending: {
    bg: "hsl(var(--muted))",
    border: "hsl(var(--border))",
    text: "hsl(var(--muted-foreground))",
  },
  unspecified: {
    bg: "hsl(var(--muted))",
    border: "hsl(var(--border))",
    text: "hsl(var(--muted-foreground))",
  },
  in_progress: {
    bg: "hsl(217 91% 60% / 0.1)",
    border: "hsl(217 91% 60%)",
    text: "hsl(217 91% 50%)",
  },
  in_review: {
    bg: "hsl(217 91% 60% / 0.1)",
    border: "hsl(217 91% 60%)",
    text: "hsl(217 91% 50%)",
  },
  approved: {
    bg: "hsl(142 76% 36% / 0.1)",
    border: "hsl(142 76% 36%)",
    text: "hsl(142 76% 30%)",
  },
  pr_open: {
    bg: "hsl(142 76% 36% / 0.1)",
    border: "hsl(142 76% 36%)",
    text: "hsl(142 76% 30%)",
  },
  done: {
    bg: "hsl(142 76% 36% / 0.1)",
    border: "hsl(142 76% 36%)",
    text: "hsl(142 76% 30%)",
  },
  rejected: {
    bg: "hsl(0 84% 60% / 0.1)",
    border: "hsl(0 84% 60%)",
    text: "hsl(0 84% 45%)",
  },
};

interface TaskNodeData extends Record<string, unknown> {
  label: string;
  status: UnitTaskStatus | "pending";
  prompt: string;
}

interface TaskNodeProps {
  data: TaskNodeData;
  selected?: boolean;
}

function TaskNode({ data, selected }: TaskNodeProps) {
  const colors = STATUS_COLORS[data.status] || STATUS_COLORS.pending;

  return (
    <div
      className={cn(
        "rounded-lg px-4 py-3 shadow-sm min-w-[160px] max-w-[220px] transition-all",
        selected && "ring-2 ring-[hsl(var(--primary))] ring-offset-2"
      )}
      style={{
        backgroundColor: colors.bg,
        borderWidth: "2px",
        borderStyle: "solid",
        borderColor: colors.border,
      }}
    >
      <Handle
        type="target"
        position={Position.Left}
        className="!w-3 !h-3 !bg-[hsl(var(--border))]"
      />
      <div className="space-y-1">
        <div
          className="font-semibold text-sm truncate"
          style={{ color: colors.text }}
        >
          {data.label}
        </div>
        <div className="text-xs text-[hsl(var(--muted-foreground))] line-clamp-2">
          {data.prompt}
        </div>
        <div
          className="text-xs font-medium capitalize"
          style={{ color: colors.text }}
        >
          {data.status.replaceAll("_", " ")}
        </div>
      </div>
      <Handle
        type="source"
        position={Position.Right}
        className="!w-3 !h-3 !bg-[hsl(var(--border))]"
      />
    </div>
  );
}

const nodeTypes: NodeTypes = {
  taskNode: TaskNode,
};

interface TaskGraphProps {
  nodes: CompositeTaskNodeWithUnitTask[];
  className?: string;
}

export function TaskGraph({ nodes: taskNodes, className }: TaskGraphProps) {
  // Convert task nodes to React Flow format
  const { initialNodes, initialEdges } = useMemo(() => {
    const nodeMap = new Map<string, CompositeTaskNodeWithUnitTask>();
    taskNodes.forEach((n) => nodeMap.set(n.node.id, n));

    // Use a simple layout algorithm
    // Group nodes by their dependency level
    const levels = new Map<string, number>();

    function calculateLevel(nodeId: string, visiting: Set<string>): number {
      if (levels.has(nodeId)) {
        return levels.get(nodeId)!;
      }

      // Detect circular dependency - if we're already visiting this node, we have a cycle
      if (visiting.has(nodeId)) {
        console.warn(`Circular dependency detected involving node: ${nodeId}`);
        // Return 0 to break the cycle and prevent infinite recursion
        levels.set(nodeId, 0);
        return 0;
      }

      const node = nodeMap.get(nodeId);
      if (!node || node.node.dependsOnIds.length === 0) {
        levels.set(nodeId, 0);
        return 0;
      }

      // Mark this node as being visited
      visiting.add(nodeId);

      let maxParentLevel = -1;
      for (const depId of node.node.dependsOnIds) {
        if (nodeMap.has(depId)) {
          maxParentLevel = Math.max(maxParentLevel, calculateLevel(depId, visiting));
        }
      }

      // Remove from visiting set after processing
      visiting.delete(nodeId);

      const level = maxParentLevel + 1;
      levels.set(nodeId, level);
      return level;
    }

    // Calculate levels for all nodes
    taskNodes.forEach((n) => calculateLevel(n.node.id, new Set()));

    // Group nodes by level
    const nodesByLevel = new Map<number, CompositeTaskNodeWithUnitTask[]>();
    taskNodes.forEach((n) => {
      const level = levels.get(n.node.id) || 0;
      if (!nodesByLevel.has(level)) {
        nodesByLevel.set(level, []);
      }
      nodesByLevel.get(level)!.push(n);
    });

    // Create React Flow nodes with positions
    const flowNodes: Node<TaskNodeData>[] = [];
    const { HORIZONTAL_SPACING, VERTICAL_SPACING } = LAYOUT_CONFIG;

    nodesByLevel.forEach((nodesInLevel, level) => {
      const startY =
        -(nodesInLevel.length - 1) * VERTICAL_SPACING * 0.5;

      nodesInLevel.forEach((node, index) => {
        const status = node.unitTask?.status || "pending";
        flowNodes.push({
          id: node.node.id,
          type: "taskNode",
          position: {
            x: level * HORIZONTAL_SPACING,
            y: startY + index * VERTICAL_SPACING,
          },
          data: {
            label: node.unitTask?.title || `Task ${node.node.id.slice(0, 8)}`,
            status: status as UnitTaskStatus | "pending",
            prompt: node.unitTask?.prompt || "",
          },
        });
      });
    });

    // Create edges from dependencies
    const flowEdges: Edge[] = [];
    taskNodes.forEach((node) => {
      node.node.dependsOnIds.forEach((depId) => {
        if (nodeMap.has(depId)) {
          flowEdges.push({
            id: `${depId}-${node.node.id}`,
            source: depId,
            target: node.node.id,
            animated: true,
            style: {
              stroke: "hsl(var(--border))",
              strokeWidth: 2,
            },
          });
        }
      });
    });

    return { initialNodes: flowNodes, initialEdges: flowEdges };
  }, [taskNodes]);

  const [flowNodes, , onNodesChange] = useNodesState(initialNodes);
  const [flowEdges, , onEdgesChange] = useEdgesState(initialEdges);

  if (taskNodes.length === 0) {
    return (
      <div
        className={cn(
          "flex h-64 items-center justify-center rounded-md border border-dashed border-[hsl(var(--border))] bg-[hsl(var(--muted))]",
          className
        )}
        role="status"
        aria-label="Task graph is empty"
      >
        <p className="text-sm text-[hsl(var(--muted-foreground))]">
          No tasks in the graph yet. The plan is still being generated.
        </p>
      </div>
    );
  }

  return (
    <div
      className={cn("h-96 w-full rounded-md border border-[hsl(var(--border))]", className)}
      role="img"
      aria-label={`Task dependency graph showing ${taskNodes.length} tasks and their relationships`}
    >
      <ReactFlow
        nodes={flowNodes}
        edges={flowEdges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={0.5}
        maxZoom={1.5}
        proOptions={{ hideAttribution: true }}
      >
        <Controls
          showInteractive={false}
          className="!bg-[hsl(var(--background))] !border-[hsl(var(--border))] !shadow-sm [&>button]:!bg-[hsl(var(--background))] [&>button]:!border-[hsl(var(--border))] [&>button:hover]:!bg-[hsl(var(--muted))]"
        />
        <MiniMap
          nodeColor={(node) => {
            const data = node.data as unknown as TaskNodeData;
            const colors = STATUS_COLORS[data.status] || STATUS_COLORS.pending;
            return colors.border;
          }}
          maskColor="hsl(var(--background) / 0.8)"
          className="!bg-[hsl(var(--muted))] !border-[hsl(var(--border))]"
        />
        <Background
          variant={BackgroundVariant.Dots}
          gap={16}
          size={1}
          color="hsl(var(--border))"
        />
      </ReactFlow>
    </div>
  );
}
