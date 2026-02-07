import { describe, it, expect } from "vitest";
import { parseUnifiedDiff } from "../parseDiff";
import { DiffFileStatus, DiffLineType } from "@/components/review/DiffViewer";

describe("parseUnifiedDiff", () => {
  it("returns empty array for empty string", () => {
    expect(parseUnifiedDiff("")).toEqual([]);
  });

  it("returns empty array for non-diff content", () => {
    expect(parseUnifiedDiff("not a diff")).toEqual([]);
  });

  it("parses a simple modified file diff", () => {
    const patch = `diff --git a/src/main.ts b/src/main.ts
index abc1234..def5678 100644
--- a/src/main.ts
+++ b/src/main.ts
@@ -1,3 +1,4 @@
 const a = 1;
-const b = 2;
+const b = 3;
+const c = 4;
 export { a };`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    expect(file.filePath).toBe("src/main.ts");
    expect(file.status).toBe(DiffFileStatus.Modified);
    expect(file.lines).toHaveLength(6); // header + 4 content lines + 1 deletion

    // Header line
    expect(file.lines[0].type).toBe(DiffLineType.Header);

    // Context line
    expect(file.lines[1].type).toBe(DiffLineType.Context);
    expect(file.lines[1].content).toBe("const a = 1;");
    expect(file.lines[1].oldLineNumber).toBe(1);
    expect(file.lines[1].newLineNumber).toBe(1);

    // Deletion
    expect(file.lines[2].type).toBe(DiffLineType.Deletion);
    expect(file.lines[2].content).toBe("const b = 2;");
    expect(file.lines[2].oldLineNumber).toBe(2);

    // Additions
    expect(file.lines[3].type).toBe(DiffLineType.Addition);
    expect(file.lines[3].content).toBe("const b = 3;");
    expect(file.lines[3].newLineNumber).toBe(2);

    expect(file.lines[4].type).toBe(DiffLineType.Addition);
    expect(file.lines[4].content).toBe("const c = 4;");
    expect(file.lines[4].newLineNumber).toBe(3);

    // Context
    expect(file.lines[5].type).toBe(DiffLineType.Context);
    expect(file.lines[5].content).toBe("export { a };");
    expect(file.lines[5].oldLineNumber).toBe(3);
    expect(file.lines[5].newLineNumber).toBe(4);
  });

  it("parses a new file diff", () => {
    const patch = `diff --git a/src/new.ts b/src/new.ts
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/src/new.ts
@@ -0,0 +1,3 @@
+line 1
+line 2
+line 3`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    expect(file.filePath).toBe("src/new.ts");
    expect(file.status).toBe(DiffFileStatus.Added);
    expect(file.lines).toHaveLength(4); // header + 3 additions

    expect(file.lines[1].type).toBe(DiffLineType.Addition);
    expect(file.lines[1].content).toBe("line 1");
    expect(file.lines[1].newLineNumber).toBe(1);
  });

  it("parses a deleted file diff", () => {
    const patch = `diff --git a/src/old.ts b/src/old.ts
deleted file mode 100644
index abc1234..0000000
--- a/src/old.ts
+++ /dev/null
@@ -1,2 +0,0 @@
-old line 1
-old line 2`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    expect(file.filePath).toBe("src/old.ts");
    expect(file.status).toBe(DiffFileStatus.Deleted);
    expect(file.lines).toHaveLength(3); // header + 2 deletions

    expect(file.lines[1].type).toBe(DiffLineType.Deletion);
    expect(file.lines[1].content).toBe("old line 1");
    expect(file.lines[1].oldLineNumber).toBe(1);
  });

  it("parses a renamed file diff", () => {
    const patch = `diff --git a/src/old.ts b/src/new.ts
similarity index 90%
rename from src/old.ts
rename to src/new.ts
index abc1234..def5678 100644
--- a/src/old.ts
+++ b/src/new.ts
@@ -1,3 +1,3 @@
 const a = 1;
-const b = 2;
+const b = 3;
 export { a };`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    expect(file.filePath).toBe("src/new.ts");
    expect(file.oldPath).toBe("src/old.ts");
    expect(file.status).toBe(DiffFileStatus.Renamed);
  });

  it("parses multiple files in a single diff", () => {
    const patch = `diff --git a/src/a.ts b/src/a.ts
index abc1234..def5678 100644
--- a/src/a.ts
+++ b/src/a.ts
@@ -1,2 +1,2 @@
-old line
+new line
 context
diff --git a/src/b.ts b/src/b.ts
new file mode 100644
index 0000000..abc1234
--- /dev/null
+++ b/src/b.ts
@@ -0,0 +1,1 @@
+new file content`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(2);

    expect(files[0].filePath).toBe("src/a.ts");
    expect(files[0].status).toBe(DiffFileStatus.Modified);

    expect(files[1].filePath).toBe("src/b.ts");
    expect(files[1].status).toBe(DiffFileStatus.Added);
  });

  it("parses multiple hunks in a single file", () => {
    // Use array join to avoid whitespace issues with template literals
    const patch = [
      "diff --git a/src/main.ts b/src/main.ts",
      "index abc1234..def5678 100644",
      "--- a/src/main.ts",
      "+++ b/src/main.ts",
      "@@ -1,3 +1,3 @@",
      " line 1",
      "-line 2",
      "+line 2 modified",
      " line 3",
      "@@ -10,3 +10,3 @@",
      " line 10",
      "-line 11",
      "+line 11 modified",
      " line 12",
    ].join("\n");

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    // 2 headers + 4 content lines from first hunk + 4 content lines from second hunk
    expect(file.lines).toHaveLength(10);

    // First hunk
    expect(file.lines[0].type).toBe(DiffLineType.Header);
    expect(file.lines[1].oldLineNumber).toBe(1);
    expect(file.lines[1].newLineNumber).toBe(1);

    // Second hunk
    expect(file.lines[5].type).toBe(DiffLineType.Header);
    expect(file.lines[6].oldLineNumber).toBe(10);
    expect(file.lines[6].newLineNumber).toBe(10);
  });

  it("handles 'No newline at end of file' marker", () => {
    const patch = `diff --git a/src/main.ts b/src/main.ts
index abc1234..def5678 100644
--- a/src/main.ts
+++ b/src/main.ts
@@ -1,2 +1,2 @@
 line 1
-line 2
\\ No newline at end of file
+line 2 modified
\\ No newline at end of file`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);

    const file = files[0];
    // Should not include "no newline" markers as diff lines
    const noNewlineLines = file.lines.filter(
      (l) => l.content === "\\ No newline at end of file"
    );
    expect(noNewlineLines).toHaveLength(0);
  });

  it("handles binary file changes", () => {
    const patch = `diff --git a/image.png b/image.png
index abc1234..def5678 100644
Binary files a/image.png and b/image.png differ
diff --git a/src/main.ts b/src/main.ts
index abc1234..def5678 100644
--- a/src/main.ts
+++ b/src/main.ts
@@ -1,2 +1,2 @@
-old
+new
 context`;

    const files = parseUnifiedDiff(patch);
    // Binary file should still be included (with no lines)
    // and the text file should be parsed correctly
    expect(files.length).toBeGreaterThanOrEqual(1);

    const textFile = files.find((f) => f.filePath === "src/main.ts");
    expect(textFile).toBeDefined();
    expect(textFile!.status).toBe(DiffFileStatus.Modified);
  });

  it("handles file mode changes", () => {
    const patch = `diff --git a/script.sh b/script.sh
old mode 100644
new mode 100755
index abc1234..def5678
--- a/script.sh
+++ b/script.sh
@@ -1,2 +1,2 @@
-echo "old"
+echo "new"
 exit 0`;

    const files = parseUnifiedDiff(patch);
    expect(files).toHaveLength(1);
    expect(files[0].status).toBe(DiffFileStatus.Modified);
  });
});
