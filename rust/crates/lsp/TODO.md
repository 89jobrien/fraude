# Near-Term Improvements

1. **Server Crash Recovery**
   Detect server exit (poll `child.try_wait()`). Restart on next request.
   Track restart count to avoid infinite retry loops.

2. **Incremental Document Sync**
   Implement `DidChangeTextDocumentParams` with range-based changes instead of
   full text. Reduces bandwidth for large file edits.

3. **Hover & Workspace Symbols**
   Add `hover()` and `workspace_symbols()` methods. Useful for context
   enrichment in prompts.

4. **Configurable Timeout**
   Add `timeout` field to `LspServerConfig`. Use per-request (some servers
   are slow).

5. **Unit Tests for Manager**
   Add tests for extension collision detection, language_id_for(), and
   client caching. Current tests mock server; need mock manager tests.
