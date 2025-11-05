-- Add token usage tracking columns to task_attempts
-- These fields capture LLM API usage metrics for cost tracking and analytics

ALTER TABLE task_attempts ADD COLUMN input_tokens INTEGER;
ALTER TABLE task_attempts ADD COLUMN output_tokens INTEGER;
ALTER TABLE task_attempts ADD COLUMN cache_creation_tokens INTEGER;
ALTER TABLE task_attempts ADD COLUMN cache_read_tokens INTEGER;
