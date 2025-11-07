-- Add forge_task_attempt_config table for Master Genie worktree configuration
-- Stores forge-specific configuration for task attempts
-- Default to use_worktree=TRUE for backward compatibility (existing behavior)
-- Master Genie tasks will explicitly set use_worktree=FALSE to run on current branch

CREATE TABLE IF NOT EXISTS forge_task_attempt_config (
    task_attempt_id BLOB PRIMARY KEY NOT NULL,
    use_worktree BOOLEAN NOT NULL DEFAULT 1, -- TRUE = create worktree, FALSE = use main repo
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),

    FOREIGN KEY (task_attempt_id) REFERENCES task_attempts(id) ON DELETE CASCADE
);

-- Index for fast lookups
CREATE INDEX IF NOT EXISTS idx_forge_task_attempt_config_task_attempt_id
ON forge_task_attempt_config(task_attempt_id);
