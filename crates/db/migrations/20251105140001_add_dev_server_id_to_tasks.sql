-- Link tasks to dev servers for tracking dev server usage in analytics
-- Note: No foreign key constraint since dev_servers table may not exist yet

ALTER TABLE tasks ADD COLUMN dev_server_id BLOB;

-- Index for faster lookups
CREATE INDEX idx_tasks_dev_server_id ON tasks(dev_server_id);
