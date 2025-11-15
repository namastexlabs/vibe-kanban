# Forge Core Environment Configuration

## Prerequisites
- **Rust**: latest stable toolchain (`rustup toolchain install stable`), plus `cargo-watch` and `sqlx-cli` (`cargo install cargo-watch sqlx-cli`).
- **Node.js**: v18+ with `pnpm` ≥ 8. (`corepack enable` recommended).
- **GitHub CLI** *(optional, recommended)* for auth + repo automation.
- **SQLite**: required for dev + SQLx preparation (bundled on macOS/Linux).

## Canonical Commands
| Purpose | Command | Notes |
| --- | --- | --- |
| Install deps | `./setup.sh` | Installs pnpm packages, cargo tools, and fixes `gh` remotes. |
| Dev server | `pnpm run dev` | Sets `FORGE_INSTALLATION_MODE=development`, allocates ports, copies `dev_assets_seed → dev_assets`, sets `BACKEND_PORT`, then runs the server. |
| Watch mode | `npm run backend:dev:watch` | `cargo watch -w crates -x 'run --bin server'` with `RUST_LOG=debug`. |
| Lint & check | `npm run backend:lint`, `npm run backend:check` | Wrap `cargo clippy --workspace --all-targets --all-features` and `cargo check`. |
| Tests | `cargo test --workspace` | Run before packaging or publishing. |
| SQLx prep | `npm run prepare-db` | Creates a temp SQLite DB, applies migrations, runs `cargo sqlx prepare`, and deletes the temp file. |
| Generate types | `npm run generate-types` (or `:check`) | Executes `cargo run --bin generate_types` and updates `shared/types.ts`. |
| Package CLI | `pnpm run build:npx` → `cd npx-cli && npm pack` | Builds release binaries (server + mcp) and produces `.tgz` files for Automagik Forge distribution. |

## Environment Variables
| Variable | Scope | Description |
| --- | --- | --- |
| `BACKEND_PORT` | optional | Port for the Axum server. Default: auto-assigned by `scripts/setup-dev-environment.js` (written to `.dev-ports.json`). |
| `HOST` | optional | Bind address (default `127.0.0.1`). Set to `0.0.0.0` for remote testing. |
| `GITHUB_CLIENT_ID` | optional | Custom GitHub OAuth app ID. Frontend falls back to default if unset. |
| `FORGE_INSTALLATION_MODE` | optional | `development` or `production`. `pnpm run dev` exports `development` automatically. |
| `PORT` | optional | When running the combined Automagik Forge desktop app; forcing it here cascades into frontend/backend offsets. |
| `RUST_LOG` | optional | Defaults to `debug` in watch mode; set to `info` in production to reduce noise. |
| `DISABLE_WORKTREE_ORPHAN_CLEANUP` | optional | `1` (disable) or `0`/unset (enable). Use during development to preserve worktrees for debugging. |

Store sensitive values in `.env` files; `.gitignore` already prevents them from being committed.

## Setup Workflow
1. **Clone repo & run setup**
   ```bash
   git clone git@github.com:namastexlabs/forge-core.git
   cd forge-core
   ./setup.sh
   ```

2. **Start dev server**
   ```bash
   pnpm run dev
   # prints allocated frontend/backend ports and copies dev assets
   ```

3. **Prepare SQLx data after editing migrations**
   ```bash
   npm run prepare-db
   git add crates/db/sqlx-data.json
   ```

4. **Regenerate shared types when backend structs change**
   ```bash
   npm run generate-types
   cp shared/types.ts ../automagik-forge/shared/types.ts  # when syncing with sibling repo
   ```

5. **Package binaries for Automagik Forge**
   ```bash
   pnpm run build:npx
   (cd npx-cli && npm pack)
   ```
   Artifacts land in `npx-cli/dist` and `npx-cli/*.tgz`. Record filenames in release notes.

## Dev Assets & Ports
- `scripts/setup-dev-environment.js` keeps `.dev-ports.json` at repo root, assigning sequential ports (frontend first, backend = +1). Delete the file if ports get stuck.
- The script also copies `dev_assets_seed/*` into `dev_assets/`, ensuring everyone shares the same seed DB + workspace assets.
- Worktrees live under the per-platform temp directory (see `crates/utils/src/path.rs` for the exact paths on macOS/Linux/Windows).

### Port File Structure
```json
{
  "backend": 3000,
  "mcp": 3001
}
```

### Manual Port Override
```bash
BACKEND_PORT=4000 pnpm run dev
```

## Database Configuration

### SQLite (Default)
- **Location:** `dev_assets/vibe-kanban.db` (auto-copied from seed)
- **Migrations:** Applied automatically on startup via SQLx
- **Reset:** `rm -rf dev_assets && pnpm run dev`

### Migration Workflow
```bash
# Create new migration
sqlx migrate add <name>

# Apply migrations (auto on startup)
# Manual: sqlx migrate run

# Prepare offline metadata
npm run prepare-db
```

## Troubleshooting

### Port Conflicts
```bash
# Check occupied ports
lsof -i :3000

# Reset port allocation
rm .dev-ports.json
pnpm run dev
```

### Stale Dev Assets
```bash
# Reset database and config
rm -rf dev_assets
pnpm run dev  # Auto-copies from dev_assets_seed
```

### Orphaned Worktrees
```bash
# List worktrees
git worktree list

# Remove specific worktree
git worktree remove /var/tmp/automagik-forge-dev/worktrees/<id>

# Prune stale references
git worktree prune
```

### Type Generation Drift
```bash
# Verify types match Rust structs
npm run generate-types:check

# Regenerate if drift detected
npm run generate-types
```

### CLI Build Mismatch
Ensure `pnpm run build:npx` finished successfully, then verify `npx-cli/dist/automagik-forge*.zip` timestamps before running `npm pack`.

### Shared Types Drift
Compare `md5sum shared/types.ts ../automagik-forge/shared/types.ts`. If they differ, regenerate types in Forge Core and copy into the sibling repo as part of the release plan.

### SQLx Compile Errors
Rerun `npm run prepare-db` to refresh `sqlx-data.json` after touching migrations or schema definitions.

## Frontend Integration

### Shared Types
After Rust struct changes:
```bash
# Backend
npm run generate-types        # Regenerate shared/types.ts

# Frontend (in sibling repo)
npm install                    # Pick up updated types
npm run build                  # Verify TypeScript compilation
```

### Breaking Changes Protocol
1. Backend creates PR with type changes
2. Regenerate types: `npm run generate-types`
3. Frontend validates against new types
4. Backend merges only after frontend approval
5. Coordinated release

## CI/CD Environment

### GitHub Actions
- **Secrets Required:** None (public repository)
- **Workflow Triggers:** Push to `main`, PR to `main`
- **Steps:** Build, test, type generation check, clippy lint

### Deployment
- **Manual:** `pnpm run build:npx` → `npm publish` (after approval)
- **Automated:** Planned for Phase 1 (CI/CD pipeline)

## Security Notes

- **Never commit:** `.env`, `.dev-ports.json`, `dev_assets/`, API keys
- **Gitignored:** All sensitive configuration files
- **Vendored OpenSSL:** Included for portability, no external dependency

## Conventions

- **Environment Variables:** `UPPER_SNAKE_CASE`
- **Ports:** Auto-assigned unless manually overridden
- **Logs:** Structured via `tracing` (JSON-compatible)
- **Asset Reset:** Delete `dev_assets/` to restore from seed

Forge Core's environment is optimized for reproducibility: the same commands drive local dev, CI validation, and Automagik Forge releases. Keep this runbook updated whenever workflows change.
