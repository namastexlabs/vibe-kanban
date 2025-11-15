---
name: install
description: Code Collective install workflow — prepare product docs and wire up Code agents

---

# Code Install Workflow

Purpose: Initialize the project’s product docs and connect Code collective agents and workflows.
Forge Core is the backend engine for Automagik Forge (the sibling repository at `../automagik-forge`). Every install must keep both repos synchronized: capture backend architecture truth, flag API or DB constraints before Automagik Forge consumes new binaries, and emphasize incremental upgrades over breaking changes.

## Forge Core Installation Goals
- **Backend Charter:** Translate the context from Master Genie into `.genie/product/{mission,mission-lite,tech-stack,environment,roadmap}.md` so they clearly state that Forge Core is the Rust backend, MCP task server, and CLI distribution channel for Automagik Forge.
- **Coupling Map:** Document how `shared/types.ts`, SQLx migrations (`crates/db/migrations`), and CLI bundles (`npx-cli/dist/*`) flow into Automagik Forge releases. Spell out regeneration commands (`npm run generate-types`, `pnpm run build:npx`).
- **Compatibility Guardrails:** Highlight policies for migrations and API changes (e.g., no breaking changes without synced Automagik Forge release). Capture environment scripts like `scripts/setup-dev-environment.js`, `scripts/prepare-db.js`, and the auto-port allocation story.
- **Developer Runbook:** Ensure environment doc lists canonical commands (`pnpm run dev`, `npm run backend:check`, `cargo test --workspace`, `npm run prepare-db`) plus required tooling (Rust, pnpm, GitHub OAuth). Explain how dev assets seed DBs and how to coordinate with sibling repo changes.
- **Next-Step Primer:** Tee up follow-up wishes (e.g., migration automation, shared-types CI) rather than performing breaking work during install.

## Phases

1) Discovery
- Detect repository state (fresh vs existing codebase)
- Identify domain, constraints, and intended tech stack
- Choose path: Analyze Existing • New Repo Interview • Hybrid
- For Forge Core, inventory Rust workspace crates, SQLx migrations, shared TypeScript bindings, and the sibling Automagik Forge repo so coupling is explicit before changes begin.

2) Implementation
- Create/update product docs:
  - `@.genie/product/mission.md`
  - `@.genie/product/mission-lite.md`
  - `@.genie/product/tech-stack.md`
  - `@.genie/product/roadmap.md`
  - `@.genie/product/environment.md`
- Each doc needs a Forge Core specific section: call out crate layout, MCP services, CLI bundles, Automagik Forge compatibility promises, release cadence, and validated commands (`pnpm run dev`, `npm run generate-types`, `npm run prepare-db`, `pnpm run build:npx`).
- Calibrate Code agents by adding a short "Project Notes" section inside relevant `.genie/code/agents/*` or `.genie/spells/*` docs (no `custom/` folder)
- Initialize `.genie/CONTEXT.md` and add `.genie/CONTEXT.md` to `.gitignore`
- Keep edits under `.genie/` (no app code changes here)

3) Verification
- Validate cross-references and required sections in product docs
- Exercise MCP tools: `mcp__genie__list_agents` and a sample Code agent invocation
- Capture a Done Report and hand off to `code/wish` for the first scoped feature

## Context Auto-Loading
@.genie/product/mission.md
@.genie/product/tech-stack.md
@.genie/product/environment.md
@.genie/product/roadmap.md
@README.md
@package.json

## Modes

Mode 1: Codebase Analysis
- Map structure, languages/frameworks, dependencies
- Identify architecture patterns and external integrations
- Summarize implementation progress and testing approach

Mode 2: New Repository Interview
Use a concise Q&A to capture missing product identity and technical intent.

Mode 3: Hybrid
Analyze what exists, interview for the rest, reconcile discrepancies.

Mode 4: Bootstrap Guardrails (No Code Changes)
- Only write to `.genie/`
- Defer app scaffolding to a `code/wish` → `code/forge` cycle

## Outputs

Product docs populated with stable headings so downstream tools can parse consistently. Example sections:
- mission: Pitch, Users, Problem, Key Features
- tech-stack: Core Technologies, Architecture, Dependencies, Infrastructure
- environment: Required/Optional vars + Setup instructions
- roadmap: Phase 0 (completed), Phase 1 goals and measurable criteria
- Document the Automagik Forge coupling explicitly: backend release process, migration discipline, shared-types regeneration, CLI bundle packaging, and how `scripts/setup-dev-environment.js` coordinates ports/assets.

User context file:
- `.genie/CONTEXT.md` created and git-ignored

Done Report:
- `.genie/wishes/<slug>/reports/done-install-code-<timestamp>.md`

## Forge Core Discovery Checklist
- `Cargo.toml` workspace members and binary targets (server, executors, services, utils, db, local-deployment, deployment, vendor/codex).
- `pnpm` scripts + helpers: `pnpm run dev`, `npm run backend:check`, `npm run prepare-db`, `pnpm run build:npx`, `scripts/setup-dev-environment.js`.
- Database + migrations: SQLx offline mode, `dev_assets_seed`, latest migration names, policy for rolling upgrades.
- Shared surface: `shared/types.ts` (generated from `crates/server/src/bin/generate_types.rs`), `npx-cli/dist` artifacts, MCP server definitions.
- Sibling repo: current `../automagik-forge` branch/version, how it consumes CLI bundles and shared types, what features depend on backend APIs.
- Risk ledger: note pending migrations or API changes that require cross-repo coordination; convert risky changes into future wishes rather than editing code during install.

## Success Criteria
- Product docs complete and coherent
- Context file present and ignored
- Code agents discoverable via MCP/CLI
- Clear next step: `code/wish` → `code/forge` → `code/review`
- Automagik Forge compatibility + release coupling captured (shared types, migrations, CLI bundles, sibling repo branch/version)

## Safety
- Do not modify application code
- Keep changes minimal, targeted, and reviewable
