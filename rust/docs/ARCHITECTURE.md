# Architecture

This document describes the Rust workspace layout, crate responsibilities, and the
data flow through a live agent session. Read it before making cross-crate changes.

## Crate map

```
fraude-cli  ────────────────────────────────────────────────┐
  │  entry point, arg parsing, REPL, dashboard, OAuth flow  │
  │                                                         │
  ├─► runtime  ────────────────────────────────────────────┐│
  │     session, config, conversation loop, permissions,   ││
  │     MCP bootstrap, compaction, hook config             ││
  │                                                        ││
  ├─► tools  ──────────────────────────────────────────────┤│
  │     ToolSpec registry, ToolExecutor impl, all built-in ││
  │     tool dispatch (bash, file ops, glob, grep, web,    ││
  │     todo, skill, agent, plugin tools)                  ││
  │                                                        ││
  ├─► commands  ───────────────────────────────────────────┤│
  │     SlashCommand registry and handler logic            ││
  │     (/help, /status, /compact, /model, /permissions,  ││
  │      /clear, /cost, /resume, /config, /memory, /init, ││
  │      /diff, /version, /export, /session)               ││
  │                                                        ││
  ├─► api  ─────────────────────────────────────────────────┤│
  │     ProviderClient trait, Anthropic SSE client,        ││
  │     OpenAI-compatible client, xAI/Grok aliases,        ││
  │     model alias resolution, token limits               ││
  │                                                        ││
  ├─► plugins  ────────────────────────────────────────────┤│
  │     plugin discovery, manifest parsing, PluginTool     ││
  │     (lifecycle management is a planned gap)            ││
  │                                                        ││
  ├─► lsp  ─────────────────────────────────────────────────┤│
  │     LSP type definitions, workspace diagnostics        ││
  │     (wiring to agent loop is a planned gap)            ││
  │                                                        ││
  └─► server  ──────────────────────────────────────────────┘│
        Axum HTTP server (minimal; used for OAuth redirect)  │
                                                             │
compat-harness  ─────────────────────────────────────────────┘
  TypeScript surface extraction for parity verification
  (not part of the shipped binary)
```

## Request lifecycle

A single user turn through the agent loop:

```
User input
    │
    ▼
fraude-cli: parse slash command?
    ├─ yes ──► commands::dispatch(cmd) → print result → next turn
    └─ no  ──► build ApiRequest
                    │
                    ▼
             runtime::ConversationRuntime::turn()
                    │
                    ├─ [future] PreToolUse hooks (parsed, not yet executed)
                    │
                    ▼
             api::ProviderClient::stream(ApiRequest)
                    │
                    ▼
             AssistantEvent stream
                    │
                    ├─ TextDelta   → stream to terminal / TUI
                    ├─ Usage       → UsageTracker::record()
                    ├─ MessageStop → end of assistant turn
                    └─ ToolUse { name, input }
                              │
                              ▼
                    permission check (PermissionPolicy)
                              │
                              ├─ denied → tool_result: "permission denied"
                              └─ allowed
                                        │
                                        ▼
                              tools::LiveCli::execute(name, input)
                                        │
                                        ├─ bash     → spawn subprocess
                                        ├─ read     → fs::read_to_string
                                        ├─ write    → fs::write
                                        ├─ edit     → string patch
                                        ├─ glob     → glob::glob
                                        ├─ grep     → ripgrep / regex walk
                                        ├─ web_*    → reqwest
                                        ├─ todo_*   → in-memory store
                                        ├─ skill    → read SKILL.md file
                                        └─ plugin_* → PluginTool dispatch
                                                  │
                                                  ▼
                                        tool_result appended to session
                                                  │
                                                  ▼
                                    [future] PostToolUse hooks
                                                  │
                                                  ▼
                                        next API call (loop until
                                        MessageStop with no tool use)
```

## Key types

| Type                   | Crate      | Role                                                          |
| ---------------------- | ---------- | ------------------------------------------------------------- |
| `ConversationRuntime`  | `runtime`  | Owns the multi-turn loop, session state, compaction           |
| `Session`              | `runtime`  | Serialisable conversation history + metadata                  |
| `ApiClient`            | `runtime`  | Trait: `stream(ApiRequest) → Vec<AssistantEvent>`             |
| `ToolExecutor`         | `runtime`  | Trait: `execute(name, input) → Result<String>`                |
| `AssistantEvent`       | `runtime`  | Stream events: `TextDelta`, `ToolUse`, `Usage`, `MessageStop` |
| `ToolSpec`             | `tools`    | Static tool descriptor (name, schema, required permission)    |
| `GlobalToolRegistry`   | `tools`    | Registry of all live `ToolSpec` instances                     |
| `LiveCli`              | `tools`    | `ToolExecutor` impl; dispatches to concrete tool handlers     |
| `ProviderClient`       | `api`      | Trait: speaks to one LLM provider                             |
| `AnthropicClient`      | `api`      | SSE streaming impl for Anthropic API                          |
| `SlashCommandCategory` | `commands` | Enum grouping commands for `/help` rendering                  |
| `PluginTool`           | `plugins`  | Tool definition sourced from a discovered plugin manifest     |

## Trait boundaries

`runtime` defines the `ApiClient` and `ToolExecutor` traits but imports nothing from `api`
or `tools`. This keeps the core loop testable without real providers or tools.

`tools` imports from `api` (for provider types) and from `runtime` (for execution helpers
like `execute_bash`, `read_file`, etc.). It provides the concrete `LiveCli` that `fraude-cli`
wires into the loop.

`fraude-cli` is the composition root: it instantiates the provider client from `api`, the
executor from `tools`, and the runtime from `runtime`, then drives the REPL loop.

## Config resolution order

Settings are merged in this priority order (highest wins):

1. CLI flags (e.g. `--model`, `--permission`)
2. `.claw/settings.local.json` (machine-local, gitignored)
3. `.claw/settings.json` (project-level)
4. `~/.claw/settings.json` (user-level)
5. Compiled-in defaults

`CLAW.md` files are discovered by walking from the current directory to the repo root and
are concatenated into the system prompt, innermost last.

## Session persistence

Sessions are written to JSON (`Session` struct) at the end of each turn and can be resumed
with `--resume <path>`. The schema is versioned; older sessions are loaded as-is with
missing fields defaulting.

Compaction (`/compact`) summarises old turns via a separate API call and replaces the
transcript tail, keeping the session under the model's context window.

## Known gaps (as of v0.1)

These features are parsed or scaffolded but not yet wired:

- **Hook execution** — `PreToolUse`/`PostToolUse` hooks are parsed from config but the
  execution pipeline in `ConversationRuntime::turn` is absent. See `runtime::hooks`.
- **Plugin lifecycle** — Plugin manifests are discovered; install/enable/disable and
  plugin-provided commands/hooks are not yet implemented.
- **Full tool registry** — `AskUserQuestionTool`, `LSPTool`, `MCPTool`, `ScheduleCronTool`,
  `Task*`, `Team*`, and several others present in the TypeScript reference are missing.
- **MCP connection manager** — MCP config and stdio bootstrap exist; the dynamic
  connection manager and UI layer do not.
- **Dashboard live data** — The ratatui dashboard renders correctly but is driven by
  scripted events rather than the live `AgentEvent` stream.
- **`/skills`, `/agents`, `/mcp`, `/hooks`, `/plugin` commands** — Command stubs exist
  in the registry; handler logic is not yet implemented.

See `PARITY.md` for a full TypeScript-vs-Rust feature comparison.
