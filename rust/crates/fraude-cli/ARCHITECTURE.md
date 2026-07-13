## Module Breakdown

### Core

- **main.rs**: Argument parsing (`parse_args`), CliAction dispatch, entry point.
  Orchestrates OAuth flow, session recovery, REPL bootstrap, and one-shot
  prompt modes.

- **lib.rs**: Not present. All logic in `main.rs` and submodules.

### REPL & State

- **LiveCli** (in main.rs): REPL state machine holding model, runtime,
  permissions, session handle. Delegates to `ConversationRuntime` for turns.
  Handles slash commands via `handle_repl_command()`.

- **SessionHandle**, **ManagedSessionSummary**: Metadata for saved sessions.
  Sessions stored as JSON in `.fraude/sessions/session-*.json`.

### Terminal & Rendering

- **render.rs**: Markdown streaming (`MarkdownStreamState`), spinner animation,
  ANSI color management. Converts server streaming events to terminal output.

- **dashboard/**: Ratatui TUI split-pane layout. `demo.rs` runs a demo agent
  session. `mod.rs` coordinate layout and input dispatch.

- **input.rs**: Line editor wrapper around `rustyline`. Handles history,
  completion candidate generation for slash commands, multiline editing via
  Shift+Enter.

### Project Setup

- **init.rs**: Generates FRAUDE.md template, .fraude.json config, and local
  session directory. Copies workspace-level prompts if available.

### Supporting

- **helper functions**: Git status parsing, model alias resolution, permission
  mode normalization, OAuth browser open, etc.

## Data Flow

### Interactive REPL

1. User enters prompt at `> ` (or slash command)
2. `run_repl()` reads line via `LineEditor::read_line()`
3. If slash command: `LiveCli::handle_repl_command()` → command-specific logic
4. If text prompt:
   - Spin up `Spinner` ("🦀 Thinking...")
   - Call `runtime.run_turn(input, permission_prompter)`
   - Stream response via `MarkdownStreamState`
   - Update session on disk
   - Print spinner finish ("✨ Done")
5. Loop until `/exit` or Ctrl-D

### One-Shot Mode

1. `parse_args()` detects `-p "prompt"` or `prompt <text>`
2. Create `LiveCli` with tools enabled
3. Call `run_turn_with_output()` with requested format (text/json)
4. Serialize to stdout (JSON) or print text
5. Exit

### Dashboard Mode

1. `run_demo()` launches ratatui full-screen TUI
2. Split panes: input area, output area, status bar
3. Spawns internal agent turn on user text submission
4. Renders streamed content in output pane

### OAuth Flow

1. `run_login()` builds authorize URL with PKCE + state
2. Attempts browser open; fallback to manual paste
3. Listens on TCP loopback for callback
4. Extracts code and state, verifies CSRF
5. Exchanges code for tokens via `api::FraudeApiClient`
6. Saves tokens to `~/.fraude/oauth/` (via `runtime` module)

## Permission & Tool Integration

### Permission Modes

Three levels enforced by `runtime::PermissionMode`:

- `ReadOnly`: Only read/search tools
- `WorkspaceWrite`: Read + file edits
- `DangerFullAccess`: All tools unrestricted

Stored in `LiveCli::permission_mode`. User prompted per tool via
`CliPermissionPrompter` if in restrictive mode.

### Tool Registry

`GlobalToolRegistry` aggregated from:

- Built-in tool specs
- Plugin-provided tools (via `PluginManager`)
- Filtered by `allowed_tools` (if set via `--allowedTools`)

Passed to runtime; runtime requests permission for each tool call.

## Design Decisions

1. **State in LiveCli**: Avoids passing context through nested function calls.
   REPL state is single-threaded and mutable.

2. **Session on Disk**: Each session file is independent JSON. Allows resuming
   from file paths or session IDs. Compaction rebuilds the file in place.

3. **Streaming Renderer**: `MarkdownStreamState` increments line-by-line to
   support large responses without buffering. Spinner animates in parallel.

4. **Slash Commands in REPL Only**: Most slash commands require interactive
   state (model switching). A small subset (agents, skills) work via `--resume`.

5. **Async Runtime Bootstrap**: `ConversationRuntime` owns `tokio::Runtime`.
   REPL spawns one per session. Modal dialog execution via
   `run_internal_prompt_text_with_progress()`.

6. **Terminal Detection**: `is_terminal()` disables color/spinner in non-TTY.
   Allows shell script integration.

## Known Limitations

- Dashboard TUI is demo-only; not fully wired to production session management
- Slash commands are not pluggable; adding new commands requires code changes
- Session compaction blocks the REPL (should be async)
- OAuth callback parsing is fragile (single-line read + regex)
