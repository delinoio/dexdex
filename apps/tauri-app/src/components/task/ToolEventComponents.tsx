import { cn } from "@/lib/utils";
import {
  FileCodeIcon,
  TerminalIcon,
  SearchIcon,
  FolderIcon,
  EditIcon,
} from "@/components/ui/Icons";

// Known tool types for Claude Code and other agents
enum ToolType {
  Read = "Read",
  Write = "Write",
  Edit = "Edit",
  Bash = "Bash",
  Glob = "Glob",
  Grep = "Grep",
  Task = "Task",
  WebFetch = "WebFetch",
  WebSearch = "WebSearch",
  TodoWrite = "TodoWrite",
  AskUserQuestion = "AskUserQuestion",
  NotebookEdit = "NotebookEdit",
  Unknown = "Unknown",
}

function getToolType(toolName: string): ToolType {
  const normalized = toolName.toLowerCase();
  if (normalized === "read") return ToolType.Read;
  if (normalized === "write") return ToolType.Write;
  if (normalized === "edit") return ToolType.Edit;
  if (normalized === "bash") return ToolType.Bash;
  if (normalized === "glob") return ToolType.Glob;
  if (normalized === "grep") return ToolType.Grep;
  if (normalized === "task") return ToolType.Task;
  if (normalized === "webfetch") return ToolType.WebFetch;
  if (normalized === "websearch") return ToolType.WebSearch;
  if (normalized === "todowrite") return ToolType.TodoWrite;
  if (normalized === "askuserquestion") return ToolType.AskUserQuestion;
  if (normalized === "notebookedit") return ToolType.NotebookEdit;
  return ToolType.Unknown;
}

// Type guards for common tool input shapes
interface ReadInput {
  file_path: string;
  offset?: number;
  limit?: number;
}

interface WriteInput {
  file_path: string;
  content: string;
}

interface EditInput {
  file_path: string;
  old_string: string;
  new_string: string;
  replace_all?: boolean;
}

interface BashInput {
  command: string;
  description?: string;
  timeout?: number;
}

interface GlobInput {
  pattern: string;
  path?: string;
}

interface GrepInput {
  pattern: string;
  path?: string;
  glob?: string;
  type?: string;
  output_mode?: string;
}

interface TaskInput {
  description: string;
  prompt: string;
  subagent_type: string;
}

interface WebFetchInput {
  url: string;
  prompt: string;
}

interface WebSearchInput {
  query: string;
}

interface TodoWriteInput {
  todos: Array<{ content: string; status: string; activeForm?: string }>;
}

interface AskUserQuestionInput {
  questions: Array<{
    question: string;
    header?: string;
    options?: Array<{ label: string; description?: string }>;
  }>;
}

function isReadInput(input: unknown): input is ReadInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "file_path" in input &&
    typeof (input as ReadInput).file_path === "string"
  );
}

function isWriteInput(input: unknown): input is WriteInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "file_path" in input &&
    "content" in input
  );
}

function isEditInput(input: unknown): input is EditInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "file_path" in input &&
    "old_string" in input &&
    "new_string" in input
  );
}

function isBashInput(input: unknown): input is BashInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "command" in input &&
    typeof (input as BashInput).command === "string"
  );
}

function isGlobInput(input: unknown): input is GlobInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "pattern" in input &&
    typeof (input as GlobInput).pattern === "string"
  );
}

function isGrepInput(input: unknown): input is GrepInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "pattern" in input &&
    typeof (input as GrepInput).pattern === "string"
  );
}

function isTaskInput(input: unknown): input is TaskInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "prompt" in input &&
    "subagent_type" in input
  );
}

function isWebFetchInput(input: unknown): input is WebFetchInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "url" in input &&
    typeof (input as WebFetchInput).url === "string"
  );
}

function isWebSearchInput(input: unknown): input is WebSearchInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "query" in input &&
    typeof (input as WebSearchInput).query === "string"
  );
}

function isTodoWriteInput(input: unknown): input is TodoWriteInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "todos" in input &&
    Array.isArray((input as TodoWriteInput).todos)
  );
}

function isAskUserQuestionInput(input: unknown): input is AskUserQuestionInput {
  return (
    typeof input === "object" &&
    input !== null &&
    "questions" in input &&
    Array.isArray((input as AskUserQuestionInput).questions)
  );
}

// Components

interface ToolUseContentProps {
  toolName: string;
  input: unknown;
}

export function ToolUseContent({ toolName, input }: ToolUseContentProps) {
  const toolType = getToolType(toolName);

  switch (toolType) {
    case ToolType.Read:
      return <ReadToolUse input={input} />;
    case ToolType.Write:
      return <WriteToolUse input={input} />;
    case ToolType.Edit:
      return <EditToolUse input={input} />;
    case ToolType.Bash:
      return <BashToolUse input={input} />;
    case ToolType.Glob:
      return <GlobToolUse input={input} />;
    case ToolType.Grep:
      return <GrepToolUse input={input} />;
    case ToolType.Task:
      return <TaskToolUse input={input} />;
    case ToolType.WebFetch:
      return <WebFetchToolUse input={input} />;
    case ToolType.WebSearch:
      return <WebSearchToolUse input={input} />;
    case ToolType.TodoWrite:
      return <TodoWriteToolUse input={input} />;
    case ToolType.AskUserQuestion:
      return <AskUserQuestionToolUse input={input} />;
    default:
      return <DefaultToolUse toolName={toolName} input={input} />;
  }
}

interface ToolResultContentProps {
  toolName: string;
  output: unknown;
  isError: boolean;
}

export function ToolResultContent({ toolName, output, isError }: ToolResultContentProps) {
  const toolType = getToolType(toolName);

  // Handle error output
  if (isError) {
    const errorMessage = typeof output === "string" ? output : JSON.stringify(output, null, 2);
    return (
      <div className="text-destructive">
        <span className="font-medium">Error:</span>
        <pre className="mt-1 text-xs whitespace-pre-wrap">{errorMessage}</pre>
      </div>
    );
  }

  switch (toolType) {
    case ToolType.Read:
      return <ReadToolResult output={output} />;
    case ToolType.Bash:
      return <BashToolResult output={output} />;
    case ToolType.Glob:
      return <GlobToolResult output={output} />;
    case ToolType.Grep:
      return <GrepToolResult output={output} />;
    case ToolType.WebSearch:
      return <WebSearchToolResult output={output} />;
    default:
      return <DefaultToolResult output={output} />;
  }
}

// Tool Use Components

function ReadToolUse({ input }: { input: unknown }) {
  if (!isReadInput(input)) {
    return <DefaultToolUse toolName="Read" input={input} />;
  }

  return (
    <div className="flex items-center gap-2">
      <FileCodeIcon size={14} className="text-blue-400" />
      <span className="text-muted-foreground">Reading</span>
      <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
        {input.file_path}
      </code>
      {(input.offset !== undefined || input.limit !== undefined) && (
        <span className="text-xs text-muted-foreground">
          {input.offset !== undefined && `from line ${input.offset}`}
          {input.limit !== undefined && `, ${input.limit} lines`}
        </span>
      )}
    </div>
  );
}

function WriteToolUse({ input }: { input: unknown }) {
  if (!isWriteInput(input)) {
    return <DefaultToolUse toolName="Write" input={input} />;
  }

  return (
    <div>
      <div className="flex items-center gap-2">
        <FileCodeIcon size={14} className="text-green-400" />
        <span className="text-muted-foreground">Writing</span>
        <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
          {input.file_path}
        </code>
      </div>
      <details className="mt-1">
        <summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
          Show content ({input.content.length} characters)
        </summary>
        <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto bg-muted/50 p-2 rounded">
          {input.content}
        </pre>
      </details>
    </div>
  );
}

function EditToolUse({ input }: { input: unknown }) {
  if (!isEditInput(input)) {
    return <DefaultToolUse toolName="Edit" input={input} />;
  }

  return (
    <div>
      <div className="flex items-center gap-2">
        <EditIcon size={14} className="text-yellow-400" />
        <span className="text-muted-foreground">Editing</span>
        <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
          {input.file_path}
        </code>
        {input.replace_all && (
          <span className="text-xs px-1 py-0.5 bg-yellow-500/20 text-yellow-500 rounded">
            replace all
          </span>
        )}
      </div>
      <div className="mt-2 space-y-1 text-xs">
        <div className="flex items-start gap-2">
          <span className="text-red-400 shrink-0">-</span>
          <pre className="whitespace-pre-wrap bg-red-500/10 px-1.5 py-0.5 rounded max-h-20 overflow-y-auto">
            {input.old_string}
          </pre>
        </div>
        <div className="flex items-start gap-2">
          <span className="text-green-400 shrink-0">+</span>
          <pre className="whitespace-pre-wrap bg-green-500/10 px-1.5 py-0.5 rounded max-h-20 overflow-y-auto">
            {input.new_string}
          </pre>
        </div>
      </div>
    </div>
  );
}

function BashToolUse({ input }: { input: unknown }) {
  if (!isBashInput(input)) {
    return <DefaultToolUse toolName="Bash" input={input} />;
  }

  return (
    <div>
      <div className="flex items-center gap-2">
        <TerminalIcon size={14} className="text-yellow-500" />
        <span className="text-muted-foreground">Running</span>
        {input.description && (
          <span className="text-xs text-muted-foreground italic">
            ({input.description})
          </span>
        )}
      </div>
      <pre className="mt-1 text-sm font-mono bg-muted/50 px-2 py-1 rounded whitespace-pre-wrap">
        $ {input.command}
      </pre>
    </div>
  );
}

function GlobToolUse({ input }: { input: unknown }) {
  if (!isGlobInput(input)) {
    return <DefaultToolUse toolName="Glob" input={input} />;
  }

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <FolderIcon size={14} className="text-blue-400" />
      <span className="text-muted-foreground">Finding files matching</span>
      <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
        {input.pattern}
      </code>
      {input.path && (
        <>
          <span className="text-muted-foreground">in</span>
          <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
            {input.path}
          </code>
        </>
      )}
    </div>
  );
}

function GrepToolUse({ input }: { input: unknown }) {
  if (!isGrepInput(input)) {
    return <DefaultToolUse toolName="Grep" input={input} />;
  }

  return (
    <div className="flex items-center gap-2 flex-wrap">
      <SearchIcon size={14} className="text-purple-400" />
      <span className="text-muted-foreground">Searching for</span>
      <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
        {input.pattern}
      </code>
      {input.path && (
        <>
          <span className="text-muted-foreground">in</span>
          <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
            {input.path}
          </code>
        </>
      )}
      {input.glob && (
        <>
          <span className="text-muted-foreground">matching</span>
          <code className="px-1.5 py-0.5 bg-muted rounded text-sm font-mono">
            {input.glob}
          </code>
        </>
      )}
      {input.type && (
        <span className="text-xs px-1 py-0.5 bg-muted rounded">
          {input.type} files
        </span>
      )}
    </div>
  );
}

function TaskToolUse({ input }: { input: unknown }) {
  if (!isTaskInput(input)) {
    return <DefaultToolUse toolName="Task" input={input} />;
  }

  return (
    <div>
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground">Spawning</span>
        <span className="px-1.5 py-0.5 bg-cyan-500/20 text-cyan-500 rounded text-sm">
          {input.subagent_type}
        </span>
        <span className="text-muted-foreground">agent:</span>
        <span className="text-foreground">{input.description}</span>
      </div>
      <details className="mt-1">
        <summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
          Show prompt
        </summary>
        <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto bg-muted/50 p-2 rounded">
          {input.prompt}
        </pre>
      </details>
    </div>
  );
}

function WebFetchToolUse({ input }: { input: unknown }) {
  if (!isWebFetchInput(input)) {
    return <DefaultToolUse toolName="WebFetch" input={input} />;
  }

  return (
    <div>
      <div className="flex items-center gap-2">
        <span className="text-muted-foreground">Fetching</span>
        <a
          href={input.url}
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-400 hover:underline text-sm break-all"
        >
          {input.url}
        </a>
      </div>
      <div className="mt-1 text-xs text-muted-foreground italic">
        {input.prompt}
      </div>
    </div>
  );
}

function WebSearchToolUse({ input }: { input: unknown }) {
  if (!isWebSearchInput(input)) {
    return <DefaultToolUse toolName="WebSearch" input={input} />;
  }

  return (
    <div className="flex items-center gap-2">
      <SearchIcon size={14} className="text-blue-400" />
      <span className="text-muted-foreground">Searching web for</span>
      <span className="text-foreground font-medium">&ldquo;{input.query}&rdquo;</span>
    </div>
  );
}

function TodoWriteToolUse({ input }: { input: unknown }) {
  if (!isTodoWriteInput(input)) {
    return <DefaultToolUse toolName="TodoWrite" input={input} />;
  }

  return (
    <div>
      <div className="text-muted-foreground mb-1">Updating todo list:</div>
      <ul className="text-xs space-y-0.5">
        {input.todos.map((todo, idx) => (
          <li key={idx} className="flex items-center gap-2">
            <span
              className={cn(
                "w-2 h-2 rounded-full",
                todo.status === "completed" && "bg-green-500",
                todo.status === "in_progress" && "bg-yellow-500",
                todo.status === "pending" && "bg-muted-foreground"
              )}
            />
            <span
              className={cn(
                todo.status === "completed" && "line-through text-muted-foreground"
              )}
            >
              {todo.content}
            </span>
          </li>
        ))}
      </ul>
    </div>
  );
}

function AskUserQuestionToolUse({ input }: { input: unknown }) {
  if (!isAskUserQuestionInput(input)) {
    return <DefaultToolUse toolName="AskUserQuestion" input={input} />;
  }

  return (
    <div>
      {input.questions.map((q, idx) => (
        <div key={idx} className="mb-2">
          {q.header && (
            <span className="text-xs px-1.5 py-0.5 bg-purple-500/20 text-purple-400 rounded mr-2">
              {q.header}
            </span>
          )}
          <span className="text-foreground">{q.question}</span>
          {q.options && q.options.length > 0 && (
            <div className="mt-1 flex flex-wrap gap-1">
              {q.options.map((opt, optIdx) => (
                <span
                  key={optIdx}
                  className="text-xs px-2 py-0.5 bg-muted rounded"
                  title={opt.description}
                >
                  {opt.label}
                </span>
              ))}
            </div>
          )}
        </div>
      ))}
    </div>
  );
}

function DefaultToolUse({ toolName, input }: { toolName: string; input: unknown }) {
  return (
    <div>
      <span className="text-blue-500 font-medium">{toolName}</span>
      <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto">
        {JSON.stringify(input, null, 2)}
      </pre>
    </div>
  );
}

// Tool Result Components

function ReadToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);
  const lines = content.split("\n");
  const lineCount = lines.length;

  return (
    <details open={lineCount <= 30}>
      <summary className="text-xs text-muted-foreground cursor-pointer hover:text-foreground">
        File content ({lineCount} lines)
      </summary>
      <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-60 overflow-y-auto bg-muted/50 p-2 rounded font-mono">
        {content}
      </pre>
    </details>
  );
}

function BashToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);

  if (!content || content.trim() === "") {
    return (
      <span className="text-xs text-muted-foreground italic">
        (no output)
      </span>
    );
  }

  return (
    <pre className="text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto bg-muted/50 p-2 rounded font-mono">
      {content}
    </pre>
  );
}

function GlobToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);
  const files = content.split("\n").filter((f) => f.trim());

  return (
    <div>
      <span className="text-xs text-muted-foreground">
        Found {files.length} file{files.length !== 1 ? "s" : ""}
      </span>
      <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto">
        {content}
      </pre>
    </div>
  );
}

function GrepToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);
  const lines = content.split("\n").filter((l) => l.trim());

  return (
    <div>
      <span className="text-xs text-muted-foreground">
        {lines.length} match{lines.length !== 1 ? "es" : ""}
      </span>
      <pre className="mt-1 text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto bg-muted/50 p-2 rounded">
        {content}
      </pre>
    </div>
  );
}

function WebSearchToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);

  return (
    <pre className="text-xs text-muted-foreground whitespace-pre-wrap max-h-60 overflow-y-auto">
      {content}
    </pre>
  );
}

function DefaultToolResult({ output }: { output: unknown }) {
  const content = typeof output === "string" ? output : JSON.stringify(output, null, 2);

  return (
    <pre className="text-xs text-muted-foreground whitespace-pre-wrap max-h-40 overflow-y-auto">
      {content}
    </pre>
  );
}
