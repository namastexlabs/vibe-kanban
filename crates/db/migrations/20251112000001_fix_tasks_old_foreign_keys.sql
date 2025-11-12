-- Fix foreign keys still referencing tasks_old after 20251112000000 migration
--
-- The previous migration (20251112000000) rebuilt the tasks table but left
-- foreign key constraints in other tables pointing to the non-existent tasks_old table.
-- This migration fixes those foreign keys by rebuilding the affected tables.

PRAGMA foreign_keys = OFF;

-- Drop trigger temporarily to avoid conflicts
DROP TRIGGER IF EXISTS omni_execution_completed;

-- Fix task_attempts foreign key
CREATE TABLE task_attempts_new AS SELECT * FROM task_attempts;
DROP TABLE task_attempts;
CREATE TABLE task_attempts (
    id            BLOB PRIMARY KEY,
    task_id       BLOB NOT NULL,
    executor      TEXT,
    created_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at    TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    target_branch TEXT NOT NULL DEFAULT 'main',
    worktree_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    setup_completed_at DATETIME,
    container_ref TEXT,
    branch TEXT NOT NULL DEFAULT 'main',
    input_tokens INTEGER,
    output_tokens INTEGER,
    cache_creation_tokens INTEGER,
    cache_read_tokens INTEGER,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
);
INSERT INTO task_attempts SELECT * FROM task_attempts_new;
DROP TABLE task_attempts_new;

-- Fix task_images foreign key
CREATE TABLE task_images_new AS SELECT * FROM task_images;
DROP TABLE task_images;
CREATE TABLE task_images (
    id                    BLOB PRIMARY KEY,
    task_id               BLOB NOT NULL,
    image_id              BLOB NOT NULL,
    created_at            TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (image_id) REFERENCES images(id) ON DELETE CASCADE,
    UNIQUE(task_id, image_id)
);
INSERT INTO task_images SELECT * FROM task_images_new;
DROP TABLE task_images_new;

-- Fix forge_omni_notifications foreign key
CREATE TABLE forge_omni_notifications_new AS SELECT * FROM forge_omni_notifications;
DROP TABLE forge_omni_notifications;
CREATE TABLE forge_omni_notifications (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    notification_type TEXT NOT NULL,
    recipient TEXT NOT NULL,
    message TEXT NOT NULL,
    sent_at DATETIME,
    status TEXT DEFAULT 'pending',
    error_message TEXT,
    metadata TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);
INSERT INTO forge_omni_notifications SELECT * FROM forge_omni_notifications_new;
DROP TABLE forge_omni_notifications_new;

-- Fix forge_agents foreign key
CREATE TABLE forge_agents_new AS SELECT * FROM forge_agents;
DROP TABLE forge_agents;
CREATE TABLE forge_agents (
    id BLOB PRIMARY KEY NOT NULL,
    project_id BLOB NOT NULL,
    agent_type TEXT NOT NULL,
    task_id BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    UNIQUE(project_id, agent_type)
);
INSERT INTO forge_agents SELECT * FROM forge_agents_new;
DROP TABLE forge_agents_new;

PRAGMA foreign_keys = ON;

-- Trigger will be recreated by forge services after migrations apply
