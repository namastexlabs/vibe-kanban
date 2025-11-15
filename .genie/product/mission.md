# Forge Core Mission

## Pitch

Forge Core is the Rust + TypeScript backend that powers every Automagik Forge session. It brokers task orchestration, isolates Git worktrees for each attempt, exposes MCP task servers for coding agents, and packages the binaries that the Automagik Forge desktop/CLI downloads via `pnpm run build:npx`. The mandate is simple: evolve the backend architecture version after version **without breaking** the sibling repo at `../automagik-forge`.

## Users & Stakeholders

### Primary Customers
- **Forge Maintainers:** Need a predictable backend that can scale task throughput, capture telemetry, and expose new agent capabilities without surprising downstream repos.
- **Release & Distribution Engineers:** Cut new Automagik Forge builds, so they depend on Forge Core to publish reproducible server + MCP bundles and regenerated TypeScript bindings.
- **Task Operators / MCP Integrators:** Rely on the Axum API and MCP websocket servers to run attempts headlessly or from IDE extensions.

### Representative Personas

**Forge Orchestrator (Lead Maintainer)**
- Runs the kanban, triages bugs, and merges backend improvements.
- Priorities: operational transparency, worktree hygiene, deterministic migrations.

**Desktop Release Engineer**
- Publishes Automagik Forge releases and CLI packages.
- Priorities: one-command packaging, clear version bumps, zero backend surprises when shipping the UI.

**Task Ops Lead**
- Embeds Forge inside IDEs and automation.
- Priorities: stable MCP contracts, documented environment variables, scripts for preparing SQLx caches.

**Frontend Developer**
- Builds web UI for task orchestration and monitoring.
- Needs stable REST API contracts and real-time task status updates.
- Goals: Predictable API versioning, auto-generated TypeScript types, clear migration paths.

**CLI User (Developer)**
- Runs Forge backend locally for development workflows.
- Needs simple setup, automatic port allocation, and clean teardown.
- Goals: One-command startup (`pnpm run dev`), automatic seed data, clear error messages.

**Agent Executor**
- Coding agent (Claude Code, Codex, etc.) executing tasks via MCP.
- Needs isolated worktrees, clean git state, and reliable task handoff.
- Goals: Guaranteed isolation, automatic cleanup, persistent session state.

## The Problem We Solve

### Multi-Agent Coordination
Automagik Forge delegates work to multiple agents simultaneously. Forge Core must keep their worktrees isolated, capture commit metadata, and stream logs through the API without blocking other tasks.

**Our Approach:** Dedicated git worktrees per task attempt (`/var/tmp/automagik-forge-dev/worktrees/<task_id>`), automatic cleanup on completion, and orphan detection for stale workspaces.

### Cross-Repo Safety
The frontend repo vendors shared TypeScript types, expects certain REST/MCP contracts, and downloads CLI bundles from this workspace. Backend changes can break production if we do not publish guardrails and migration kits.

**Our Approach:** Treat schema changes as breaking until proven safe, auto-generate TypeScript types from Rust structs, and coordinate migrations with frontend before deployment.

### Local Developer Ergonomics
Contributors span Rust and JavaScript contexts. They need one command to spin up the backend with seeded assets, deterministic ports, and SQLx caches ready for offline compilation.

**Our Approach:** Auto-allocate ports, copy dev seed assets (`dev_assets_seed/`), provide clear setup script (`./setup.sh`), and bundle everything via `pnpm run dev`.

## Differentiators

1. **Task-Oriented Git Worktree Manager** – `crates/services/src/services/worktree_manager.rs` handles one worktree per attempt, keeping Automagik Forge agents isolated by default.
2. **Built-In MCP Task Server** – `crates/server/src/mcp/task_server.rs` exposes long-lived MCP sessions so IDE clients can launch tasks without running the full desktop app.
3. **Single Source of Truth for Generated Types** – `crates/server/src/bin/generate_types.rs` along with `npm run generate-types` guarantees `shared/types.ts` stays aligned with backend schemas before shipping a new Automagik Forge release.
4. **Release-Ready Binary Packaging** – `pnpm run build:npx` invokes `local-build.sh`, drops zipped server + MCP artifacts into `npx-cli/dist`, and feeds the CLI wrappers that Automagik Forge publishes to npm.
5. **Port & Asset Automation** – `scripts/setup-dev-environment.js` reserves dev ports, copies `dev_assets_seed` into `dev_assets`, and keeps local instances from colliding.

## Symbiosis with Automagik Forge

The sibling repo consumes backend APIs, `shared/types.ts`, and CLI bundles. Every backend change must include a migration story, a regenerated types artifact, and packaging notes so the frontend can update deliberately.

### Non-Breaking Policy
Backend maintains API stability unless coordinated with frontend:
- Schema changes require frontend approval
- TypeScript types (`shared/types.ts`) regenerated before frontend integration
- Migration compatibility verified before merge

### Release Handshake
1. Backend creates PR with type changes
2. Frontend validates against new types
3. Backend merges only after frontend confirms compatibility
4. Frontend updates dependency after npm publish

### Current Alignment
- **Backend Branch:** `main` (stable API)
- **Backend Version:** `0.0.115` (from package.json)
- **Latest Migration:** `20251020000001_add_agent_task_status.sql`
- **Shared Types:** Auto-generated via `npm run generate-types`

### Release Cadence
Bump `package.json` + Cargo versions, run `pnpm run build:npx`, publish the npm tarballs from `npx-cli/`, then update Automagik Forge's dependency pins (CLI + shared types) before announcing the release.

### Compatibility Checklist (Before Merging)
- No breaking SQLx migrations without a feature flag or immediate Automagik Forge upgrade plan.
- MCP schemas remain backward compatible; new capabilities ship behind explicit version gates.
- `shared/types.ts` differences between repos are intentional and called out in the install report.

## Guardrails & Non-Negotiables

- **Backwards Compatibility First:** Schema and API drift must include fallback paths or version negotiation.
- **Document Every Automation:** Commands like `pnpm run dev`, `npm run prepare-db`, `npm run generate-types`, and `pnpm run build:npx` stay authoritative and appear in the environment runbook.
- **No Silent CLI Changes:** All CLI/binary packaging changes propagate to Automagik Forge through a wish → forge → release flow.
- **Telemetry with Privacy:** Tracing/logging is instrumented via `tracing` + `tracing-subscriber` with configurable levels, but never leaks secrets from customer repos.

## Current Focus Areas

- **Schema Discipline:** Track migrations under `crates/db/migrations` and gate high-risk changes with dual-write shims so Automagik Forge can roll forward without downtime.
- **Shared Types Automation:** Keep `npm run generate-types` in CI and document when the frontend must pull regenerated bindings.
- **MCP & Task Runtime Hardening:** Expand coverage for `crates/server/src/routes/task_attempts.rs` and the MCP task server so remote IDEs behave the same as the desktop app.
- **Release Automation:** Codify `pnpm run build:npx` + `npm pack` steps and publish checklists that Automagik Forge maintainers can execute with confidence.

Forge Core exists to keep Automagik Forge fast, safe, and predictable. We evolve backend architecture slowly, version after version, with zero breaking surprises for the sibling repo.
