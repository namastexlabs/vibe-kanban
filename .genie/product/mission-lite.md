# Forge Core Mission • Lite

## Pitch
Forge Core is Automagik Forge’s backend engine — a Rust + TypeScript workspace that orchestrates Git worktrees, streams MCP sessions, and packages the binaries that the UI/CLI downloads. Every release is engineered to be backward compatible so the sibling repo can upgrade without firefighting.

## Who We Serve
- Forge maintainers who need reliable task orchestration and worktree isolation.
- Release engineers who publish Automagik Forge builds from the `npx-cli` artifacts.
- Agent integrators who depend on stable REST + MCP contracts and generated TypeScript types.

## What We Ship
- Axum API (`crates/server`) + MCP task server used by Automagik Forge and IDE integrations.
- SQLx models/migrations (`crates/db`) with offline preparation via `npm run prepare-db`.
- Shared types (`shared/types.ts`) generated from Rust structs through `npm run generate-types`.
- CLI bundles zipped by `pnpm run build:npx`, later consumed by Automagik Forge’s npm distribution.

## Release Principles
1. No breaking schema or API changes without a matching Automagik Forge rollout plan.
2. Regenerate shared types and document deltas whenever backend models change.
3. Package binaries + MCP server in lockstep and publish the exact commands used.
4. Keep developer ergonomics simple: `./setup.sh`, `pnpm run dev`, `cargo test --workspace`.

## Current Focus
- Harden task attempt + MCP telemetry before exposing new agents.
- Automate shared-types + CLI build verification in CI.
- Plan migration guardrails so Automagik Forge UI can roll forward safely.
