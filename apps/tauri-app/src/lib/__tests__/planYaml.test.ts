import { describe, it, expect } from "vitest";
import { parsePlanYamlToNodes } from "../planYaml";
import { UnitTaskStatus } from "@/api/types";

describe("parsePlanYamlToNodes", () => {
  it("parses a simple PLAN.yaml with one task", () => {
    const yaml = `
tasks:
  - id: "task-1"
    prompt: "Do something"
`;
    const nodes = parsePlanYamlToNodes(yaml, "composite-123");

    expect(nodes).toHaveLength(1);
    expect(nodes[0].node.id).toBe("task-1");
    expect(nodes[0].node.compositeTaskId).toBe("composite-123");
    expect(nodes[0].node.dependsOnIds).toEqual([]);
    expect(nodes[0].unitTask.prompt).toBe("Do something");
    expect(nodes[0].unitTask.title).toBe("task-1");
    expect(nodes[0].unitTask.status).toBe(UnitTaskStatus.Unspecified);
  });

  it("parses tasks with titles, branches, and dependencies", () => {
    const yaml = `
tasks:
  - id: "setup-db"
    title: "Setup Database"
    prompt: "Create database schema"
    branchName: "feature/db"

  - id: "auth-api"
    title: "Auth API"
    prompt: "Implement auth endpoints"
    dependsOn:
      - "setup-db"
`;
    const nodes = parsePlanYamlToNodes(yaml, "comp-456");

    expect(nodes).toHaveLength(2);

    expect(nodes[0].unitTask.title).toBe("Setup Database");
    expect(nodes[0].unitTask.branchName).toBe("feature/db");
    expect(nodes[0].node.dependsOnIds).toEqual([]);

    expect(nodes[1].unitTask.title).toBe("Auth API");
    expect(nodes[1].node.dependsOnIds).toEqual(["setup-db"]);
  });

  it("returns empty array for invalid YAML", () => {
    const nodes = parsePlanYamlToNodes("not valid yaml: [", "comp-789");
    expect(nodes).toEqual([]);
  });

  it("returns empty array for YAML without tasks", () => {
    const nodes = parsePlanYamlToNodes("foo: bar", "comp-000");
    expect(nodes).toEqual([]);
  });

  it("returns empty array for empty tasks array", () => {
    const nodes = parsePlanYamlToNodes("tasks: []", "comp-000");
    expect(nodes).toEqual([]);
  });

  it("sets all task statuses to Unspecified", () => {
    const yaml = `
tasks:
  - id: "a"
    prompt: "Task A"
  - id: "b"
    prompt: "Task B"
    dependsOn:
      - "a"
`;
    const nodes = parsePlanYamlToNodes(yaml, "comp-111");
    for (const node of nodes) {
      expect(node.unitTask.status).toBe(UnitTaskStatus.Unspecified);
    }
  });
});
