Primary interactive CLI entry point and REPL for Fraude. Provides a split-pane
TUI dashboard, command-line mode, OAuth login/logout, project initialization,
session management, and a comprehensive slash-command interface for model
switching, permission modes, git integration, and AI-powered assistance.

## Features

- **Interactive REPL**: Multi-turn conversations with model switching and
  permission mode control
- **Dashboard Mode**: Split-pane ratatui-based TUI for agent sessions
- **Session Management**: Save, resume, compact, and export conversations
- **OAuth Integration**: Login/logout with Fraude platform authentication
- **Project Initialization**: Bootstrap new projects with FRAUDE.md templates
- **Slash Commands**: `/help`, `/status`, `/model`, `/permissions`, `/commit`,
  `/pr`, `/issue`, `/ultraplan`, `/teleport`, `/bughunter`, `/compact`, and
  more
- **Permission Modes**: Read-only, workspace-write, or danger-full-access
  sandboxing
- **Tool Control**: Allowlist filtering and tool execution with user prompts
- **Output Formats**: Text (REPL) or JSON (batch mode)

## Architecture

### Entry Points

- `main()`: Dispatcher for CLI subcommands and REPL modes
- `run_repl()`: Interactive REPL loop with line editing and history
- `run_prompt()`: Single-turn CLI-mode inference
- `run_login()`, `run_logout()`: OAuth credential management
- `run_init()`: Project scaffolding

### State Management

`LiveCli` encapsulates the REPL state machine:

- Manages a `ConversationRuntime` with session history
- Tracks model, permissions, system prompt, and session file
- Handles command dispatch and rendering

### Rendering

- `MarkdownStreamState`: Streaming markdown-to-terminal renderer
- `Spinner`: Progress indicators
- `TerminalRenderer`: Color/style management
- `CliPermissionPrompter`: Interactive permission request dialogs

### Submodules

- `dashboard/`: Demo and live TUI via ratatui
- `render.rs`: Terminal formatting and streaming
- `init.rs`: FRAUDE.md template generation
- `input.rs`: Line editor with history and completion

## Public API

```rust
fn main() -> Result<(), Box<dyn std::error::Error>>
struct LiveCli { /* internal */ }
impl LiveCli {
    fn new(...) -> Result<Self, Box<dyn std::error::Error>>
    fn run_turn(&mut self, input: &str) -> Result<(), Box<dyn std::error::Error>>
    fn run_turn_with_output(...) -> Result<(), Box<dyn std::error::Error>>
}
enum CliAction { ... }
enum CliOutputFormat { Text, Json }
```

## Configuration

- Models: `opus`, `sonnet`, `haiku`, or full model IDs
- OAuth config: Client ID, endpoints, scopes (Fraude platform defaults)
- Session storage: `.fraude/sessions/` (JSON format)
- System prompt: Loaded from project FRAUDE.md or workspace defaults

## Dependencies

- `api`: HTTP client, auth, stream events
- `runtime`: Session, message format, permission engine
- `tools`: Global tool registry and execution
- `ratatui`, `rustyline`, `crossterm`: TUI and editing
- `tokio`: Async runtime and I/O
- `syntect`: Syntax highlighting for output

## Testing

Integration tests cover REPL command dispatch, OAuth flow, and session I/O.
Mock tests exercise permission prompting and tool filtering.
