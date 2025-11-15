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

## Troubleshooting
- **Port already in use:** delete `.dev-ports.json` and rerun `pnpm run dev`, or export `BACKEND_PORT` manually before running commands.
- **SQLx compile errors:** rerun `npm run prepare-db` to refresh `sqlx-data.json` after touching migrations or schema definitions.
- **CLI build mismatch:** ensure `pnpm run build:npx` finished successfully, then verify `npx-cli/dist/automagik-forge*.zip` timestamps before running `npm pack`.
- **Shared types drift:** compare `md5sum shared/types.ts ../automagik-forge/shared/types.ts`. If they differ, regenerate types in Forge Core and copy into the sibling repo as part of the release plan.

Forge Core’s environment is optimized for reproducibility: the same commands drive local dev, CI validation, and Automagik Forge releases. Keep this runbook updated whenever workflows change.
