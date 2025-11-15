# Forge Core Technical Stack

## Workspace Overview
- **Languages:** Rust (stable toolchain) + TypeScript for tooling.
- **Package Manager:** `pnpm` for Node scripts, Cargo workspace for Rust crates.
- **Structure:** `crates/` (server, db, executors, services, utils, deployment, local-deployment, vendor/codex), `shared/` (generated TS bindings), `npx-cli/` (CLI wrappers), `scripts/` (dev tooling), `dev_assets_seed/` (seed DB + assets).

## Rust Workspace
- **Server Binary:** `crates/server/src/bin/server.rs` (Axum + tower-http) exposes REST endpoints for tasks, attempts, assets, and file diffs.
- **MCP Task Server:** `crates/server/src/bin/mcp_task_server.rs` exposes MCP-compatible streams so IDE agents can talk to Forge without the desktop app.
- **Executors:** `crates/executors` defines MCP integration defaults (see `default_mcp.json`) and delegates work to provider-specific runners.
- **Services:** `crates/services` handles worktree management, repo orchestration, auth, and task bookkeeping.
- **Database Models:** `crates/db` contains SQLx models + migrations (SQLite dev DB, Postgres-ready schemas). Run `npm run prepare-db` to regenerate SQLx data before compiling offline.
- **Utilities:** `crates/utils` houses path + port helpers. Worktree temp dirs follow `automagik-forge` naming; port files under `$TMPDIR/automagik-forge/automagik-forge.port`.

## Task Execution & MCP
- API routes such as `crates/server/src/routes/task_attempts.rs` manage attempt lifecycle, port allocations, commit metadata, and diff presentation.
- MCP task server streams logs/output through JSON-RPC; `crates/server/src/mcp/task_server.rs` references the same worktree manager as the REST API to keep behavior consistent.
- Default MCP registration lives in `crates/executors/default_mcp.json` and is bundled inside CLI builds.

## Database & Assets
- Migrations live in `crates/db/migrations`; latest applied timestamps inform Automagik Forge release readiness. Never skip the migration ledger when shipping features.
- `dev_assets_seed/` contains canonical SQLite DB + asset files. `scripts/setup-dev-environment.js` copies the seed into `dev_assets/` whenever dev ports are allocated.
- `npm run prepare-db` runs `cargo sqlx migrate run` + `cargo sqlx prepare` with a temporary SQLite file to keep `sqlx-data.json` fresh for offline builds.

## TypeScript Surface & CLI Bundles
- `shared/types.ts` is generated from Rust structs/enums via `npm run generate-types` (runs `cargo run --bin generate_types`). Automagik Forge vendors this file, so regenerate + commit whenever backend schemas change.
- `npx-cli/` contains the CLI wrapper published to npm. `pnpm run build:npx` (aka `./local-build.sh`) compiles `server` + `mcp_task_server`, zips them (`automagik-forge.zip` & `automagik-forge-mcp.zip`), and drops the artifacts into `npx-cli/dist`.
- Running `npm pack` inside `npx-cli/` produces `automagik-forge-*.tgz` that Automagik Forge’s desktop installer downloads.

## Development Tooling & Commands
- `./setup.sh` installs pnpm dependencies and required Rust tools (`cargo-watch`, `sqlx-cli`) plus sets the correct GitHub remote.
- `pnpm run dev` sets `FORGE_INSTALLATION_MODE=development`, allocates ports via `scripts/setup-dev-environment.js`, copies `dev_assets_seed`, and launches the backend.
- `npm run backend:dev:watch` runs `cargo watch -w crates -x 'run --bin server'` with verbose logs; respects `DISABLE_WORKTREE_ORPHAN_CLEANUP=1` to keep debugging sessions alive.
- `npm run backend:check` / `npm run backend:lint` alias `cargo check` and `cargo clippy --workspace --all-features`.
- `cargo test --workspace` covers Rust crates end-to-end; run after significant backend changes.
- `npm run generate-types` / `npm run generate-types:check` keep `shared/types.ts` synced with backend models.
- `npm run prepare-db` must be run after editing SQL to update `sqlx-data.json`.
- `pnpm run build:npx` + `npm pack` create distributable CLI bundles; always capture the produced filenames inside release notes.

## Observability & Logging
- `tracing` + `tracing-subscriber` manage structured logs with `RUST_LOG` toggles. Default dev command sets `RUST_LOG=debug` for verbose streaming to Automagik Forge.
- Port + path helpers live in `crates/utils/src/port_file.rs` and `crates/utils/src/path.rs`, ensuring logs point to deterministic directories (e.g., `/var/tmp/automagik-forge-dev/worktrees/...` on macOS).

## External Integrations
- **GitHub OAuth:** Optional `GITHUB_CLIENT_ID` (frontends fallback to default if unset).
- **CLI Consumers:** Automagik Forge desktop/CLI downloads zipped binaries from this repo’s npm artifacts and expects matching versions between backend + frontend.
- **IDE MCP Clients:** Tools such as VS Code, Cursor, and custom agents connect over the MCP server that runs alongside the backend binary.

This stack keeps backend + CLI artifacts in one place, ensuring Automagik Forge can upgrade confidently while backend engineers iterate through Rust-first workflows.
