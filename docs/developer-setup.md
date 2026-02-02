# DeliDev Developer Setup Guide

This guide helps developers set up their environment for contributing to DeliDev.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Repository Structure](#repository-structure)
3. [Initial Setup](#initial-setup)
4. [Development Workflow](#development-workflow)
5. [Building](#building)
6. [Testing](#testing)
7. [Code Style](#code-style)
8. [Debugging](#debugging)
9. [Contributing](#contributing)

---

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.83+ | Backend development |
| **Node.js** | 22+ | Frontend development |
| **pnpm** | 9+ | Package management |
| **Docker** | 20+ | Worker container execution |

### Platform-Specific Requirements

#### macOS

```bash
# Install Xcode command line tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (via nvm recommended)
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install 22

# Install pnpm
npm install -g pnpm

# Install Docker Desktop
brew install --cask docker
```

#### Linux

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install pnpm
npm install -g pnpm

# Install Docker
sudo apt-get install docker.io
sudo usermod -aG docker $USER

# Install system dependencies for Tauri
sudo apt-get install libwebkit2gtk-4.1-dev \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

#### Windows

1. Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
2. Install [Rust](https://www.rust-lang.org/tools/install)
3. Install [Node.js](https://nodejs.org/)
4. Install pnpm: `npm install -g pnpm`
5. Install [Docker Desktop](https://www.docker.com/products/docker-desktop)

---

## Repository Structure

```
delidev/
├── apps/
│   ├── main-server/       # Main Server (Rust binary)
│   ├── worker-server/     # Worker Server (Rust binary)
│   ├── tauri-app/         # Desktop/Mobile app (Tauri + React)
│   └── webapp/            # Marketing website (Next.js)
├── crates/
│   ├── auth/              # JWT & OIDC authentication
│   ├── coding_agents/     # AI agent abstraction
│   ├── config/            # Configuration management
│   ├── entities/          # Core data models
│   ├── git_ops/           # Git operations
│   ├── plan_parser/       # PLAN.yaml parsing
│   ├── rpc_protocol/      # RPC types and errors
│   ├── secrets/           # Keychain access
│   └── task_store/        # Task persistence
├── docs/                  # Documentation
├── .github/               # GitHub Actions workflows
├── Cargo.toml             # Rust workspace config
├── pnpm-workspace.yaml    # pnpm workspace config
└── turbo.json             # Turbo build config
```

---

## Initial Setup

### 1. Clone the Repository

```bash
git clone https://github.com/delinoio/delidev.git
cd delidev
```

### 2. Install Dependencies

```bash
# Install Rust dependencies (automatic on build)
cargo fetch

# Install Node.js dependencies
pnpm install
```

### 3. Set Up Environment

Create a `.env` file in the root:

```env
# For local development
DELIDEV_SINGLE_USER_MODE=true
DELIDEV_LOG_LEVEL=debug

# Optional: For testing remote mode
# DELIDEV_JWT_SECRET=your-secret-key
# DELIDEV_OIDC_ISSUER_URL=https://auth.example.com
```

### 4. Verify Setup

```bash
# Check Rust builds
cargo check

# Check frontend builds
pnpm -C apps/tauri-app build
```

---

## Development Workflow

### Running the Main Server

```bash
cd apps/main-server
cargo run
```

The server starts at `http://localhost:54871`.

### Running the Worker Server

```bash
cd apps/worker-server
cargo run
```

The worker starts at `http://localhost:54872`.

### Running the Desktop App (Development)

```bash
cd apps/tauri-app
pnpm dev
```

This starts:
- Vite dev server at `http://localhost:1420` (Tauri-specific port)
- Tauri desktop window with hot reload

### Running Frontend Only

```bash
cd apps/tauri-app
pnpm dev:frontend
```

This starts the Vite dev server at `http://localhost:5173` (default Vite port).
Useful for UI development without the Tauri overhead.

### Running with Docker Compose

```bash
docker-compose up
```

This starts the full stack with all services.

---

## Building

### Debug Build

```bash
# All Rust crates
cargo build

# Tauri app
cd apps/tauri-app
pnpm tauri build --debug
```

### Release Build

```bash
# Optimized Rust build
cargo build --release

# Tauri app
cd apps/tauri-app
pnpm tauri build
```

### Building for Specific Platforms

```bash
# macOS (ARM)
cargo build --target aarch64-apple-darwin

# macOS (Intel)
cargo build --target x86_64-apple-darwin

# Linux
cargo build --target x86_64-unknown-linux-gnu

# Windows
cargo build --target x86_64-pc-windows-msvc
```

### Building Mobile Apps

```bash
cd apps/tauri-app

# iOS
pnpm tauri ios build

# Android
pnpm tauri android build
```

---

## Testing

### Rust Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p entities

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_workspace_crud
```

### Frontend Tests

```bash
cd apps/tauri-app

# Run unit tests
pnpm test

# Run tests in watch mode
pnpm test:watch

# Run E2E tests
pnpm test:e2e

# Run E2E tests with UI
pnpm test:e2e:ui
```

### Integration Tests

```bash
# Main server integration tests
cd apps/main-server
cargo test --test api_tests

# Worker server integration tests
cd apps/worker-server
cargo test --test api_tests
```

### Test Coverage

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html
```

---

## Code Style

### Rust

We use `rustfmt` for formatting and `clippy` for linting.

```bash
# Format all Rust code
cargo fmt

# Run clippy
cargo clippy --all-features

# Fix clippy warnings automatically
cargo clippy --fix
```

Configuration in `.rustfmt.toml`:

```toml
edition = "2018"
format_strings = true
group_imports = "StdExternalCrate"
```

### TypeScript/JavaScript

We use ESLint and Prettier.

```bash
cd apps/tauri-app

# Lint
pnpm lint

# Fix lint issues
pnpm lint --fix

# Format
pnpm format
```

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add user authentication
fix: resolve login timeout issue
docs: update API documentation
test: add integration tests for task API
refactor: simplify error handling
```

---

## Debugging

### Rust Debugging

#### VS Code

Add to `.vscode/launch.json`:

```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug Main Server",
      "cargo": {
        "args": ["build", "--bin=main-server", "--package=main-server"],
        "filter": {
          "name": "main-server",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}",
      "env": {
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

#### Logging

Enable detailed logging:

```bash
RUST_LOG=debug cargo run
```

Log levels: `error`, `warn`, `info`, `debug`, `trace`

### Frontend Debugging

1. Open DevTools in the Tauri window (Cmd/Ctrl + Shift + I)
2. Use React DevTools extension
3. Check Network tab for API requests

### Debugging Tauri

```bash
# Run with devtools enabled
WEBKIT_DISABLE_COMPOSITING_MODE=1 pnpm tauri dev
```

---

## Contributing

### Branch Naming

- `feature/description` - New features
- `fix/description` - Bug fixes
- `docs/description` - Documentation
- `refactor/description` - Code refactoring

### Pull Request Process

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run tests and linting
5. Submit a pull request

### Pre-commit Checklist

Before committing:

1. `cargo fmt` - Format Rust code
2. `cargo clippy` - Check for warnings
3. `cargo test` - Run Rust tests
4. `pnpm -C apps/tauri-app test` - Run frontend tests
5. `pnpm -C apps/tauri-app build` - Verify build

### CI/CD

GitHub Actions runs on every PR:

- Rust formatting check
- Clippy linting
- Rust unit tests
- TypeScript type checking
- ESLint
- Frontend build verification

---

## Useful Commands

### Quick Reference

```bash
# Start everything for development
docker-compose up -d && cd apps/tauri-app && pnpm dev

# Run all tests
cargo test && pnpm -C apps/tauri-app test

# Format everything
cargo fmt && pnpm -C apps/tauri-app format

# Clean build artifacts
cargo clean && pnpm -C apps/tauri-app clean

# Update dependencies
cargo update && pnpm update

# Generate documentation
cargo doc --open
```

### Database Operations

```bash
# Reset SQLite database
rm ~/.delidev/data.db

# View SQLite database
sqlite3 ~/.delidev/data.db ".tables"
```

### Docker Operations

```bash
# View running containers
docker ps

# View logs
docker logs delidev-main-server

# Stop all containers
docker-compose down

# Rebuild containers
docker-compose build --no-cache
```

---

## Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Tauri Documentation](https://tauri.app/v2/guides/)
- [React Documentation](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/handbook/)

---

## Getting Help

- Check existing [GitHub Issues](https://github.com/delinoio/delidev/issues)
- Join the [Discord community](https://discord.gg/delidev)
- Read the [documentation](/docs)

Happy coding!
