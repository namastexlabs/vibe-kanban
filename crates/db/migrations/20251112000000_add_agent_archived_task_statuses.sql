-- Add 'agent' and 'archived' status values to task status constraint
-- This migration adds support for new task statuses while maintaining backward compatibility

ALTER TABLE tasks RENAME TO tasks_old;

CREATE TABLE tasks (
    id          BLOB PRIMARY KEY,
    project_id  BLOB NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    status      TEXT NOT NULL DEFAULT 'todo'
                   CHECK (status IN ('todo','inprogress','done','cancelled','inreview','agent','archived')),
    created_at  TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    updated_at  TEXT NOT NULL DEFAULT (datetime('now', 'subsec')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Copy data from old table
INSERT INTO tasks (id, project_id, title, description, status, created_at, updated_at)
SELECT id, project_id, title, description, status, created_at, updated_at FROM tasks_old;

-- Drop old table
DROP TABLE tasks_old;
