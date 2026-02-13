# Developer Setup

This guide describes the development setup for DeliDev.

## Repository Layout

- `apps/main-server/` - Go main server
- `apps/worker-server/` - Go worker server
- `apps/tauri-app/` - Tauri + React client
- root `go.mod` - single Go module for server apps

## Prerequisites

1. Go version defined in root `go.mod`
2. Node.js + pnpm
3. Rust toolchain for Tauri host
4. Docker or Podman for worker execution

## Bootstrap

1. install JS dependencies
- `pnpm install`

2. verify Go workspace
- `go mod tidy`
- `go test ./...`

3. verify frontend tests
- `cd apps/tauri-app && pnpm test`

## Running Local Endpoint Workspace Stack

1. start main server locally
- `go run ./apps/main-server`

2. start worker server locally
- `go run ./apps/worker-server`

3. start Tauri app
- `cd apps/tauri-app && pnpm tauri dev`

4. create workspace with endpoint
- `http://127.0.0.1:<main-server-port>`

## Configuration Files

1. `.env` for server runtime
2. `apps/tauri-app/.env` for client runtime
3. OS keychain entries for credentials and tokens

## Recommended Local Env Vars

| Key | Example |
|---|---|
| `DELIDEV_HTTP_ADDR` | `127.0.0.1:4621` |
| `DELIDEV_DATABASE_URL` | `postgres://localhost:5432/delidev` |
| `DELIDEV_PR_POLL_INTERVAL_SEC` | `30` |
| `DELIDEV_WORKTREE_ROOT` | `~/.delidev/worktrees` |

## Validation Checklist

1. client can create and switch workspaces
2. UnitTask can start and produce session logs
3. event stream reconnect works after server restart
4. PR polling updates appear in PR Management
5. Web Notification permission flow works

## Notes

1. validate business flows via Connect RPC, not Tauri-only shortcuts
2. direct local-folder execution is unsupported; use worktree-based execution only
