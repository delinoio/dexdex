// Utility for parsing PLAN.yaml content into a task preview format.
//
// During the plan approval phase, tasks don't exist in the database yet
// (they're created on approval). This module parses the raw plan_yaml
// string to produce a preview-compatible data structure.

import yaml from "js-yaml";
import type { UnitTask } from "@/api/types";

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

export interface PlanTaskNode {
  node: {
    id: string;
    dependsOnIds: string[];
  };
  unitTask: Omit<UnitTask, 'workspaceId' | 'repositoryGroupId' | 'actionTypes' | 'prTrackingIds' | 'latestCommitSha' | 'generatedCommitCount' | 'latestPatchRef'>;
}

/**
 * Parses raw PLAN.yaml content and converts it into PlanTaskNode[]
 * for rendering a preview before tasks are created.
 *
 * All tasks are given a 'queued' status since they haven't started execution yet.
 */
export function parsePlanYamlToNodes(
  planYaml: string,
  _compositeTaskId: string
): PlanTaskNode[] {
  try {
    const parsed = yaml.load(planYaml, { schema: yaml.JSON_SCHEMA }) as PlanYaml;
    if (!parsed?.tasks || !Array.isArray(parsed.tasks) || parsed.tasks.length === 0) {
      return [];
    }

    return parsed.tasks.map((task) => ({
      node: {
        id: task.id,
        dependsOnIds: task.dependsOn ?? [],
      },
      unitTask: {
        id: task.id,
        prompt: task.prompt,
        title: task.title ?? task.id,
        branchName: task.branchName,
        status: 'queued' as const,
        createdAt: new Date().toISOString(),
        updatedAt: new Date().toISOString(),
      },
    }));
  } catch (e) {
    console.warn("Failed to parse PLAN.yaml content:", e);
    return [];
  }
}
