# Near-Term Improvements

1. **Tool Execution Parallelization**
   Collect all tool calls from assistant response before executing. Execute
   in parallel via tokio::join_all instead of sequentially. Collect results,
   send all at once. Reduces latency for multi-tool turns.

2. **Interactive Permission Policy**
   Add session-level permission cache: after user allows a tool once, cache
   decision for remainder of session. Add option to "allow all for this
   session" to reduce prompt fatigue.

3. **Configurable Token Limits Per Model**
   Currently hardcoded context window sizes. Move to config/database. Add
   support for dynamic limits (query model endpoint for current limits on
   account).

4. **File Operation Diff Context**
   `edit_file()` currently returns minimal diff. Add configurable context
   lines (before/after per hunk). Include full old content for verification.

5. **Bash Command History**
   Track executed bash commands in session. Include in compaction so LLM can
   reference what's been run. Add shell aliases auto-discovery (read
   ~/.bashrc on init).
