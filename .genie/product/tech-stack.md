# Forge Core Technical Stack

## Workspace Overview
- **Languages:** Rust (stable toolchain) + TypeScript for tooling.
- **Package Manager:** `pnpm` for Node scripts, Cargo workspace for Rust crates.
- **Structure:** `crates/` (server, db, executors, services, utils, deployment, local-deployment, vendor/codex), `shared/` (generated TS bindings), `npx-cli/` (CLI wrappers), `scripts/` (dev tooling), `dev_assets_seed/` (seed DB + assets).

## Rust Workspace

### Crates Structure
- **server** - Main API server (Axum routes, MCP task server binaries)
  - `crates/server/src/bin/server.rs` - Axum + tower-http exposes REST endpoints for tasks, attempts, assets, and file diffs
  - `crates/server/src/bin/mcp_task_server.rs` - MCP-compatible streams so IDE agents can talk to Forge without the desktop app
  - `crates/server/src/bin/generate_types.rs` - TypeScript type generator
- **db** - Database models, migrations (`crates/db/migrations`), SQLx integration. Run `npm run prepare-db` to regenerate SQLx data before compiling offline.
- **executors** - Task execution engines (Claude Code, Codex, Gemini, Cursor Agent, OpenCode). Defines MCP integration defaults (see `default_mcp.json`) and delegates work to provider-specific runners.
- **services** - Business logic layer (task orchestration, session management, process lifecycle). Handles worktree management, repo orchestration, auth, and task bookkeeping.
- **utils** - Shared utilities (port allocation, file operations, git helpers). Houses path + port helpers. Worktree temp dirs follow `automagik-forge` naming; port files under `$TMPDIR/automagik-forge/automagik-forge.port`.
- **deployment** - Production deployment configuration
- **local-deployment** - Local development deployment utilities

### Vendored Dependencies (Codex Integration)
Patched for thread-safety (LazyLock → thread_local! migration):
- **vendor/codex/codex-rs/protocol** - Codex MCP protocol definitions
- **vendor/codex/codex-rs/app-server-protocol** - Application server protocol
- **vendor/codex/codex-rs/mcp-types** - MCP type definitions

### Major Rust Dependencies
- **axum** (0.8.4) - Web framework with macros, multipart, WebSocket support
- **tokio** (1.0) - Async runtime with full feature set
- **tower-http** (0.5) - HTTP middleware (CORS)
- **serde** + **serde_json** - JSON serialization with order preservation
- **sqlx** - Async SQL toolkit (SQLite/Postgres support)
- **tracing** + **tracing-subscriber** - Structured logging with env filters
- **anyhow** + **thiserror** - Error handling
- **ts-rs** - TypeScript type generation from Rust structs
- **schemars** - JSON schema generation
- **openssl-sys** - Vendored OpenSSL for portability

## Task Execution & MCP

### MCP Configuration
- **Default Config:** `crates/executors/default_mcp.json`
- **Protocol Version:** Negotiated dynamically (see `86e00ce4` commit)
- **Server Binary:** `mcp_task_server.rs` handles task lifecycle via stdio

### API Routes & Execution
API routes such as `crates/server/src/routes/task_attempts.rs` manage attempt lifecycle, port allocations, commit metadata, and diff presentation. MCP task server streams logs/output through JSON-RPC; `crates/server/src/mcp/task_server.rs` references the same worktree manager as the REST API to keep behavior consistent.

### Executor Support
- Claude Code (Anthropic)
- Codex (OpenAI)
- Gemini (Google)
- Cursor Agent
- OpenCode

## Database & Assets

### SQLx Offline Mode
- **Migrations:** `crates/db/migrations/*.sql`
- **Latest:** `20251020000001_add_agent_task_status.sql`
- **Prepare:** `npm run prepare-db` generates SQLx metadata for offline compilation

Migrations live in `crates/db/migrations`; latest applied timestamps inform Automagik Forge release readiness. Never skip the migration ledger when shipping features.

### Dev Assets
- **Seed Data:** `dev_assets_seed/config.json` (sample database) + canonical SQLite DB + asset files
- **Runtime Copy:** `scripts/setup-dev-environment.js` copies the seed into `dev_assets/` whenever dev ports are allocated
- **Port Allocation:** `.dev-ports.json` tracks auto-assigned ports

`npm run prepare-db` runs `cargo sqlx migrate run` + `cargo sqlx prepare` with a temporary SQLite file to keep `sqlx-data.json` fresh for offline builds.

## TypeScript Surface

### Shared Types Generation
- **Source:** Rust structs annotated with `#[derive(TS)]`
- **Output:** `shared/types.ts` (auto-generated, not manually edited)
- **Command:** `npm run generate-types`
- **Consumers:** Frontend (automagik-forge), CLI tools

`shared/types.ts` is generated from Rust structs/enums via `npm run generate-types` (runs `cargo run --bin generate_types`). Automagik Forge vendors this file, so regenerate + commit whenever backend schemas change.

### Type Safety Pipeline
```
Rust Struct → ts-rs → shared/types.ts → Frontend Import
```

## CLI Distribution

### NPX Package Structure
- **CLI Entry:** `npx-cli/bin/cli.js`
- **Bundled Binaries:**
  - `npx-cli/dist/server` (Axum API)
  - `npx-cli/dist/mcp-task-server` (MCP server)
- **Build:** `pnpm run build:npx` compiles Rust binaries + packages for npm
- **Distribution:** `npm pack` inside `npx-cli/` → publishable `.tgz`

`npx-cli/` contains the CLI wrapper published to npm. `pnpm run build:npx` (aka `./local-build.sh`) compiles `server` + `mcp_task_server`, zips them (`automagik-forge.zip` & `automagik-forge-mcp.zip`), and drops the artifacts into `npx-cli/dist`. Running `npm pack` inside `npx-cli/` produces `automagik-forge-*.tgz` that Automagik Forge's desktop installer downloads.

### Local Build Flow
```bash
./local-build.sh →
  cargo build --release →
  copy binaries to npx-cli/dist/ →
  npm pack in npx-cli/
```

## Developer Tooling

### Core Commands
```bash
# Development
pnpm run dev                 # Start backend with auto-port allocation
npm run backend:dev:watch    # Watch-mode dev server (RUST_LOG=debug)

# Build & Test
npm run backend:check        # Cargo check
npm run backend:lint         # Clippy with strict warnings
cargo test --workspace       # Run all tests

# Type Generation
npm run generate-types       # Regenerate shared/types.ts
npm run generate-types:check # Verify types without writing

# Database
npm run prepare-db           # Prepare SQLx offline metadata

# CLI Packaging
pnpm run build:npx           # Build + bundle CLI package
npm run test:npm             # Test npm package installation
```

### Setup & Workflow
- `./setup.sh` installs pnpm dependencies and required Rust tools (`cargo-watch`, `sqlx-cli`) plus sets the correct GitHub remote.
- `pnpm run dev` sets `FORGE_INSTALLATION_MODE=development`, allocates ports via `scripts/setup-dev-environment.js`, copies `dev_assets_seed`, and launches the backend.
- `npm run backend:dev:watch` runs `cargo watch -w crates -x 'run --bin server'` with verbose logs; respects `DISABLE_WORKTREE_ORPHAN_CLEANUP=1` to keep debugging sessions alive.
- `npm run backend:check` / `npm run backend:lint` alias `cargo check` and `cargo clippy --workspace --all-features`.
- `cargo test --workspace` covers Rust crates end-to-end; run after significant backend changes.
- `pnpm run build:npx` + `npm pack` create distributable CLI bundles; always capture the produced filenames inside release notes.

### Environment Variables
- **`BACKEND_PORT`** - Server port (default: auto-assign)
- **`HOST`** - Server host (default: 127.0.0.1)
- **`GITHUB_CLIENT_ID`** - GitHub OAuth client ID (optional)
- **`FORGE_INSTALLATION_MODE`** - `development` or `production`
- **`DISABLE_WORKTREE_ORPHAN_CLEANUP`** - `1` to disable automatic cleanup
- **`RUST_LOG`** - Log level (e.g., `debug`, `info`, `warn`)

### Port Management
- **Auto-Allocation:** `scripts/setup-dev-environment.js` finds free ports
- **Persistence:** `.dev-ports.json` stores allocated ports
- **Collision Handling:** Retries on occupied ports

## Git Worktree Architecture

### Isolation Strategy
- **Task Root:** `/var/tmp/automagik-forge-dev/worktrees/<task_id>`
- **Branch Per Task:** `forge/<task-slug>` or `feature/<task-slug>`
- **Lifecycle:** Create → Execute → PR → Cleanup
- **Orphan Detection:** Automatic cleanup of stale worktrees (unless `DISABLE_WORKTREE_ORPHAN_CLEANUP=1`)

### Worktree Commands (Internal)
```bash
# Create (handled by Forge)
git worktree add /var/tmp/automagik-forge-dev/worktrees/<id> -b <branch>

# Cleanup (handled by Forge)
git worktree remove /var/tmp/automagik-forge-dev/worktrees/<id>
```

## Observability

### Logging
- **Framework:** `tracing` + `tracing-subscriber`
- **Configuration:** `RUST_LOG` environment variable
- **Dev Default:** `RUST_LOG=debug` in watch mode

`tracing` + `tracing-subscriber` manage structured logs with `RUST_LOG` toggles. Default dev command sets `RUST_LOG=debug` for verbose streaming to Automagik Forge.

### Port Files
- **Implementation:** `crates/utils/src/port_file.rs`
- **Purpose:** Track allocated ports for server + MCP server
- **Location:** `.dev-ports.json`

Port + path helpers live in `crates/utils/src/port_file.rs` and `crates/utils/src/path.rs`, ensuring logs point to deterministic directories (e.g., `/var/tmp/automagik-forge-dev/worktrees/...` on macOS).

## Build Profiles

### Development
- Default cargo profile
- Debug symbols enabled
- Fast compilation

### Release
```toml
[profile.release]
debug = true              # Include debug info
split-debuginfo = "packed" # Bundle debug symbols
strip = true              # Strip unnecessary symbols
```

## Testing & Validation

### Test Suites
- **Workspace Tests:** `cargo test --workspace`
- **CLI Smoke Test:** `./test-npm-package.sh`
- **Integration Tests:** Located in `crates/*/tests/`

### Pre-Commit Hooks
- Token efficiency checks
- Cross-reference validation
- Worktree isolation verification

## Frontend Coordination

### Sibling Repository
- **Path:** `../automagik-forge` (if cloned side-by-side)
- **Branch:** `main` (frontend stable branch)
- **Dependency:** Consumes `automagik-forge` npm package + `shared/types.ts`

### Integration Points
- **REST API:** Frontend consumes Axum routes
- **TypeScript Types:** Frontend imports from `shared/types.ts`
- **CLI Bundle:** Frontend can trigger backend via `npx automagik-forge`

### Breaking Change Protocol
1. Backend proposes schema change via PR
2. Regenerate `shared/types.ts` with `npm run generate-types`
3. Frontend validates against new types
4. Backend merges only after frontend approval
5. Coordinated release (backend publish → frontend dependency update)

## External Integrations
- **GitHub OAuth:** Optional `GITHUB_CLIENT_ID` (frontends fallback to default if unset).
- **CLI Consumers:** Automagik Forge desktop/CLI downloads zipped binaries from this repo's npm artifacts and expects matching versions between backend + frontend.
- **IDE MCP Clients:** Tools such as VS Code, Cursor, and custom agents connect over the MCP server that runs alongside the backend binary.

This stack keeps backend + CLI artifacts in one place, ensuring Automagik Forge can upgrade confidently while backend engineers iterate through Rust-first workflows.
