# Token Tracking Implementation Handoff

**Status:** Database ready, executor integration incomplete
**Tracking:** Issues #79, #80, #81 in automagik-forge repo

---

## What's Complete ✅

### Database Schema
- ✅ Migration `20251105140000_add_token_usage_to_task_attempts.sql`
  - `input_tokens INTEGER` - Total input tokens used
  - `output_tokens INTEGER` - Total output tokens generated
  - `cache_creation_tokens INTEGER` - Cache creation tokens (Claude)
  - `cache_read_tokens INTEGER` - Cache read tokens (Claude)

- ✅ Migration `20251105140001_add_dev_server_id_to_tasks.sql`
  - `dev_server_id BLOB` - Links tasks to dev servers

### Rust Models
- ✅ `TaskAttempt` struct updated with token fields (all queries updated)
- ✅ `Task` struct updated with `dev_server_id` (all queries updated)
- ✅ Helper function: `TaskAttempt::update_token_usage()` at `crates/db/src/models/task_attempt.rs:227`

### Frontend
- ✅ TypeScript types regenerated with new fields
- ✅ Tracking updated to fetch real attempt data
- ✅ Uses real `dev_server_id` instead of fallbacks

---

## What's Missing ❌

### Token Capture & Persistence

**Problem:** Token usage data streams through executor logs but is never saved to database.

**Current Flow:**
```
Executor → SSE stream → Log processor → UI updates → [DISCARDED]
```

**Needed Flow:**
```
Executor → SSE stream → Log processor → Accumulate tokens → On completion → Save to DB
```

**Implementation Points:**

### 1. Claude Code Executor (Issue #79)

**File:** `crates/executors/src/executors/claude.rs`

Add token accumulator to `ClaudeLogProcessor`:
```rust
struct ClaudeLogProcessor {
    // ... existing fields
    accumulated_usage: ClaudeUsage,  // NEW: accumulate across stream
}
```

Accumulate tokens from `MessageDelta { usage }` events:
```rust
// In process_logs() when handling MessageDelta
if let Some(usage) = message_delta.usage {
    self.accumulated_usage.input_tokens = Some(
        self.accumulated_usage.input_tokens.unwrap_or(0) + usage.input_tokens.unwrap_or(0)
    );
    self.accumulated_usage.output_tokens = Some(
        self.accumulated_usage.output_tokens.unwrap_or(0) + usage.output_tokens.unwrap_or(0)
    );
    // ... same for cache_creation_tokens and cache_read_tokens
}
```

Expose accumulated tokens via method:
```rust
impl ClaudeLogProcessor {
    pub fn get_accumulated_usage(&self) -> &ClaudeUsage {
        &self.accumulated_usage
    }
}
```

**File:** `crates/local-deployment/src/container.rs:388`

After `update_executor_session_summary()`, save tokens:
```rust
// Around line 388, after update_executor_session_summary()
if let Ok(ctx) = ExecutionProcess::load_context(&db.pool, exec_id).await {
    // Check if this is a Claude Code execution
    if let Ok(executor_action) = ctx.execution_process.executor_action()
        && matches!(
            executor_action.typ,
            ExecutorActionType::CodingAgentInitialRequest(_) | ExecutorActionType::CodingAgentFollowUpRequest(_)
        )
    {
        // TODO: Retrieve accumulated tokens from log processor/msg_store
        // For now this is pseudocode - need to wire up access to ClaudeLogProcessor
        // let usage = log_processor.get_accumulated_usage();

        // Convert u64 to i32 (or consider changing schema to BIGINT)
        // let input = usage.input_tokens.map(|t| t as i32);
        // let output = usage.output_tokens.map(|t| t as i32);
        // let cache_creation = usage.cache_creation_input_tokens.map(|t| t as i32);
        // let cache_read = usage.cache_read_input_tokens.map(|t| t as i32);

        // if let Err(e) = TaskAttempt::update_token_usage(
        //     &db.pool,
        //     ctx.task_attempt.id,
        //     input,
        //     output,
        //     cache_creation,
        //     cache_read,
        // ).await {
        //     tracing::warn!("Failed to save token usage: {}", e);
        // }
    }
}
```

**Challenge:** Need to pass `ClaudeLogProcessor` or accumulated usage through to the completion handler. Options:
1. Store accumulated usage in `MsgStore`
2. Pass processor reference through ExecutionContext
3. Add token_usage field to ExecutionProcess model

### 2. Codex Executor (Issue #80)

**Blocked:** Need to investigate `TokenUsageInfo` structure from external `codex_protocol` crate.

**Tasks:**
1. Clone https://github.com/openai/codex.git at rev `488ec061bf4d36916b8f477c700ea4fde4162a7a`
2. Inspect `TokenUsageInfo` in `codex-protocol` package
3. Map fields to our schema
4. Implement similar accumulator as Claude

### 3. Other Executors (Issue #81)

**Research needed:** Check if Gemini, AMP, Cursor, Copilot, Opencode, Qwen expose token data.

---

## Example Usage

Once token data is available in completion handler:

```rust
use db::models::task_attempt::TaskAttempt;

// After executor completes
let input_tokens = Some(1500);
let output_tokens = Some(800);
let cache_creation_tokens = Some(200);
let cache_read_tokens = Some(500);

TaskAttempt::update_token_usage(
    &pool,
    attempt_id,
    input_tokens,
    output_tokens,
    cache_creation_tokens,
    cache_read_tokens,
).await?;
```

---

## Testing

**Manual Test Flow:**
1. Create task with Claude Code executor
2. Let it complete successfully
3. Query database: `SELECT input_tokens, output_tokens FROM task_attempts WHERE id = ?`
4. Verify tokens are populated (not NULL)
5. Check frontend displays token usage in analytics

**Edge Cases:**
- Empty/zero tokens should be represented as `Some(0)`, not `None`
- Failed executions should still save accumulated tokens (partial usage)
- u64 overflow when converting to i32 (unlikely but handle gracefully)

---

## References

- **Database helper:** `crates/db/src/models/task_attempt.rs:227`
- **Completion handler:** `crates/local-deployment/src/container.rs:388`
- **Claude usage struct:** `crates/executors/src/executors/claude.rs:1554`
- **Issues:** #79 (Claude), #80 (Codex), #81 (Others)
