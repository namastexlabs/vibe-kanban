# Forge Core Roadmap
Forge Core iterates in lockstep with `../automagik-forge`. Every phase describes backend goals **plus** the coordination steps required to keep the sibling repo unbroken.

## Phase 0 — Production Backbone (✅ complete)
- Fork stabilized from `BloopAI/vibe-kanban`; Axum API + MCP server power Automagik Forge attempts in production.
- Worktree isolation, task orchestration, and CLI packaging flow established (`pnpm run build:npx` + `npm pack`).
- Seed assets + dev scripts (`scripts/setup-dev-environment.js`) standardized so contributors share deterministic ports + data.
- **Evidence:** Current Automagik Forge release (`../automagik-forge` @ `dev`, version `0.7.2`) runs against Forge Core `dev` (package `0.0.115`) without compatibility issues aside from the shared types drift noted below.

## Phase 1 — Schema & Type Discipline (🚧 in progress)
- Add checklists for every SQLx migration: document rollout plan, run `npm run prepare-db`, and capture the generated timestamp in release notes.
- Keep `shared/types.ts` regenerated and copied to Automagik Forge whenever backend structs change; CI should fail if the sibling repo lags.
- Define feature-flag strategy for potentially breaking columns (dual-write or read-fallback) until Automagik Forge updates.
- **Dependencies:** Wish to automate shared-types verification + add migration template in `.genie/templates`.

## Phase 2 — MCP & Task Runtime Hardening (🔜 queued)
- Expand coverage for `crates/server/src/mcp/task_server.rs` and task attempt routes: load testing, telemetry, and protocol versioning.
- Document MCP capabilities consumed by IDE clients; add contract tests so CLI + Automagik Forge share the same expectations.
- Improve worktree cleanup/resume logic inside `crates/services/src/services/worktree_manager.rs` so multi-agent runs survive restarts.
- **Coordination:** Automagik Forge UI must be notified before new MCP methods land; include sample clients in release notes.

## Phase 3 — Release Automation & Observability (🗺 planned)
- Script `pnpm run build:npx` + `npm pack` verification in CI, upload artifacts, and gate merges on successful packaging.
- Publish release playbooks that include: version bump steps, shared-types sync instructions, migration status, and CLI artifact hashes.
- Add structured tracing/metrics (per-attempt throughput, MCP latency) surfaced through API endpoints for dashboards.
- **Outcome:** Backend + frontend ship from the same checklist, reducing human toil and ensuring reproducible builds.

## Guardrails & Coordination Notes
- **No breaking changes alone:** Automagik Forge consumes these binaries; create a wish → forge cycle that touches both repos or ships behind flags.
- **Shared Types Source of Truth:** Always update `shared/types.ts` here first, then copy into the sibling repo. Note MD5 hashes in done reports.
- **CLI Artifact Registry:** Keep `npx-cli/dist` tidy and document which zips belong to which release.

## Dependencies
- Dedicated owner for SQLx migrations & type generation.
- CI agents capable of running Rust + Node builds plus packaging (macOS or Linux).
- Access to Automagik Forge repo to copy shared types and validate CLI installs.

## Active Risks
- **Shared types drift:** Current hashes differ (`934f...` vs `e94f...`). Mitigation: regenerate + sync before the next Automagik Forge release.
- **Migration sprawl:** Multiple pending migrations (ending `20251105140001`) require a published rollout plan; create a wish to bundle them.
- **Packaging knowledge silo:** CLI build steps currently live in maintainer memory; Phase 3 formalizes them.
