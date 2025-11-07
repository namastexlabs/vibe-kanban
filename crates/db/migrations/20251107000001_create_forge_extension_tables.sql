-- Forge Extension Auxiliary Tables
-- Task 2: Backend feature extraction migration
--
-- Creates auxiliary tables for forge-specific features without modifying upstream schema
-- These tables use foreign keys to reference upstream tables but remain completely separate
--
-- HISTORICAL NOTE: Originally created as 20250924090001_auxiliary_tables.sql in forge-app/migrations/
-- Deleted in commit 6c094a2e8 (2025-10-08) during upstream reintegration
-- Restored 2025-11-07 to upstream/crates/db/migrations/ to fix "no such table: forge_global_settings"

-- Extensions for individual tasks
CREATE TABLE IF NOT EXISTS forge_task_extensions (
    task_id TEXT PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    omni_settings TEXT, -- JSON for Omni notification settings
    genie_metadata TEXT, -- JSON for future Genie integration
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Global forge settings (singleton table)
CREATE TABLE IF NOT EXISTS forge_global_settings (
    id INTEGER PRIMARY KEY CHECK (id = 1),
    forge_config TEXT NOT NULL DEFAULT '{}', -- JSON for global forge settings
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Initialize global settings row
INSERT OR IGNORE INTO forge_global_settings (id, forge_config) VALUES (1, '{}');

-- Project-level settings and configuration
CREATE TABLE IF NOT EXISTS forge_project_settings (
    project_id TEXT PRIMARY KEY REFERENCES projects(id) ON DELETE CASCADE,
    custom_executors TEXT, -- JSON for custom executor configurations
    forge_config TEXT, -- JSON for forge-specific project settings
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Omni notification history and tracking
CREATE TABLE IF NOT EXISTS forge_omni_notifications (
    id TEXT PRIMARY KEY,
    task_id TEXT REFERENCES tasks(id) ON DELETE SET NULL,
    notification_type TEXT NOT NULL,
    recipient TEXT NOT NULL,
    message TEXT NOT NULL,
    sent_at DATETIME,
    status TEXT DEFAULT 'pending', -- pending, sent, failed
    error_message TEXT,
    metadata TEXT, -- JSON metadata for notification context
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Convenience view for enhanced task access with forge extensions
CREATE VIEW IF NOT EXISTS enhanced_tasks AS
SELECT
    t.*,
    fte.omni_settings,
    fte.genie_metadata
FROM tasks t
LEFT JOIN forge_task_extensions fte ON t.id = fte.task_id;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_forge_omni_notifications_task_id
ON forge_omni_notifications(task_id);

CREATE INDEX IF NOT EXISTS idx_forge_omni_notifications_status
ON forge_omni_notifications(status);

CREATE INDEX IF NOT EXISTS idx_forge_task_extensions_task_id
ON forge_task_extensions(task_id);

CREATE INDEX IF NOT EXISTS idx_forge_project_settings_project_id
ON forge_project_settings(project_id);

-- Timestamp triggers for automatic updated_at maintenance
CREATE TRIGGER IF NOT EXISTS update_forge_task_extensions_timestamp
AFTER UPDATE ON forge_task_extensions
BEGIN
    UPDATE forge_task_extensions
    SET updated_at = CURRENT_TIMESTAMP
    WHERE task_id = NEW.task_id;
END;

CREATE TRIGGER IF NOT EXISTS update_forge_global_settings_timestamp
AFTER UPDATE ON forge_global_settings
BEGIN
    UPDATE forge_global_settings
    SET updated_at = CURRENT_TIMESTAMP
    WHERE id = NEW.id;
END;

CREATE TRIGGER IF NOT EXISTS update_forge_project_settings_timestamp
AFTER UPDATE ON forge_project_settings
BEGIN
    UPDATE forge_project_settings
    SET updated_at = CURRENT_TIMESTAMP
    WHERE project_id = NEW.project_id;
END;
