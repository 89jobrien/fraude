## Module Breakdown

### lib.rs

Public exports and submodule re-exports. Central coordinator is
`ConversationRuntime`.

### conversation.rs

`ConversationRuntime` implementation. Entry point for multi-turn loops:

- `new()`: Initialize with session, config, tool executor, permission prompter
- `run_turn()`: Main event loop:
  1. Send user message to LLM
  2. Stream response text via `AssistantEvent::Text`
  3. Extract tool calls from response
  4. For each tool: check permissions, execute, collect result
  5. Append tool results to session
  6. If tool results, loop (new turn with assistant)
  7. When assistant stops, return `TurnSummary` wrapped in `Crux<TurnSummary>`
- Tracks turn metadata: tokens, stop reason, tool execution times

### session.rs

Session and message types:

- `Session`: Immutable snapshot of conversation (vec of messages)
- `ConversationMessage`: Role + content blocks
- `ContentBlock`: Union of Text, ToolUse, ToolResult
- `MessageRole`: User, Assistant, Tool
- Message construction: `ConversationMessage::user()`, `::assistant()`,
  `::tool_result()`

### file_ops.rs

File operation implementations:

- `read_file()`: Open, read all, return size + charset + content
- `write_file()`: Validate path; create/truncate; persist
- `edit_file()`: Parse hunks; apply patches; generate unified diff
- `glob_search()`: Scan filesystem; filter by pattern; return sorted paths
- `grep_search()`: Regex search; return lines + line numbers + context (before/after)

All operations validate paths:

- Reject absolute paths (except workspace root marker)
- Reject `../` escaping
- Reject hidden files (leading `.`)
- Check config allowlist (if set)
- Check config blocklist (if set)

### bash.rs

Bash command execution:

- `execute_bash()`: Spawn tokio subprocess
- Timeout handling: tokio::time::timeout(duration)
- Combine stdout/stderr
- Return exit code + combined output
- Kill process on timeout

Environment setup:

- Inherit parent env
- Can inject vars from config
- Set PWD to workspace root

### config.rs

Configuration loading and validation:

- `RuntimeConfig`: Struct holding all settings
- `ConfigLoader`: Builder for loading from multiple sources
  1. Defaults (hardcoded)
  2. Environment variables (FRAUDE\_\* prefix)
  3. Dotenv file (.env, .env.local, .env.{profile})
  4. YAML config (~/.fraude/config.yaml, {workspace}/.fraude.yaml)
  5. Merge with precedence (env > local yaml > dotenv > defaults)
- Validation: Check required fields, enum values, path existence
- Error collection: All validation errors returned at once

Config fields:

- Model, provider URL, API key
- Default tool permissions
- File operation allowlists/blocklists
- LSP server configs
- MCP server configs
- Session storage path
- Workspace root

### compact.rs

Session compaction logic:

- `estimate_session_tokens()`: Sum tokens per message + tool results using
  model's pricing
- `should_compact()`: Check if tokens exceed threshold (default 80% of limit)
- `compact_session()`:
  1. Keep system message and last N user/assistant exchanges
  2. Summarize old messages (external LLM call or rule-based)
  3. Return new session with summary block + metadata
- `CompactionResult`: Old message count, summary block, new token estimate

Used to prevent context overflow in long conversations.

### prompt.rs

System prompt injection:

- `load_system_prompt()`: Load base template (hardcoded or from file)
- Inject project context:
  - Workspace root, language(s), build system
  - Key files (README, package.json, Cargo.toml)
  - Codebase statistics (lines of code, file count)
- `SystemPromptBuilder`: Fluent API to construct prompt piecemeal
- `SYSTEM_PROMPT_DYNAMIC_BOUNDARY`: Marker for where dynamic content ends

### permissions.rs

Permission checking and prompting:

- `PermissionMode`: Deny, Allow, Prompt, Policy
- `PermissionPolicy`: Struct holding per-tool permissions
- `PermissionPrompter`: Trait for interactive prompts (blocks async)
- `PermissionRequest`: Tool name, required mode, rationale
- `PermissionOutcome`: User decision (allow, deny, allow-all-session)

Flow:

1. Tool executor checks permission for tool
2. If Prompt: call prompter.prompt(request).await
3. Prompter shows user message, waits for response
4. Return Allow or Deny

### bash.rs (continued)

Dangerous command detection:

- Patterns for `rm -rf` (prompt for confirmation)
- Patterns for `sudo` (deny by default)
- Patterns for system paths (deny by default)
- Bypass via explicit allow in config

### oauth.rs

OAuth 2.0 PKCE flow:

- `OAuthAuthorizationRequest`: Build authorization URL
- `PkceCodePair`: Generate + store code verifier + challenge
- `OAuthTokenExchangeRequest`: Exchange code for tokens
- `OAuthTokenSet`: Access token + refresh token + expiry
- `save_oauth_credentials()`: Persist to ~/.fraude/oauth/{provider}.json
- `load_oauth_credentials()`: Restore from disk
- `generate_pkce_pair()`: S256 challenge method
- `loopback_redirect_uri()`: Listen on 127.0.0.1:random_port

Used for:

- MCP OAuth servers
- Remote session authentication
- Upstream proxy auth

### mcp_stdio.rs

MCP client for stdio-based servers:

- `spawn_mcp_stdio_process()`: Start server subprocess
- `McpInitializeParams`: Capabilities, client info
- `McpListToolsResult`: Available tools
- `McpToolCallParams`: Tool invocation
- `McpToolCallResult`: Tool result (success or error)
- Message framing: JSON-RPC over lines
- Async request/response matching via channels

### mcp_client.rs

MCP transport abstraction:

- `McpClientTransport`: Trait for different transports
- `McpStdioTransport`: Stdio subprocess
- `McpRemoteTransport`: HTTP/WebSocket server
- `McpManagedProxyTransport`: Managed proxy (e.g., Claude Code proxy)
- Bootstrap protocol: Load transport config, connect, initialize

### lsp/\* (via lsp crate)

Language server management (external crate):

- Spawns/manages LSP servers
- Document lifecycle (open, change, save, close)
- Diagnostics, definitions, references
- Used for context enrichment

### mcp/\* (MCP coordination)

MCP server manager:

- Discover servers from config
- Initialize on startup
- Route tool calls to servers
- Handle MCP errors

### remote.rs

Remote session support:

- `RemoteSessionContext`: Session stored on remote server
- `UpstreamProxyBootstrap`: Set up upstream proxy auth
- `inherited_upstream_proxy_env()`: Capture proxy env from parent
- Token storage: ~/.fraude/session-token

### hooks.rs

Hook execution (delegates to plugins):

- `HookRunner`: Execute PreToolUse, PostToolUse hooks
- `HookEvent`: Hook type + context
- `HookRunResult`: Captured stdout/stderr/exit code

### json.rs

JSON utilities:

- Schema validation helpers
- JSON Path queries
- Serialization/deserialization wrappers

### usage.rs

Token counting and cost estimation:

- `TokenUsage`: Prompt tokens, completion tokens
- `ModelPricing`: Per-model pricing ($/1k tokens)
- `pricing_for_model()`: Look up rates
- `format_usd()`: Format cost as USD
- `UsageTracker`: Accumulate usage across turns

### bootstrap.rs

Agent initialization:

- `BootstrapPhase`: Setup steps (load config, init LSP, init MCP)
- `BootstrapPlan`: Ordered phases
- Dependency resolution: LSP depends on workspace root, etc.

## Data Flow

### Turn Execution

```
User Message
  ↓
run_turn(message, model)
  ↓
Append UserMessage to session
  ↓
LLM API Call (stream response)
  ↓
Collect AssistantEvent::Text
  ↓
Extract tool calls → loop per tool:
    ↓
  Tool Call → Check Permission
    ↓
  Permission: Prompt? → Call prompter.prompt()
    ↓
  Permission: Deny → return error
    ↓
  Permission: Allow → Execute tool
    ↓
  Tool result → AssistantEvent::ToolResult
    ↓
Append to session
  ↓
Continue LLM (send tool results)
  ↓
Loop until stop reason = EndTurn
  ↓
Return TurnSummary wrapped in Crux<TurnSummary>
```

### Config Loading

```
Defaults (hardcoded)
  ↓ Merge
Env vars (FRAUDE_*)
  ↓ Merge
Dotenv file (.env)
  ↓ Merge
YAML config (local, home)
  ↓ Merge
Final RuntimeConfig
  ↓
Validate (required fields, enums)
  ↓
RuntimeConfig or ConfigError
```

### Session Compaction

```
estimate_session_tokens(session, model)
  ↓
Sum tokens per message/block
  ↓
should_compact(tokens, limit)?
  ↓
Yes → compact_session():
  ↓
Keep last N exchanges
  ↓
Summarize old messages
  ↓
New session with summary
  ↓
CompactionResult (old_count, summary, new_tokens)
```

## Design Decisions

1. **Async I/O**: All tool execution and LLM calls async via tokio. Enables
   concurrent tool execution and responsive streaming.

2. **Permission Separation**: Permission checking split from execution. Allows
   dry-run permission verification and testing without side effects.

3. **Configuration Precedence**: Env vars > local yaml > dotenv > defaults.
   Enables both local dev config (dotenv) and production overrides (env).

4. **Immutable Session**: Session snapshots, not mutable references. Enables
   safe concurrent access and rollback on error.

5. **Crux Wrapping**: Turn results wrapped in crux-types for trace
   compatibility. Allows downstream systems (devloop, etc.) to consume traces.

6. **Path Validation Upfront**: File operations validate paths before
   execution. Prevents accidental writes to system paths or secrets dirs.

7. **Tool Executor Trait**: Abstraction for tool execution. Enables mock
   implementations for testing; allows swapping executors (e.g., remote vs
   local).

8. **Permission Prompter Trait**: Abstraction for prompts. Enables different
   UI backends (TUI, HTTP, log-and-allow).

9. **Lazy LSP Initialization**: LSP servers only spawned when files of their
   language are accessed. Reduces startup time.

10. **MCP as Optional**: MCP support optional; degrades gracefully if no MCP
    servers configured.

## Known Limitations

- **No Distributed Sessions**: Sessions stored locally only. Requires process
  restart to switch to remote session.
- **No Session Branching**: Cannot fork conversation into alternatives.
- **No Tool Caching**: Same tool call always re-executed; no memoization.
- **No Cross-Turn Context**: Previous turn context lost after compaction.
  Only summary preserved.
- **Sequential Tool Execution**: Tools executed in order, one at a time. No
  parallel execution.
- **Permission Prompt Blocks**: Interactive prompts block async execution.
  Cannot prompt multiple tools concurrently.
