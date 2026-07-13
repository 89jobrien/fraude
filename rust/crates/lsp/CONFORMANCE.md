## Standards & Protocols

### Language Server Protocol (LSP) 3.17

Implements core LSP 3.17 features (per lsp-types crate):

- **Initialization**: `initialize` request with capabilities negotiation
- **Document Sync**: `textDocument/didOpen`, `textDocument/didChange`,
  `textDocument/didSave`, `textDocument/didClose` notifications
- **Diagnostics**: Receives `textDocument/publishDiagnostics` notifications
- **Code Navigation**: `textDocument/definition` and `textDocument/references`
  requests
- **JSON-RPC 2.0**: Request/response pairing with numeric IDs, notifications
  without IDs

See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/

### File URI Scheme

Converts filesystem paths to `file://` URIs per RFC 8089:

```
/path/to/file.rs  →  file:///path/to/file.rs
C:\path\file.rs   →  file:///C:/path/file.rs
```

Uses `url` crate for proper percent-encoding and round-trip safety.

### JSON-RPC 2.0

Message framing with Content-Length headers:

```
Content-Length: <bytes>\r\n\r\n
<JSON-serialized payload>
```

- All messages must include `Content-Length` header
- Responses include `id` matching request; notifications omit `id`
- Errors encoded as `{"jsonrpc":"2.0","id":N,"error":{"code":-32600,"message":"..."}}`

## Known Deviations

1. **Partial LSP Feature Set**: Does not implement completion, hover, rename,
   folding ranges, or semantic tokens. Only definition, references, and
   diagnostics.

2. **No Incremental Document Sync**: Always sends full document text on change.
   LSP supports sending diffs; not implemented.

3. **No Initialization Options**: Does not pass
   `initializationOptions` to server (reserved in config but unused).

4. **No Custom Capabilities**: Does not request server-specific capabilities
   (e.g., `textDocument.definition.linkSupport`). Assumes full support.

5. **No Multi-Root Workspace**: Each `LspClient` has single `workspace_root`.
   Does not support LSP `workspace/didChangeConfiguration`.

6. **Timeout Hardcoded**: 30-second request timeout not configurable via LSP
   config. Should be per-server option.

7. **No Trace/Debug Mode**: Does not implement LSP `$/trace` notifications for
   server debugging.

## No External Standard

LSP is the primary standard. This crate is a partial implementation focused on
diagnostics and navigation; not a full LSP client.
