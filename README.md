<p align="center">
  <a href="https://vibekanban.com">
    <picture>
      <source srcset="frontend/public/automagik-forge-logo-dark.svg" media="(prefers-color-scheme: dark)">
      <source srcset="frontend/public/automagik-forge-logo.svg" media="(prefers-color-scheme: light)">
      <img src="frontend/public/automagik-forge-logo.svg" alt="Automagik Forge Logo">
    </picture>
  </a>
</p>

<p align="center">Get 10X more out of Claude Code, Gemini CLI, Codex, Amp and other coding agents...</p>
<p align="center">
  <a href="https://www.npmjs.com/package/automagik-forge"><img alt="npm" src="https://img.shields.io/npm/v/automagik-forge?style=flat-square" /></a>
  <a href="https://github.com/NamastexLabs/automagik-forge/blob/main/.github/workflows/publish.yml"><img alt="Build status" src="https://img.shields.io/github/actions/workflow/status/NamastexLabs/automagik-forge/.github%2Fworkflows%2Fpublish.yml" /></a>
  <a href="https://deepwiki.com/NamastexLabs/automagik-forge"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"></a>
</p>

![](frontend/public/automagik-forge-screenshot-overview.png)

## Overview

AI coding agents are increasingly writing the world's code and human engineers now spend the majority of their time planning, reviewing, and orchestrating tasks. Automagik Forge streamlines this process, enabling you to:

- Easily switch between different coding agents
- Orchestrate the execution of multiple coding agents in parallel or in sequence
- Quickly review work and start dev servers
- Track the status of tasks that your coding agents are working on
- Centralise configuration of coding agent MCP configs

You can watch a video overview [here](https://youtu.be/TFT3KnZOOAk).

## Installation

Make sure you have authenticated with your favourite coding agent. A full list of supported coding agents can be found in the [docs](https://vibekanban.com/docs). Then in your terminal run:

```bash
npx automagik-forge
```

## Documentation

Please head to the [website](https://vibekanban.com/docs) for the latest documentation and user guides.

## Support

We use [GitHub Discussions](https://github.com/NamastexLabs/automagik-forge/discussions) for feature requests. Please open a discussion to create a feature request. For bugs please open an issue on this repo.

## Contributing

We would prefer that ideas and changes are first raised with the core team via [GitHub Discussions](https://github.com/NamastexLabs/automagik-forge/discussions) or Discord, where we can discuss implementation details and alignment with the existing roadmap. Please do not open PRs without first discussing your proposal with the team.

## Development

### Prerequisites

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) (>=18)
- [pnpm](https://pnpm.io/) (>=8)

Additional development tools:
```bash
cargo install cargo-watch
cargo install sqlx-cli
```

Install dependencies:
```bash
pnpm i
```

### Running the dev server

```bash
pnpm run dev
```

This will start the backend. A blank DB will be copied from the `dev_assets_seed` folder.

### Building the frontend

To build just the frontend:

```bash
cd frontend
pnpm build
```

### Build from source

1. Run `build-npm-package.sh`
2. In the `npx-cli` folder run `npm pack`
3. You can run your build with `npx [GENERATED FILE].tgz`


### Environment Variables

The following environment variables can be configured at build time or runtime:

| Variable | Type | Default | Description |
|----------|------|---------|-------------|
| `GITHUB_CLIENT_ID` | Build-time | `Ov23li9bxz3kKfPOIsGm` | GitHub OAuth app client ID for authentication |
| `POSTHOG_API_KEY` | Build-time | Empty | PostHog analytics API key (disables analytics if empty) |
| `POSTHOG_API_ENDPOINT` | Build-time | Empty | PostHog analytics endpoint (disables analytics if empty) |
| `BACKEND_PORT` | Runtime | `0` (auto-assign) | Backend server port |
| `FRONTEND_PORT` | Runtime | `3000` | Frontend development server port |
| `HOST` | Runtime | `127.0.0.1` | Backend server host |
| `DISABLE_WORKTREE_ORPHAN_CLEANUP` | Runtime | Not set | Disable git worktree cleanup (for debugging) |

**Build-time variables** must be set when running `pnpm run build`. **Runtime variables** are read when the application starts.

#### Custom GitHub OAuth App (Optional)

By default, Automagik Forge uses Namastex Labs's GitHub OAuth app for authentication. To use your own GitHub app for self-hosting or custom branding:

1. Create a GitHub OAuth App at [GitHub Developer Settings](https://github.com/settings/developers)
2. Enable "Device Flow" in the app settings
3. Set scopes to include `user:email,repo`
4. Build with your client ID:
   ```bash
   GITHUB_CLIENT_ID=your_client_id_here pnpm run build
   ```
