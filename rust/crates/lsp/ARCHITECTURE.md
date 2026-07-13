## Module Breakdown

### lib.rs

Public exports: `LspError`, `LspManager`, type definitions. Re-exports
submodule APIs.

### manager.rs

`LspManager` implementation. Coordinates multiple servers:

- `new()`: Validates config, builds extension→server map, detects duplicate
  extensions
- `client_for_path()`: Lazy initialization; creates `LspClient` on first use
  for a server, caches in `clients` map
- `supports_path()`: Fast path check via extension map
- Document operations: Delegate to client (open, change, save, close)
- Queries: Route to client, dedupe results (definitions, references)
- `shutdown()`: Drain all clients, send shutdown+exit to each server

### client.rs

`LspClient` implementation. Manages a single server subprocess:

- `connect()`: Spawn process, initialize, return ready client
- `spawn_reader()`: Background task reads `stdout`, parses JSON-RPC messages,
  delivers responses to pending requests or stores notifications
- `spawn_stderr_drain()`: Background task consumes stderr (servers may log here)
- Request helpers: `notify()`, `request()`, `request_string()` (send message,
  await response via channel)
- Document tracking: `open_documents` map avoids duplicate open/redundant changes
- `initialize()`: Handshake; sends capabilities request, waits for result
- `shutdown()`: Graceful shutdown; send shutdown+exit, kill process if needed

### types.rs

Data structures and helpers:

- **LspServerConfig**: Server command + args + env + language mapping
  - `language_id_for()`: Reverse lookup file extension → language ID
- **FileDiagnostics**: Single file + diagnostics list
- **WorkspaceDiagnostics**: Aggregated diagnostics across workspace
  - `total_diagnostics()`: Count method for metrics
- **SymbolLocation**: File + range; Display impl formats as `path:line:char`
  - Converts LSP 0-indexed positions to 1-indexed for display
- **LspContextEnrichment**: Bundle of diagnostics + definitions + references
  - `render_prompt_section()`: Formats enrichment for LLM consumption (truncates
    to 12 items per category)
- **normalize_extension()**: Normalizes file extension (e.g., `.rs` → `rs`)

### error.rs

`LspError` enum:

- `UnsupportedDocument`: File extension not recognized
- `UnknownServer`: Server config not found
- `Protocol`: JSON-RPC framing or semantic violation
- `Timeout`: Request pending >30s
- `IO`: OS-level I/O error

## Data Flow

### Initialization

1. User creates `LspManager::new(configs)`
   - Validates each config, builds extension→server map
   - Rejects duplicate extensions
2. Manager ready; no servers spawned yet

### First Document Open

1. `open_document(path, text)` called
2. `client_for_path(path)` looks up extension → server name
3. Check `clients` cache; miss
4. Spawn new `LspClient::connect()`:
   - Spawn server process
   - Initialize (handshake)
   - Spawn reader task
   - Return initialized client
5. Cache in `clients` map
6. Send didOpen notification to client
7. Server publishes diagnostics (async notification)

### Symbol Query (e.g., go_to_definition)

1. `go_to_definition(path, position)` called
2. Ensure document open (if not already)
3. Send JSON-RPC request: `textDocument/definition`
4. Reader task parses response, resolves promise
5. Dedupe locations (by path + line + char)
6. Return to caller

### Diagnostics Aggregation

1. Reader task subscribes to `textDocument/publishDiagnostics` (automatic)
2. Each diagnostic stored in client's `diagnostics` map by URI
3. `collect_workspace_diagnostics()` iterates all clients, aggregates
4. Convert URI → path, filter empty, sort by path
5. Return `WorkspaceDiagnostics`

### Shutdown

1. `shutdown()` called
2. Collect all active clients (with Arc refs)
3. Clear clients map
4. Iterate clients, send LSP shutdown + exit
5. Wait for servers to exit gracefully

## Design Decisions

1. **Lazy Initialization**: Servers only spawned when a file of their language
   is opened. Reduces startup latency and resource usage for single-language
   workspaces.

2. **Async Client Channels**: Requests paired with responses via `tokio::oneshot`.
   Allows concurrent requests to same server without blocking.

3. **Deduplication at Manager Layer**: Both symbol queries and diagnostics
   dedupe at `LspManager`, not client. Simplifies client and centralizes logic.

4. **Immutable Config**: `LspServerConfig` cloned, never mutated. No runtime
   reconfiguration (not needed for MVP).

5. **Arc<LspClient> Sharing**: Multiple concurrent calls can reference same
   client safely via Arc. Mutex guards writer and child process.

6. **Notification vs. Request**: Diagnostic updates are notifications (no
   request ID, no response expected). Manager polls client snapshot on demand.

7. **Document Versioning**: LSP requires document version. Client tracks version
   per open document (incremented on each change). Not exposed to public API.

## Known Limitations

- **No Server Crash Recovery**: If server crashes, no automatic restart.
  Manager will return error on next request.
- **No Hot Reconfiguration**: Adding new language after manager creation
  requires creating new manager.
- **Single Reader Thread per Client**: Does not scale to thousands of
  concurrent requests (not a use case for code intelligence).
- **No Workspace Symbols**: Does not implement `workspace/symbol` query.
- **No Hover**: Hover over position not implemented (easy to add).
- **Timeout Hardcoded**: 30-second timeout not configurable.
