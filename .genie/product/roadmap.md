# Forge Core Roadmap

Forge Core iterates in lockstep with `../automagik-forge`. Every phase describes backend goals **plus** the coordination steps required to keep the sibling repo unbroken.

## Phase 0 — Production Backbone (✅ complete)

**Evidence:** 523 commits, v0.0.115, MCP protocol negotiation operational

### What We Shipped
- Fork stabilized from `BloopAI/vibe-kanban`; Axum API + MCP server power Automagik Forge attempts in production.
- Worktree isolation, task orchestration, and CLI packaging flow established (`pnpm run build:npx` + `npm pack`).
- Seed assets + dev scripts (`scripts/setup-dev-environment.js`) standardized so contributors share deterministic ports + data.
- Rust workspace architecture (7 crates + vendored codex)
- SQLx database with migrations (latest: `20251020000001_add_agent_task_status.sql`)
- TypeScript type generation (`ts-rs`) for frontend integration
- Multi-executor support (Claude Code, Codex, Gemini, Cursor, OpenCode)

### Validation
Current Automagik Forge release (`../automagik-forge` @ `dev`, version `0.7.2`) runs against Forge Core `dev` (package `0.0.115`) without compatibility issues aside from the shared types drift noted below.

## Phase 1 — Schema & Type Discipline (🚧 in progress)

**Goal:** Non-breaking backend evolution with frontend coordination

### Active Work
- 🚧 Schema migration guardrails (require frontend approval before breaking changes)
- 🚧 Shared types CI validation (fail if `shared/types.ts` drifts from Rust structs)
- 🚧 API versioning strategy (prepare for v1 endpoint stability)

### Implementation Plan
- Add checklists for every SQLx migration: document rollout plan, run `npm run prepare-db`, and capture the generated timestamp in release notes.
- Keep `shared/types.ts` regenerated and copied to Automagik Forge whenever backend structs change; CI should fail if the sibling repo lags.
- Define feature-flag strategy for potentially breaking columns (dual-write or read-fallback) until Automagik Forge updates.
- Automated type generation checks in pre-commit hooks
- Migration rollback safety (test rollback scripts before merge)

### Success Criteria
- Zero uncoordinated breaking changes shipped to frontend
- `npm run generate-types:check` passes in CI
- Migration rollback tested for last 3 migrations

### Dependencies
Wish to automate shared-types verification + add migration template in `.genie/templates`.

## Phase 2 — MCP & Task Runtime Hardening (🔜 queued)

**Goal:** Richer task lifecycle, better concurrency, clearer observability

### Planned Features
- Expand coverage for `crates/server/src/mcp/task_server.rs` and task attempt routes: load testing, telemetry, and protocol versioning.
- Document MCP capabilities consumed by IDE clients; add contract tests so CLI + Automagik Forge share the same expectations.
- Improve worktree cleanup/resume logic inside `crates/services/src/services/worktree_manager.rs` so multi-agent runs survive restarts.
- **Task Dependencies:** DAG-based task execution (task B waits for task A)
- **Parallel Execution Limits:** Configurable concurrency caps per project
- **Enhanced Status Tracking:** Granular sub-task progress reporting
- **Webhook Notifications:** Callback URLs for task completion events
- **Session Replay:** Store + replay agent session transcripts

### Technical Requirements
- Non-blocking: All features behind feature flags until stable
- Frontend coordination: New endpoints documented + typed before implementation
- Migration strategy: Additive schema changes only

### Success Criteria
- Task dependency execution demonstrates correct ordering
- Parallel execution respects concurrency limits
- Webhook delivery has <1% failure rate

### Coordination
Automagik Forge UI must be notified before new MCP methods land; include sample clients in release notes.

## Phase 3 — Release Automation & Observability (🗺 planned)

**Goal:** Handle higher throughput, optimize resource usage, automate releases

### Optimization Targets
- Script `pnpm run build:npx` + `npm pack` verification in CI, upload artifacts, and gate merges on successful packaging.
- Publish release playbooks that include: version bump steps, shared-types sync instructions, migration status, and CLI artifact hashes.
- Add structured tracing/metrics (per-attempt throughput, MCP latency) surfaced through API endpoints for dashboards.
- **Database Query Performance:** Index optimization for common queries
- **Worktree Management:** Faster creation/cleanup via parallel operations
- **Memory Footprint:** Profile + reduce allocations in hot paths
- **Log Volume:** Structured logging with sampling for high-frequency events

### Metrics
- Target: 50 concurrent task attempts per backend instance
- Target: <500ms p95 latency for task creation API
- Target: <2s worktree initialization time

### Success Criteria
- Load test demonstrates 50 concurrent tasks without degradation
- Memory usage stable under sustained load
- No orphaned worktrees after 24hr stress test

### Outcome
Backend + frontend ship from the same checklist, reducing human toil and ensuring reproducible builds.

## Phase 4 — Ecosystem Integration (🗺 planned)

**Goal:** Broader agent ecosystem support, plugin architecture

### Planned Integrations
- **New Executors:** Cody, Aider, Continue (via MCP)
- **Custom Executor API:** Plugin system for third-party agent integrations
- **Git Provider Abstraction:** Support GitLab, Bitbucket (beyond GitHub)
- **Cloud Deployment:** Docker + Kubernetes deployment configurations

### Technical Requirements
- Executor plugin spec (input/output contracts)
- Git provider trait abstraction
- Containerization without breaking npx CLI workflow

### Success Criteria
- At least 2 community-contributed executor plugins operational
- GitLab repository support validated
- Docker deployment documented + tested

## Success Metrics (Overall)

### Quality Gates
- 100% of migrations have rollback tests
- `shared/types.ts` always synced with Rust structs (CI enforced)
- Zero breaking API changes without frontend approval

### Release Cadence
- Patch releases (bug fixes): weekly
- Minor releases (new features): bi-weekly
- Major releases (breaking changes): coordinated with frontend quarterly

### Frontend Compatibility
- Frontend dependency update within 24hrs of backend release
- <5% rollback rate on coordinated releases
- TypeScript compilation errors = 0 after type regeneration

## Guardrails & Coordination Notes

- **No breaking changes alone:** Automagik Forge consumes these binaries; create a wish → forge cycle that touches both repos or ships behind flags.
- **Shared Types Source of Truth:** Always update `shared/types.ts` here first, then copy into the sibling repo. Note MD5 hashes in done reports.
- **CLI Artifact Registry:** Keep `npx-cli/dist` tidy and document which zips belong to which release.

## Dependencies & Enablers

### Required Infrastructure
- CI/CD pipeline (GitHub Actions) for automated testing
- Staging environment for frontend integration testing
- Version tagging automation (aligned with npm publish)
- Dedicated owner for SQLx migrations & type generation
- CI agents capable of running Rust + Node builds plus packaging (macOS or Linux)
- Access to Automagik Forge repo to copy shared types and validate CLI installs

### Team Coordination
- Weekly sync with frontend team for breaking change planning
- Migration review process (backend + frontend sign-off)
- Release notes automation (changes → CHANGELOG.md)

## Risk Log (actively monitored)

### Technical Risks
- **Database Migration Failures:** → Mitigate with rollback tests + staging validation
- **MCP Protocol Changes:** Codex updates may break compatibility → Mitigate with version negotiation + fallback
- **Worktree Orphans:** Cleanup failures leave stale workspaces → Mitigate with orphan detection + manual cleanup scripts
- **Type Generation Drift:** Rust changes not reflected in `shared/types.ts` → Mitigate with CI checks on every PR

### Coordination Risks
- **Frontend Breaking Changes:** Backend changes land before frontend ready → Mitigate with approval gates + phased rollout
- **Release Timing Misalignment:** Backend publishes, frontend delays → Mitigate with coordinated release schedule
- **Migration Coordination:** Schema changes require simultaneous frontend update → Mitigate with backward-compatible migrations by default

### Active Risks
- **Shared types drift:** Current hashes differ (`934f...` vs `e94f...`). Mitigation: regenerate + sync before the next Automagik Forge release.
- **Migration sprawl:** Multiple pending migrations (ending `20251105140001`) require a published rollout plan; create a wish to bundle them.
- **Packaging knowledge silo:** CLI build steps currently live in maintainer memory; Phase 3 formalizes them.

## Non-Breaking Policy (Critical)

### Rules
1. **Schema changes:** Additive only unless frontend approves breaking change
2. **API endpoints:** Deprecate old before removing (versioned endpoints)
3. **TypeScript types:** Regenerate + validate before frontend integration
4. **Migrations:** Test rollback before merge

### Approval Gates
- **Breaking Change:** Frontend team must review + approve PR
- **Migration:** Backend + frontend leads sign off on timing
- **Major Version Bump:** Coordinated release plan required

## Future Vision (Phase 5+)

### Exploration Areas
- **Multi-Tenancy:** Isolated environments per user/team
- **Distributed Execution:** Task execution across multiple backend nodes
- **Agent Marketplace:** Community-contributed executor + plugin registry
- **Real-Time Collaboration:** Live editing via WebSocket multiplexing

### Research Questions
- Can we support 1000+ concurrent tasks with current architecture?
- What's the limit of worktree-based isolation at scale?
- How do we handle cross-repository task dependencies?
