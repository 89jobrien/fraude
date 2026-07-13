Spawns and manages LSP servers (rust-analyzer, pylsp, etc.) to provide
diagnostics, symbol navigation, and context enrichment for LLM prompts. Handles
JSON-RPC protocol, document synchronization, and multi-language coordination.

## Features

- **Multi-Server Coordination**: Manages multiple LSP servers based on file
  extension; routes requests to the appropriate server
- **Document Lifecycle**: Open, change, save, close operations with versioning
- **Diagnostics Aggregation**: Collects and dedupes workspace diagnostics by
  file and severity
- **Symbol Navigation**: Go-to-definition and find-references with deduplication
- **Context Enrichment**: Single method to gather diagnostics, definitions, and
  references for a file+position
- **JSON-RPC 2.0**: Implements message framing (Content-Length headers),
  async request/response matching via channel pairs
- **Async I/O**: Tokio-based reader/writer; non-blocking server communication
- **Error Handling**: Detailed error types for protocol, configuration, and
  server lifecycle issues

## Architecture

### LspManager (Public Entry Point)

Facade for all LSP operations. Manages server configuration, extension mapping,
and lazy initialization of LSP clients.

```rust
pub struct LspManager {
    server_configs: BTreeMap<String, LspServerConfig>,
    extension_map: BTreeMap<String, String>,
    clients: Mutex<BTreeMap<String, Arc<LspClient>>>,
}
```

Routes all requests to the appropriate `LspClient` based on file extension.
Dedupes symbol locations across results.

### LspClient (Internal)

Spawns a single LSP server process. Handles:

- Initialization handshake (initialize + initialized)
- Document lifecycle (didOpen, didChange, didSave, didClose)
- Request/response pairing via pending request map
- Diagnostic subscription (textDocument/publishDiagnostics notifications)
- Async reader thread for inbound messages

```rust
pub(crate) struct LspClient {
    config: LspServerConfig,
    writer: Mutex<BufWriter<ChildStdin>>,
    child: Mutex<Child>,
    pending_requests: PendingRequests,
    diagnostics: Arc<Mutex<BTreeMap<String, Vec<Diagnostic>>>>,
    next_request_id: AtomicI64,
}
```

### Types

- **LspServerConfig**: Command, args, env, workspace root, language mappings
- **FileDiagnostics**: URI, path, and diagnostic list for a single file
- **WorkspaceDiagnostics**: Collection of file diagnostics
- **SymbolLocation**: Path, range, and display helpers (1-indexed line/char)
- **LspContextEnrichment**: Bundled enrichment data (diagnostics + definitions +
  references)

## Public API

```rust
pub struct LspManager { /* ... */ }
impl LspManager {
    pub fn new(server_configs: Vec<LspServerConfig>) -> Result<Self, LspError>
    pub fn supports_path(&self, path: &Path) -> bool
    pub async fn open_document(&self, path: &Path, text: &str) -> Result<(), LspError>
    pub async fn sync_document_from_disk(&self, path: &Path) -> Result<(), LspError>
    pub async fn change_document(&self, path: &Path, text: &str) -> Result<(), LspError>
    pub async fn save_document(&self, path: &Path) -> Result<(), LspError>
    pub async fn close_document(&self, path: &Path) -> Result<(), LspError>
    pub async fn go_to_definition(&self, path: &Path, position: Position)
        -> Result<Vec<SymbolLocation>, LspError>
    pub async fn find_references(&self, path: &Path, position: Position,
        include_declaration: bool) -> Result<Vec<SymbolLocation>, LspError>
    pub async fn collect_workspace_diagnostics(&self)
        -> Result<WorkspaceDiagnostics, LspError>
    pub async fn context_enrichment(&self, path: &Path, position: Position)
        -> Result<LspContextEnrichment, LspError>
    pub async fn shutdown(&self) -> Result<(), LspError>
}
```

## Message Format

LSP communication follows JSON-RPC 2.0 with Content-Length headers:

```
Content-Length: 256\r\n\r\n
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{...}}
```

Requests matched to responses via numeric IDs. Notifications (no ID) broadcast
via `diagnostics` snapshot. Server stderr drained asynchronously.

## Error Handling

- **UnsupportedDocument**: File extension not mapped to any server
- **UnknownServer**: Server config missing (configuration error)
- **Protocol**: JSON-RPC framing, serialization, or server protocol violation
- **Timeout**: Request pending for >30s without response
- **IO**: Process spawn, pipe I/O, or filesystem read failures

## Dependencies

- `lsp-types`: LSP specification types (from language-server-protocol)
- `tokio`: Async runtime, child processes, sync primitives
- `serde_json`: JSON serialization/deserialization
- `url`: File URL parsing (file:// URIs for LSP)

## Testing

Unit tests exercise mock LSP server implementation (Python subprocess). Tests
verify:

- Configuration validation (extension collision detection)
- Definition/reference deduplication
- Diagnostic snapshot aggregation
- Document lifecycle state transitions
