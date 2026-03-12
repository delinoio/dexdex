### Instructions

- Use the `@docs/` directory as the source of truth. You should list the files in the docs directory before starting any task, and update the documents as required. The `@docs/` directory should always be up-to-date.
- After completing each task, update the relevant documentation in `@docs/` to reflect any changes made. In particular, `@docs/design.md` must accurately reflect the current state of the codebase (entities, architecture, workflows, etc.).
- Write all comments in English.
- Prefer enum types over strings when all variants are known at the moment of writing the code.
- If you modified Rust code, run `cargo test` from the root directory before finishing your task.
- If you modified frontend code, run `pnpm test` from the frontend directory before finishing your task.
- Commit your work as frequent as possible using git. Do NOT use `--no-verify` flag.
- Do not guess; rather search for the web.
- Debug by logging. You should write enough logging code.
- Prioritize Connect RPC-based communication for business flows over Tauri-specific bindings.

### Project Structure

This is a monorepo with three main components:

| Component | Path | Description |
|-----------|------|-------------|
| Main Server | `apps/main-server/` | Rust backend (Axum), task management, RPC server |
| Worker Server | `apps/worker-server/` | Rust worker, runs AI agents in Docker sandboxes |
| Desktop App | `apps/tauri-app/` | Tauri + React frontend |

Shared Rust crates live in `crates/`:
- `entities` — Core entity definitions
- `coding_agents` — AI agent abstraction, Docker sandboxing, task execution
- `task_store` — Task persistence (SQLite, PostgreSQL, in-memory)
- `rpc_protocol` — Connect RPC protocol definitions (Protobuf)
- `git_ops` — Git operations, worktree management
- `auth` — JWT authentication & RBAC
- `secrets` — Cross-platform keychain access
- `worker_impl` — Local worker implementation
- `config` — Configuration management
- `plan_parser` — YAML plan parsing

### API Reference

The RPC API is defined in `crates/rpc_protocol/proto/dexdex.proto`. When you need to find or modify an RPC method or message type, read this file first — it lists all method names and message types, eliminating the need to guess and grep.

### Types and Schemas

- Never use `any` type in TypeScript. Use concrete types instead.
- Prefer schema-based parsers (e.g., Zod) over raw `JSON.parse`.
- Avoid loose interfaces and undocumented data structures. If AI has to make assumptions about the shape of data, those assumptions may be wrong and all subsequent reasoning can spiral out of control.

### Utility Functions

- Input sanitization utilities are in `crates/entities/src/lib.rs` (e.g., `sanitize_user_input()`, `validate_prompt()`).
- Frontend shared utilities are in `apps/tauri-app/src/lib/`.
- Read the relevant utility file before writing new helper functions to avoid duplicating existing code.
- For Tauri commands, define a single function per command. Do NOT define two separate functions with `#[cfg(desktop)]` and `#[cfg(not(desktop))]`. Instead, use inline `#[cfg(desktop)]` blocks within the function body for desktop-only logic, and `#[cfg(not(desktop))]` to suppress unused variables and return a "not supported" error. See `workspace.rs` and `repository.rs` for reference.
