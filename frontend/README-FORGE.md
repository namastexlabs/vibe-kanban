# Upstream Frontend (Read-Only Reference)

**⚠️ DO NOT EDIT THIS DIRECTORY ⚠️**

This directory contains the **unmodified** vibe-kanban frontend for reference purposes only.

## Purpose

- **Reference**: When our frontend has bugs, compare with upstream implementation
- **Cherry-picking**: Identify useful upstream features to port to our frontend
- **Learning**: Understand how they solved problems

## Automagik Forge Frontend Location

**Edit here**: `/frontend/` (root of repo)

The `/frontend/` directory is our **fully owned** frontend that contains:
- Upstream base code (from this directory)
- All Automagik Forge customizations layered on top
- Simplified build system (no overlay complexity)

## Backend Sync vs Frontend Ownership

| Component | Location | Sync Strategy |
|-----------|----------|---------------|
| **Backend** | `upstream/crates/` | Sync 4x/month from vibe-kanban |
| **Frontend** | `/frontend/` | Owned by us, cherry-pick features |
| **Frontend Reference** | `upstream/frontend/` | Read-only, never edit |

## Migration History

**Date**: 2025-10-28  
**Change**: Frontend unified from 3-layer architecture to single owned directory  
**Reason**: Eliminate complexity, improve dev speed, maintain backend sync

**Before**:
```
frontend/                    # Bootstrap
forge-overrides/frontend/    # Customizations
upstream/frontend/           # Base (via complex resolver)
→ 3 layers = slow, confusing
```

**After**:
```
frontend/                    # Our complete frontend (edit here!)
upstream/frontend/           # Reference only (never edit)
→ 1 layer = fast, simple
```

## Cherry-Picking Upstream Features

```bash
# Add upstream remote if not already added
git remote add vibe-upstream https://github.com/vibekanban/vibe-kanban.git
git fetch vibe-upstream

# Cherry-pick a specific commit's frontend changes
git cherry-pick <commit-hash> -- frontend/

# Or manually review and port
git show <commit-hash>:frontend/src/SomeComponent.tsx
```

## Questions?

See `.genie/architecture/frontend-backend-split.md` for full context.
