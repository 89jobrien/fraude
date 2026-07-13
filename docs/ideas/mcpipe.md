# mcpipe — MCP connection manager

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude has MCP config parsing and stdio bootstrap (`runtime/src/mcp.rs`,
`runtime/src/mcp_client.rs`, `runtime/src/mcp_stdio.rs`) but no dynamic connection
manager: no health monitoring, no reconnect, no UI layer for browsing available tools, no
multi-server orchestration. `mcpipe` is an MCP proxy and tool pipeline CLI that owns
exactly this layer.

## What it would do

`mcpipe` runs as a sidecar process that manages MCP server connections on behalf of
fraude. Fraude connects to mcpipe via a local socket rather than managing individual MCP
server connections itself. mcpipe handles:

- Starting and restarting MCP stdio servers
- Aggregating tools from multiple MCP servers into a single tool namespace
- Health checking and reconnecting dropped servers
- Proxying tool calls and returning results

## Integration sketch

```
.fraude/settings.json:
{
  "mcpServers": {
    "filesystem": { "command": "mcp-server-filesystem", "args": [...] },
    "github":     { "command": "mcp-server-github",     "args": [...] }
  }
}

fraude startup
    │
    ▼
runtime::mcp::start_via_mcpipe(config)
    └─► spawn: mcpipe serve --config .fraude/settings.json --socket /tmp/fraude-mcp.sock
                  │
                  ├─ starts each configured MCP server
                  ├─ aggregates tool namespaces
                  └─ exposes unified JSON-RPC socket

tools::LiveCli  ──► MCPTool::execute(name, input)
                        └─► connect to /tmp/fraude-mcp.sock
                            send: tools/call { name, input }
                            recv: tool result
```

The `/mcp` slash command surface (currently a stub) connects to mcpipe to list servers,
show tool availability, and display connection health.

## Fraude changes required

1. Add `MCPTool` to the tool registry — a generic dispatcher that forwards any
   `mcp__<server>__<tool>` call to the mcpipe socket.
2. Replace the current `mcp_stdio.rs` one-shot bootstrap with a mcpipe sidecar launch at
   session start.
3. Implement `/mcp list`, `/mcp status`, `/mcp restart <server>` handlers using mcpipe's
   control API.
4. Graceful shutdown: send mcpipe a shutdown signal at session end.

## Dependencies

- `mcpipe` binary on PATH
- Unix socket support (already available via `std::os::unix::net`)

## Reference

`~/dev/mcpipe` — source repo.
