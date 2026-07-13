Core multi-turn agent loop runtime. Orchestrates conversation state, tool
execution, LLM provider communication, permissions, session compaction, and
file/shell operations. Wraps turn results in crux-types for agent trace
compatibility.

## Features

- **Multi-Turn Conversation**: Manages message history (user, assistant, tool
  results) with versioning
- **LLM Provider Abstraction**: Delegates to API crate; routes requests to
  OpenAI-compatible or hosted providers
- **Tool Executor**: Coordinates tool invocation (bash, file ops, plugins,
  custom tools) with permission prompting
- **Permission Gating**: Interactive or policy-based access control for
  sensitive operations (execute, write, delete)
- **Bash Execution**: Secure subprocess management with timeout, stdio
  buffering, signal handling
- **File Operations**: read, write, edit, glob_search, grep_search with path
  validation and diff generation
- **Session Persistence**: Serialize/deserialize conversations; support
  encryption-at-rest for remote sessions
- **Token Compaction**: Estimate token usage; compact old messages when
  approaching context limits
- **System Prompt Loading**: Dynamic prompt injection with project context
  (files, language, codebase metadata)
- **LSP Integration**: Async language server manager for diagnostics,
  definitions, references
- **MCP Support**: Stdio-based Model Context Protocol client; multi-server
  coordination
- **OAuth Flow**: PKCE-based authorization; token storage and refresh

## Core Types

### Session

Immutable conversation snapshot:

```rust
pub struct Session {
    pub messages: Vec<ConversationMessage>,
}

pub struct ConversationMessage {
    pub role: MessageRole,
    pub content: Vec<ContentBlock>,
}
```

### ContentBlock

Union of content types:

- `Text(String)`: User/assistant text
- `ToolUse(ToolUseBlock)`: Tool invocation request
- `ToolResult(ToolResultBlock)`: Tool result (success or error)

### AssistantEvent

Streamed event from `run_turn()`:

- `Text(String)`: Partial assistant response text
- `ToolCall(ToolUseBlock)`: Tool invocation
- `ToolResult(ToolResultBlock)`: Tool result
- `Metadata(TurnMetadata)`: Token usage, stop reason, timing
- `Final(TurnSummary)`: Wrapped in `Crux<TurnSummary>` for trace compatibility

### PermissionMode

Access control level for tool:

- `Allow`: No prompt; always execute
- `Deny`: Never execute
- `Prompt`: Interactive prompt (blocks async execution)
- `Policy`: Check policy file; prompt if undecided

## Architecture

### ConversationRuntime

Entry point for agent loop:

```rust
pub struct ConversationRuntime {
    api_client: ApiClient,
    config: RuntimeConfig,
    session: Session,
    lsp_manager: Option<LspManager>,
    mcp_servers: McpServerManager,
    tool_executor: Box<dyn ToolExecutor>,
    permission_prompter: Box<dyn PermissionPrompter>,
}

impl ConversationRuntime {
    pub async fn run_turn(
        &mut self,
        user_message: &str,
        model: &str,
    ) -> Result<impl Stream<Item = AssistantEvent>, RuntimeError>
}
```

### Tool Execution

`run_turn()` delegates tool calls to `tool_executor`:

1. Extract tool use blocks from assistant response
2. For each tool:
   - Check permission mode (deny, allow, prompt, policy)
   - If prompt/policy: await user decision
   - Execute tool via executor
   - Capture result (stdout, stderr, exit code)
   - Wrap in `ToolResultBlock`
3. Append tool result to session
4. Continue loop (may trigger new assistant turn)

### File Operations

Static functions in `file_ops` module:

- `read_file(path)`: Read file; return metadata (size, charset) + content
- `write_file(path, content)`: Create or truncate; validate path safety
- `edit_file(path, edits)`: Apply hunks; generate unified diff before commit
- `glob_search(pattern)`: Find files matching glob; return paths
- `grep_search(query, file_pattern)`: Search files; return line + context

Path validation:

- Reject absolute paths
- Reject `../` traversal outside workspace
- Reject hidden files (leading `.`)
- Validate against config allowlist/blocklist

### Bash Execution

`execute_bash(command, input)`:

1. Spawn subprocess via `tokio::process`
2. Set timeout (configurable, default 30s)
3. Capture stdout/stderr; combine with stderr-to-stdout
4. Return `BashCommandOutput` (exit code, combined output)

Unsafe operations filtered:

- No `sudo` allowed
- No `rm -rf` without confirmation prompt
- No write to system paths (/etc, /usr, ...)

### Session Compaction

`compact_session()`:

1. Estimate total tokens via model's pricing
2. If approaching limit (e.g., 80% of context window):
   - Keep system message + last N user/assistant exchanges
   - Summarize old messages into single `ToolResult` block
   - Prepend summarization prompt to next turn
3. Return compaction metadata for tracing

### Config Loading

`ConfigLoader::load()`:

1. Check env vars (FRAUDE\_\* prefix)
2. Check dotenv file (.env, .env.local, .env.{profile})
3. Check YAML config files (~/.fraude/config.yaml, {workspace}/.fraude.yaml)
4. Merge with defaults
5. Validate (check required fields, enum values)

Config includes:

- Model (claude-opus, gpt-4, etc.)
- LLM provider (Anthropic, OpenAI, custom)
- Tool permissions (default deny/allow/prompt)
- File operation allowlists
- LSP server configs
- MCP server configs
- Session storage path

### OAuth

PKCE-based authorization flow:

1. Generate state + PKCE challenge (S256)
2. Redirect user to authorization URL
3. Listen on loopback; capture callback
4. Exchange code + verifier for tokens
5. Store tokens (encrypted at `~/.fraude/oauth/{provider}.json`)
6. Refresh on expiry via refresh token

Used for:

- MCP OAuth servers
- Remote session authentication
- Upstream proxy credentials

## Public API

```rust
pub struct ConversationRuntime { /* ... */ }
impl ConversationRuntime {
    pub fn new(session: Session, config: RuntimeConfig) -> Result<Self, RuntimeError>
    pub async fn run_turn(&mut self, message: &str, model: &str)
        -> Result<impl Stream<Item = AssistantEvent>, RuntimeError>
    pub fn session(&self) -> &Session
}

pub async fn execute_bash(input: BashCommandInput) -> Result<BashCommandOutput, RuntimeError>
pub async fn read_file(path: &str) -> Result<ReadFileOutput, RuntimeError>
pub async fn write_file(path: &str, content: &str) -> Result<WriteFileOutput, RuntimeError>
pub async fn edit_file(path: &str, edits: Vec<Hunk>) -> Result<EditFileOutput, RuntimeError>
pub async fn glob_search(pattern: &str) -> Result<GlobSearchOutput, RuntimeError>
pub async fn grep_search(input: GrepSearchInput) -> Result<GrepSearchOutput, RuntimeError>

pub async fn compact_session(
    session: &Session,
    config: &CompactionConfig,
    summarizer: &dyn LlmSummarizer,
) -> Result<CompactionResult, RuntimeError>

pub fn load_system_prompt(config: &RuntimeConfig) -> Result<String, RuntimeError>
pub fn estimate_session_tokens(session: &Session, model: &str) -> usize
```

## Async Architecture

All I/O is async via tokio:

- LLM API calls: streaming responses via `async_stream::stream`
- File operations: blocking ops wrapped in `spawn_blocking`
- Subprocess: tokio::process::Command
- LSP: async JSON-RPC with oneshot channels
- MCP: async stdio, websocket, or managed proxy transport

## Error Handling

`RuntimeError` enum:

- `ToolExecutionFailed`: Tool (bash, file ops, plugin) returned error
- `PermissionDenied`: User denied permission prompt or policy blocked
- `ApiError`: LLM provider returned error or connection failed
- `ConfigError`: Invalid runtime config (missing env var, bad path)
- `FileNotFound`: Attempt to read non-existent file
- `PathNotAllowed`: Path validation rejected (traversal, system path)
- `SessionError`: Message history corrupted or deserialization failed
- `LspError`: Language server failed to respond
- `McpError`: MCP server communication failed

## Dependencies

- `api`: LLM provider client
- `lsp`: Language server management
- `plugins`: Plugin tool execution
- `crux-types`: Trace model for turn results
- `tokio`: Async runtime
- `serde_json`: Message serialization
- `chrono`: Timestamps
- `sha2`: Content hashing

## Testing

Tests cover:

- Session state transitions (message append, role validation)
- Tool execution error handling (timeout, permission denied)
- File operations (path validation, safe writes)
- Bash command execution (timeout, signal handling)
- Session compaction (token estimation, summary injection)
- Config loading (env var override, merge order)
- OAuth flow (state validation, token storage)
