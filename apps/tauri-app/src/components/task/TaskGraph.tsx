import { useCallback, useMemo } from "react";
import {
  ReactFlow,
  Controls,
  MiniMap,
  Background,
  useNodesState,
  useEdgesState,
  type Edge,
  type Node,
  Handle,
  Position,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";

// Node status enum matching the task status types
export enum TaskNodeStatus {
  Pending = "pending",
  InProgress = "in_progress",
  Done = "done",
  Rejected = "rejected",
}

// Task node data structure (exported for consumers)
export interface TaskNodeData {
  id: string;
  title: string;
  prompt?: string;
  status: TaskNodeStatus;
}

// Internal data structure that extends Record<string, unknown> for ReactFlow compatibility
interface TaskNodeDataInternal extends TaskNodeData {
  [key: string]: unknown;
}

// Props for TaskGraph component
export interface TaskGraphProps {
  nodes: TaskNodeData[];
  edges: Array<{ source: string; target: string }>;
  onNodeClick?: (nodeId: string) => void;
}

// Status color mapping
const statusColors: Record<TaskNodeStatus, { background: string; border: string; text: string }> = {
  [TaskNodeStatus.Pending]: {
    background: "hsl(var(--muted))",
    border: "hsl(var(--border))",
    text: "hsl(var(--muted-foreground))",
  },
  [TaskNodeStatus.InProgress]: {
    background: "hsl(217 91% 60% / 0.15)",
    border: "hsl(217 91% 60%)",
    text: "hsl(217 91% 60%)",
  },
  [TaskNodeStatus.Done]: {
    background: "hsl(142 76% 36% / 0.15)",
    border: "hsl(142 76% 36%)",
    text: "hsl(142 76% 36%)",
  },
  [TaskNodeStatus.Rejected]: {
    background: "hsl(var(--destructive) / 0.15)",
    border: "hsl(var(--destructive))",
    text: "hsl(var(--destructive))",
  },
};

// Custom node component props
interface TaskNodeComponentProps {
  data: TaskNodeDataInternal;
  selected?: boolean;
}

// Custom node component
function TaskNodeComponent({ data, selected }: TaskNodeComponentProps) {
  const colors = statusColors[data.status];

  return (
    <div
      className="rounded-lg border-2 px-4 py-3 shadow-sm transition-shadow hover:shadow-md"
      style={{
        backgroundColor: colors.background,
        borderColor: selected ? colors.text : colors.border,
        minWidth: 180,
        maxWidth: 250,
      }}
    >
      <Handle
        type="target"
        position={Position.Top}
        className="!bg-[hsl(var(--border))] !w-3 !h-3"
      />

      <div className="flex items-start justify-between gap-2">
        <div className="flex-1 min-w-0">
          <div
            className="font-medium text-sm truncate"
            style={{ color: "hsl(var(--foreground))" }}
            title={data.title}
          >
            {data.title}
          </div>
          {data.prompt && (
            <div
              className="mt-1 text-xs line-clamp-2"
              style={{ color: "hsl(var(--muted-foreground))" }}
              title={data.prompt}
            >
              {data.prompt}
            </div>
          )}
        </div>
        <StatusBadge status={data.status} />
      </div>

      <Handle
        type="source"
        position={Position.Bottom}
        className="!bg-[hsl(var(--border))] !w-3 !h-3"
      />
    </div>
  );
}

// Status badge component
function StatusBadge({ status }: { status: TaskNodeStatus }) {
  const colors = statusColors[status];

  const statusLabel: Record<TaskNodeStatus, string> = {
    [TaskNodeStatus.Pending]: "Pending",
    [TaskNodeStatus.InProgress]: "Running",
    [TaskNodeStatus.Done]: "Done",
    [TaskNodeStatus.Rejected]: "Rejected",
  };

  return (
    <span
      className="px-2 py-0.5 rounded text-xs font-medium shrink-0"
      style={{
        backgroundColor: colors.border,
        color: status === TaskNodeStatus.Pending ? "hsl(var(--foreground))" : "white",
      }}
    >
      {statusLabel[status]}
    </span>
  );
}

// Node types for ReactFlow - using 'any' to bypass strict typing with custom node data
// eslint-disable-next-line @typescript-eslint/no-explicit-any
const nodeTypes: Record<string, any> = {
  task: TaskNodeComponent,
};

// Auto-layout function using topological sort for DAG
function calculateLayout(
  nodes: TaskNodeData[],
  edges: Array<{ source: string; target: string }>
): Map<string, { x: number; y: number }> {
  const positions = new Map<string, { x: number; y: number }>();
  const nodeWidth = 220;
  const nodeHeight = 100;
  const horizontalGap = 50;
  const verticalGap = 80;

  // Build adjacency list and in-degree map
  const adjacency = new Map<string, string[]>();
  const inDegree = new Map<string, number>();

  nodes.forEach((node) => {
    adjacency.set(node.id, []);
    inDegree.set(node.id, 0);
  });

  edges.forEach((edge) => {
    const existing = adjacency.get(edge.source) ?? [];
    existing.push(edge.target);
    adjacency.set(edge.source, existing);
    inDegree.set(edge.target, (inDegree.get(edge.target) ?? 0) + 1);
  });

  // Perform topological sort with level tracking
  const levels: string[][] = [];
  let queue = nodes.filter((n) => inDegree.get(n.id) === 0).map((n) => n.id);

  while (queue.length > 0) {
    levels.push([...queue]);
    const nextQueue: string[] = [];

    for (const nodeId of queue) {
      const neighbors = adjacency.get(nodeId) ?? [];
      for (const neighbor of neighbors) {
        const newDegree = (inDegree.get(neighbor) ?? 1) - 1;
        inDegree.set(neighbor, newDegree);
        if (newDegree === 0) {
          nextQueue.push(neighbor);
        }
      }
    }

    queue = nextQueue;
  }

  // Calculate positions based on levels
  levels.forEach((level, levelIndex) => {
    const totalWidth = level.length * nodeWidth + (level.length - 1) * horizontalGap;
    const startX = -totalWidth / 2 + nodeWidth / 2;

    level.forEach((nodeId, nodeIndex) => {
      positions.set(nodeId, {
        x: startX + nodeIndex * (nodeWidth + horizontalGap),
        y: levelIndex * (nodeHeight + verticalGap),
      });
    });
  });

  return positions;
}

export function TaskGraph({ nodes: taskNodes, edges: taskEdges, onNodeClick }: TaskGraphProps) {
  // Calculate positions for nodes
  const positions = useMemo(
    () => calculateLayout(taskNodes, taskEdges),
    [taskNodes, taskEdges]
  );

  // Convert to ReactFlow nodes
  const initialNodes = useMemo(
    () =>
      taskNodes.map((node) => ({
        id: node.id,
        type: "task",
        position: positions.get(node.id) ?? { x: 0, y: 0 },
        data: { ...node } as TaskNodeDataInternal,
      })),
    [taskNodes, positions]
  );

  // Convert to ReactFlow edges
  const initialEdges: Edge[] = useMemo(
    () =>
      taskEdges.map((edge, index) => ({
        id: `edge-${index}`,
        source: edge.source,
        target: edge.target,
        animated: taskNodes.find((n) => n.id === edge.source)?.status === TaskNodeStatus.InProgress,
        style: {
          stroke: "hsl(var(--border))",
          strokeWidth: 2,
        },
        markerEnd: {
          type: "arrowclosed" as const,
          color: "hsl(var(--border))",
        },
      })),
    [taskEdges, taskNodes]
  );

  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, , onEdgesChange] = useEdgesState(initialEdges);

  const handleNodeClick = useCallback(
    (_event: React.MouseEvent, node: Node) => {
      onNodeClick?.(node.id);
    },
    [onNodeClick]
  );

  // MiniMap node color based on status
  const nodeColor = useCallback((node: Node) => {
    const data = node.data as TaskNodeDataInternal;
    const status = data.status;
    switch (status) {
      case TaskNodeStatus.InProgress:
        return "hsl(217 91% 60%)";
      case TaskNodeStatus.Done:
        return "hsl(142 76% 36%)";
      case TaskNodeStatus.Rejected:
        return "hsl(var(--destructive))";
      default:
        return "hsl(var(--muted-foreground))";
    }
  }, []);

  if (taskNodes.length === 0) {
    return (
      <div className="flex h-64 items-center justify-center rounded-md border border-dashed border-[hsl(var(--border))] bg-[hsl(var(--muted))]">
        <p className="text-sm text-[hsl(var(--muted-foreground))]">
          No tasks in the plan yet
        </p>
      </div>
    );
  }

  return (
    <div className="h-80 w-full rounded-md border border-[hsl(var(--border))] overflow-hidden">
      <ReactFlow
        nodes={nodes}
        edges={edges}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        onNodeClick={handleNodeClick}
        nodeTypes={nodeTypes}
        fitView
        fitViewOptions={{ padding: 0.2 }}
        minZoom={0.5}
        maxZoom={2}
        attributionPosition="bottom-left"
      >
        <Background color="hsl(var(--border))" gap={16} />
        <Controls
          showInteractive={false}
          className="!bg-[hsl(var(--card))] !border-[hsl(var(--border))] !shadow-md [&>button]:!bg-[hsl(var(--card))] [&>button]:!border-[hsl(var(--border))] [&>button:hover]:!bg-[hsl(var(--muted))] [&>button>svg]:!fill-[hsl(var(--foreground))]"
        />
        <MiniMap
          nodeColor={nodeColor}
          className="!bg-[hsl(var(--card))] !border-[hsl(var(--border))]"
          maskColor="hsl(var(--background) / 0.6)"
        />
      </ReactFlow>
    </div>
  );
}
