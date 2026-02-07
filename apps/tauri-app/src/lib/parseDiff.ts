// Parses a unified diff string (git patch) into structured DiffFile objects
// for rendering in the DiffViewer component.

import {
  type DiffFile,
  type DiffLine,
  DiffFileStatus,
  DiffLineType,
} from "@/components/review/DiffViewer";

/**
 * Parses a unified diff string into an array of DiffFile objects.
 *
 * Handles standard unified diff format as produced by `git diff`.
 * Supports added, modified, deleted, and renamed files.
 */
export function parseUnifiedDiff(patch: string): DiffFile[] {
  const files: DiffFile[] = [];
  const lines = patch.split("\n");

  let i = 0;
  while (i < lines.length) {
    // Look for "diff --git" header
    if (!lines[i].startsWith("diff --git ")) {
      i++;
      continue;
    }

    // Parse the diff header to extract file paths
    const diffHeader = lines[i];
    const pathMatch = diffHeader.match(/^diff --git a\/(.+?) b\/(.+)$/);
    if (!pathMatch) {
      i++;
      continue;
    }

    const oldPath = pathMatch[1];
    const newPath = pathMatch[2];
    i++;

    // Parse extended header lines (old mode, new mode, similarity, rename, etc.)
    let status = DiffFileStatus.Modified;
    let isNewFile = false;
    let isDeletedFile = false;
    let isRenamedFile = false;

    while (i < lines.length && !lines[i].startsWith("diff --git ")) {
      const line = lines[i];

      if (line.startsWith("new file mode")) {
        isNewFile = true;
        status = DiffFileStatus.Added;
        i++;
      } else if (line.startsWith("deleted file mode")) {
        isDeletedFile = true;
        status = DiffFileStatus.Deleted;
        i++;
      } else if (line.startsWith("rename from ") || line.startsWith("rename to ")) {
        isRenamedFile = true;
        status = DiffFileStatus.Renamed;
        i++;
      } else if (line.startsWith("similarity index") || line.startsWith("dissimilarity index")) {
        i++;
      } else if (line.startsWith("old mode") || line.startsWith("new mode")) {
        i++;
      } else if (line.startsWith("index ")) {
        i++;
      } else if (line.startsWith("Binary files")) {
        // Binary file change - skip content
        i++;
        break;
      } else if (line.startsWith("--- ") || line.startsWith("+++ ")) {
        // Start of the unified diff content - handled below
        i++;
      } else if (line.startsWith("@@")) {
        // Hunk header - start parsing diff lines
        break;
      } else {
        i++;
      }
    }

    // Parse diff hunks
    const diffLines: DiffLine[] = [];

    while (i < lines.length && !lines[i].startsWith("diff --git ")) {
      const line = lines[i];

      if (line.startsWith("@@")) {
        // Parse hunk header: @@ -oldStart,oldCount +newStart,newCount @@
        const hunkMatch = line.match(
          /^@@ -(\d+)(?:,\d+)? \+(\d+)(?:,\d+)? @@(.*)$/
        );
        if (hunkMatch) {
          diffLines.push({
            type: DiffLineType.Header,
            content: line,
          });

          let oldLineNum = parseInt(hunkMatch[1], 10);
          let newLineNum = parseInt(hunkMatch[2], 10);
          i++;

          // Parse lines within this hunk
          while (i < lines.length && !lines[i].startsWith("@@") && !lines[i].startsWith("diff --git ")) {
            const hunkLine = lines[i];

            if (hunkLine.startsWith("+")) {
              diffLines.push({
                type: DiffLineType.Addition,
                content: hunkLine.substring(1),
                newLineNumber: newLineNum,
              });
              newLineNum++;
            } else if (hunkLine.startsWith("-")) {
              diffLines.push({
                type: DiffLineType.Deletion,
                content: hunkLine.substring(1),
                oldLineNumber: oldLineNum,
              });
              oldLineNum++;
            } else if (hunkLine.startsWith(" ")) {
              diffLines.push({
                type: DiffLineType.Context,
                content: hunkLine.substring(1),
                oldLineNumber: oldLineNum,
                newLineNumber: newLineNum,
              });
              oldLineNum++;
              newLineNum++;
            } else if (hunkLine === "\\ No newline at end of file") {
              // Skip "no newline" markers
              i++;
              continue;
            } else if (hunkLine === "") {
              // Empty line at the end of the diff
              i++;
              continue;
            } else if (hunkLine.trim() !== "") {
              // Unexpected non-empty line format - log and stop parsing this hunk
              console.warn(`Unexpected line format in diff at line ${i}: "${hunkLine}"`);
              break;
            } else {
              // Empty line at end of hunk
              break;
            }
            i++;
          }
        } else {
          i++;
        }
      } else {
        i++;
      }
    }

    const file: DiffFile = {
      filePath: isDeletedFile ? oldPath : newPath,
      status,
      lines: diffLines,
    };

    // Include oldPath for renamed files
    if (isRenamedFile && oldPath !== newPath) {
      file.oldPath = oldPath;
    }
    // For deleted files, show oldPath as the filePath is the old path
    if (isNewFile) {
      file.filePath = newPath;
    }

    files.push(file);
  }

  return files;
}
