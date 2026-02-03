import { describe, it, expect, vi } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { DiffViewer, DiffFileList, DiffLineType, type DiffFile } from "../DiffViewer";
import type { ReviewComment } from "@/hooks/useReviewComments";

const mockDiffFile: DiffFile = {
  filePath: "src/test.ts",
  status: "modified",
  lines: [
    { type: DiffLineType.Header, content: "@@ -1,5 +1,6 @@" },
    { type: DiffLineType.Context, content: "const a = 1;", oldLineNumber: 1, newLineNumber: 1 },
    { type: DiffLineType.Deletion, content: "const b = 2;", oldLineNumber: 2 },
    { type: DiffLineType.Addition, content: "const b = 3;", newLineNumber: 2 },
    { type: DiffLineType.Addition, content: "const c = 4;", newLineNumber: 3 },
    { type: DiffLineType.Context, content: "export { a };", oldLineNumber: 3, newLineNumber: 4 },
  ],
};

const mockComment: ReviewComment = {
  id: "comment-1",
  filePath: "src/test.ts",
  lineNumber: 2,
  content: "This line needs attention",
  author: "Test User",
  createdAt: "2026-01-01T10:00:00Z",
  updatedAt: "2026-01-01T10:00:00Z",
};

describe("DiffViewer", () => {
  it("renders the file path in header", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    expect(screen.getByText("src/test.ts")).toBeInTheDocument();
  });

  it("renders all diff lines", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    expect(screen.getByText("const a = 1;")).toBeInTheDocument();
    expect(screen.getByText("const b = 2;")).toBeInTheDocument();
    expect(screen.getByText("const b = 3;")).toBeInTheDocument();
    expect(screen.getByText("const c = 4;")).toBeInTheDocument();
    expect(screen.getByText("export { a };")).toBeInTheDocument();
  });

  it("displays file status badge", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    expect(screen.getByText("MODIFIED")).toBeInTheDocument();
  });

  it("shows comment count when there are comments", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[mockComment]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    expect(screen.getByText("1 comment")).toBeInTheDocument();
  });

  it("shows 'Mark as viewed' button when onMarkAsViewed is provided", () => {
    const handleMarkAsViewed = vi.fn();
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
        onMarkAsViewed={handleMarkAsViewed}
        isViewed={false}
      />
    );

    const button = screen.getByRole("button", { name: /mark as viewed/i });
    expect(button).toBeInTheDocument();

    fireEvent.click(button);
    expect(handleMarkAsViewed).toHaveBeenCalled();
  });

  it("shows 'Viewed' when file is marked as viewed", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
        onMarkAsViewed={vi.fn()}
        isViewed={true}
      />
    );

    expect(screen.getByRole("button", { name: /viewed/i })).toBeInTheDocument();
  });

  it("opens comment input when line number is clicked", () => {
    render(
      <DiffViewer
        file={mockDiffFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    // Click on the second row (index 1), which is a context line with line number 1
    const lineNumber = screen.getByTestId("diff-line-1");
    fireEvent.click(lineNumber);

    expect(screen.getByTestId("comment-input-form")).toBeInTheDocument();
  });

  it("shows renamed file info when applicable", () => {
    const renamedFile: DiffFile = {
      ...mockDiffFile,
      filePath: "src/newTest.ts",
      oldPath: "src/test.ts",
      status: "renamed",
    };

    render(
      <DiffViewer
        file={renamedFile}
        comments={[]}
        onAddComment={vi.fn()}
        onEditComment={vi.fn()}
        onDeleteComment={vi.fn()}
      />
    );

    expect(screen.getByText("(was: src/test.ts)")).toBeInTheDocument();
  });
});

describe("DiffFileList", () => {
  const mockFiles: DiffFile[] = [
    { ...mockDiffFile },
    {
      filePath: "src/another.ts",
      status: "added",
      lines: [],
    },
    {
      filePath: "src/deleted.ts",
      status: "deleted",
      lines: [],
    },
  ];

  it("renders all files", () => {
    render(
      <DiffFileList
        files={mockFiles}
        onSelectFile={vi.fn()}
        viewedFiles={new Set()}
        commentCounts={{}}
      />
    );

    expect(screen.getByText("test.ts")).toBeInTheDocument();
    expect(screen.getByText("another.ts")).toBeInTheDocument();
    expect(screen.getByText("deleted.ts")).toBeInTheDocument();
  });

  it("shows viewed count", () => {
    render(
      <DiffFileList
        files={mockFiles}
        onSelectFile={vi.fn()}
        viewedFiles={new Set(["src/test.ts"])}
        commentCounts={{}}
      />
    );

    expect(screen.getByText("1/3 viewed")).toBeInTheDocument();
  });

  it("calls onSelectFile when a file is clicked", () => {
    const handleSelectFile = vi.fn();
    render(
      <DiffFileList
        files={mockFiles}
        onSelectFile={handleSelectFile}
        viewedFiles={new Set()}
        commentCounts={{}}
      />
    );

    fireEvent.click(screen.getByText("test.ts"));
    expect(handleSelectFile).toHaveBeenCalledWith("src/test.ts");
  });

  it("shows comment counts", () => {
    render(
      <DiffFileList
        files={mockFiles}
        onSelectFile={vi.fn()}
        viewedFiles={new Set()}
        commentCounts={{ "src/test.ts": 2 }}
      />
    );

    expect(screen.getByText("2")).toBeInTheDocument();
  });

  it("highlights selected file", () => {
    render(
      <DiffFileList
        files={mockFiles}
        selectedFilePath="src/test.ts"
        onSelectFile={vi.fn()}
        viewedFiles={new Set()}
        commentCounts={{}}
      />
    );

    const selectedButton = screen.getByText("test.ts").closest("button");
    expect(selectedButton).toHaveClass("bg-[hsl(var(--accent))]");
  });

  it("shows checkmark for viewed files", () => {
    const { container } = render(
      <DiffFileList
        files={mockFiles}
        onSelectFile={vi.fn()}
        viewedFiles={new Set(["src/test.ts"])}
        commentCounts={{}}
      />
    );

    // Check for the checkmark SVG
    const checkmarks = container.querySelectorAll("svg.text-green-500");
    expect(checkmarks.length).toBe(1);
  });
});
