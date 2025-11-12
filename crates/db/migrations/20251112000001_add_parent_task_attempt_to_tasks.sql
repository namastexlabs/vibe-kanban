-- Add parent_task_attempt column to track task hierarchy
-- Maps tasks to their parent task attempts (e.g., subtasks under a Genie agent attempt)
ALTER TABLE tasks ADD COLUMN parent_task_attempt BLOB;
