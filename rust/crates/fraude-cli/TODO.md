# Near-Term Improvements

1. **Async Session Compaction**
   Compaction currently blocks the REPL. Spawn background task or add a
   `/compact --async` flag to queue compaction and return immediately. Track
   compaction state in LiveCli.

2. **Pluggable Slash Commands**
   Extract slash command dispatch into a registry trait. Allow plugins to
   register custom commands via the plugin manager. Requires changes to
   `commands` crate to support dynamic registration.

3. **Dashboard Session Integration**
   Wire TUI dashboard to use the same session persistence as REPL. Add
   `/session list` and `/session switch` support to dashboard. Requires
   ratatui event loop refactor to integrate with REPL command handling.

4. **OAuth Callback Parser Hardening**
   Replace single-line TCP read with proper HTTP request parsing. Handle
   malformed callbacks gracefully. Log callback details for debugging.

5. **Tool Schema Validation**
   Add optional JSON Schema validation for tool inputs before passing to
   runtime. Catch schema mismatches early and provide better error messages.
