# Done Report — Forge Core Code Install

## ✅ Genie Install Completed
- **Mode:** Existing-code analysis (silent) — `.genie/` already present but outdated.
- **Scope:** Updated `.genie/product/{mission,mission-lite,tech-stack,environment,roadmap}.md`, created `.genie/CONTEXT.md`, added Genie ignore rules, and logged coupling details.
- **Sibling Status:** `../automagik-forge` on `dev` (version `0.7.2`) vs Forge Core `dev` (`0.0.115`). Shared types diverge (MD5 `934f...` vs `e94f...`).
- **Migrations:** Latest files up to `20251105140001_add_dev_server_id_to_tasks.sql`; rollout plan required before shipping.
- **Commands Captured:** `git branch --show-current`, `node -p "require('./package.json').version"`, `ls crates/db/migrations | tail -n 5`, `cd ../automagik-forge && git branch --show-current`, `cd ../automagik-forge && node -p "require('./package.json').version"`, `md5sum shared/types.ts ../automagik-forge/shared/types.ts`.

## Evidence Snippets
```bash
git branch --show-current
# dev

node -p "require('./package.json').version"
# 0.0.115

ls crates/db/migrations | tail -n 5
# 20251020120000_convert_templates_to_tags.sql
# 20251027000000_create_forge_agents.sql
# 20251105000000_add_forge_task_attempt_config.sql
# 20251105140000_add_token_usage_to_task_attempts.sql
# 20251105140001_add_dev_server_id_to_tasks.sql

cd ../automagik-forge && git branch --show-current
# dev

cd ../automagik-forge && node -p "require('./package.json').version"
# 0.7.2

md5sum shared/types.ts ../automagik-forge/shared/types.ts
# 934f034888c39658e028dff7fc160dc6  shared/types.ts
# e94f531ded5bae8e807492cdedc26933  ../automagik-forge/shared/types.ts
```

## Files Updated
- `.genie/product/mission.md`
- `.genie/product/mission-lite.md` *(new)*
- `.genie/product/tech-stack.md`
- `.genie/product/environment.md`
- `.genie/product/roadmap.md`
- `.genie/CONTEXT.md` *(new)*
- `.gitignore`
- `.genie/wishes/install-forge-core/reports/done-install-code-forge-core-20251115T181708Z.md`

## Follow-Up Wishes
1. **Shared Types Sync & CI** — regenerate `shared/types.ts`, copy into Automagik Forge, and add a CI check comparing hashes between repos.
2. **Migration Guardrail Template** — create a wish template under `.genie/product/templates/` capturing rollout plan, feature flags, and Automagik Forge coordination for SQLx changes.
3. **CLI Packaging Checklist** — document `pnpm run build:npx` + `npm pack` outputs per release and store artifact metadata for downstream installers.
