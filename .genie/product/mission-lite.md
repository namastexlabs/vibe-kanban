# Forge Core Mission • Lite

## Pitch
Forge Core is Automagik Forge's backend engine — a Rust + TypeScript workspace that orchestrates Git worktrees, streams MCP sessions, and packages the binaries that the UI/CLI downloads. Every release is engineered to be backward compatible so the sibling repo can upgrade without firefighting.

## Who We Serve
- Forge maintainers who need reliable task orchestration and worktree isolation.
- Release engineers who publish Automagik Forge builds from the `npx-cli` artifacts.
- Agent integrators who depend on stable REST + MCP contracts and generated TypeScript types.

## What We Ship
- Axum API (`crates/server`) + MCP task server used by Automagik Forge and IDE integrations.
- SQLx models/migrations (`crates/db`) with offline preparation via `npm run prepare-db`.
- Shared types (`shared/types.ts`) generated from Rust structs through `npm run generate-types`.
- CLI bundles zipped by `pnpm run build:npx`, later consumed by Automagik Forge's npm distribution.

## Tech Stack
- **Rust:** Axum API, SQLx database, MCP server
- **TypeScript:** Auto-generated types from Rust structs
- **Distribution:** NPM package with bundled Rust binaries

## Current Phase
**Phase 0 Complete** (523 commits, v0.0.115)
- ✅ Rust workspace (7 crates + vendored codex)
- ✅ REST API + MCP task server
- ✅ Worktree isolation
- ✅ Type generation
- ✅ CLI bundling

**Phase 1 In Progress:** Stability & Type Safety
- 🚧 Schema migration guardrails
- 🚧 Shared types CI validation
- 🚧 API versioning

## Quick Start
```bash
./setup.sh      # One-time setup
pnpm run dev    # Start backend
```

## Key Commands
```bash
pnpm run dev                 # Development server
npm run generate-types       # Regenerate TypeScript types
npm run prepare-db           # Prepare database metadata
pnpm run build:npx           # Build CLI package
cargo test --workspace       # Run tests
```

## Release Principles
1. No breaking schema or API changes without a matching Automagik Forge rollout plan.
2. Regenerate shared types and document deltas whenever backend models change.
3. Package binaries + MCP server in lockstep and publish the exact commands used.
4. Keep developer ergonomics simple: `./setup.sh`, `pnpm run dev`, `cargo test --workspace`.

## Non-Breaking Policy
- Schema changes require frontend approval
- TypeScript types regenerated before merge
- Migration rollback tested
- Coordinated releases with frontend

## Frontend Coordination
- **Backend Version:** 0.0.115
- **Latest Migration:** `20251020000001_add_agent_task_status.sql`
- **Shared Types:** `shared/types.ts` (auto-generated)
- **Handshake:** Backend PR → Frontend validates → Backend merges → Frontend updates

## Differentiators
1. **Rust Performance:** Axum async efficiency
2. **MCP-Native:** First-class protocol support
3. **Worktree-Based:** Guaranteed isolation
4. **Self-Contained CLI:** Single `npx` command bundles everything

## Current Focus
- Harden task attempt + MCP telemetry before exposing new agents.
- Automate shared-types + CLI build verification in CI.
- Plan migration guardrails so Automagik Forge UI can roll forward safely.

## Links
- **Full Mission:** `.genie/product/mission.md`
- **Tech Stack:** `.genie/product/tech-stack.md`
- **Roadmap:** `.genie/product/roadmap.md`
- **Environment:** `.genie/product/environment.md`
