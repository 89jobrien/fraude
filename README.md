# Fraude

A local coding-agent CLI written in safe Rust. Claude Code inspired, clean-room implementation.

## Repository layout

```
rust/          Rust workspace — the active product (fraude binary + supporting crates)
src/           Python tooling: parity analysis, reference snapshots, prototype surfaces
docs/          Ideas, architecture notes, release drafts
```

## Rust workspace

The `fraude` binary is built from the `rust/` workspace.

### Crates

| Crate            | Role                                                                       |
| ---------------- | -------------------------------------------------------------------------- |
| `fraude-cli`     | User-facing binary: REPL, prompt, session management, TUI dashboard        |
| `runtime`        | Sessions, config, permissions, prompts, tool loop                          |
| `api`            | Provider clients (Anthropic, Grok, OAuth) and streaming                    |
| `tools`          | Built-in tool implementations (shell, file I/O, search, web, todos)        |
| `commands`       | Slash-command registry and handlers                                        |
| `plugins`        | Plugin discovery, registry, manifest parsing, and lifecycle                |
| `lsp`            | Language-server protocol types and process helpers                         |
| `server`         | Supporting service layer                                                   |
| `compat-harness` | Compatibility tooling for extracting manifests from existing installations |
| `macros`         | Proc-macro utilities                                                       |

### Build

```bash
# Prerequisites: Rust stable toolchain

cargo build --release -p fraude-cli

# Or install locally
cargo install --path rust/crates/fraude-cli --locked
```

### Authentication

```bash
export ANTHROPIC_API_KEY="..."       # Anthropic models
export XAI_API_KEY="..."             # Grok models

# Or use OAuth login
fraude login
```

### Run

```bash
fraude                                  # interactive REPL
fraude prompt "summarize this repo"     # one-shot
fraude --model sonnet "review changes"  # model override
fraude agents                           # list available agents
fraude skills                           # list available skills
```

### Verify

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --test-threads=1
```

## Configuration

| File                          | Scope                                          |
| ----------------------------- | ---------------------------------------------- |
| `~/.fraude/settings.json`     | User-level defaults                            |
| `.fraude/settings.json`       | Project-level config                           |
| `.fraude/settings.local.json` | Machine-local overrides (gitignore this)       |
| `.fraude.json`                | Legacy single-file config                      |
| `FRAUDE.md`                   | Workspace instruction file loaded into context |

`FRAUDE_CONFIG_HOME` overrides the user config directory.

## Capabilities

- Interactive REPL and one-shot prompt execution
- Session save, resume, and inspection
- Built-in tools: shell, file read/write/edit, glob, grep, web fetch/search, todos, notebooks
- Slash commands: `/status`, `/config`, `/diff`, `/compact`, `/export`, `/version`, and more
- Local agent and skill discovery (`fraude agents`, `fraude skills`)
- Plugin system with manifest-driven discovery and CLI management
- Workspace-aware instruction loading (`FRAUDE.md`, nested config files)
- OAuth login and per-request model selection

## License

MIT OR Apache-2.0
