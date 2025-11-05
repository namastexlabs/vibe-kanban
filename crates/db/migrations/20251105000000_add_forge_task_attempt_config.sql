-- Add forge_task_attempt_config table for Master Genie worktree configuration
CREATE TABLE IF NOT EXISTS forge_task_attempt_config (
    task_attempt_id TEXT PRIMARY KEY NOT NULL,
    use_worktree BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (task_attempt_id) REFERENCES forge_task_attempts(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_forge_task_attempt_config_task_attempt_id
ON forge_task_attempt_config(task_attempt_id);
