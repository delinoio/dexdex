// Utility for parsing PLAN.yaml content into a task graph preview format.
//
// During the PendingApproval phase, CompositeTaskNode records don't exist in the
// database yet (they're created on approval). This module parses the raw plan_yaml
// string to produce a preview-compatible data structure for the TaskGraph component.

import yaml from "js-yaml";
import type { CompositeTaskNodeWithUnitTask } from "@/api/types";
import { UnitTaskStatus } from "@/api/types";

interface PlanYamlTask {
  id: string;
  title?: string;
  prompt: string;
  branchName?: string;
  dependsOn?: string[];
}

interface PlanYaml {
  tasks: PlanYamlTask[];
}

/**
 * Parses raw PLAN.yaml content and converts it into CompositeTaskNodeWithUnitTask[]
 * for rendering in the TaskGraph component.
 *
 * All tasks are given an "unspecified" status since they haven't started execution yet.
 */
export function parsePlanYamlToNodes(
  planYaml: string,
  compositeTaskId: string
): CompositeTaskNodeWithUnitTask[] {
  try {
    const parsed = yaml.load(planYaml, { schema: yaml.JSON_SCHEMA }) as PlanYaml;
    if (!parsed?.tasks || !Array.isArray(parsed.tasks)) {
      return [];
    }

    return parsed.tasks.map((task) => ({
      node: {
        id: task.id,
        compositeTaskId,
        unitTaskId: task.id,
        dependsOnIds: task.dependsOn ?? [],
        createdAt: new Date().toISOString(),
      },
      unitTask: {
        id: task.id,
        repositoryGroupId: "",
        agentTaskId: "",
        prompt: task.prompt,
        title: task.title ?? task.id,
        branchName: task.branchName,
        autoFixTaskIds: [],
        status: UnitTaskStatus.Unspecified,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    }));
  } catch (e) {
    console.warn("Failed to parse PLAN.yaml content:", e);
    return [];
  }
}
