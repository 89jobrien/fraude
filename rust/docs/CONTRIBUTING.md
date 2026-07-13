# Contributing

## Build & test

```bash
cd rust
cargo fmt
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --release -p fraude-cli
```

All four commands must pass before opening a PR. There is no separate CI gate beyond this.

## Adding a built-in tool

Tools live in `crates/tools/src/lib.rs`. A tool has three parts:

**1. A `ToolSpec` constant**

```rust
pub const MY_TOOL_SPEC: ToolSpec = ToolSpec {
    name: "my_tool",
    description: "One sentence. What it does and when the model should call it.",
    input_schema: serde_json::json!({
        "type": "object",
        "properties": {
            "param": { "type": "string", "description": "..." }
        },
        "required": ["param"]
    }),
    required_permission: PermissionMode::WorkspaceWrite, // or ReadOnly / DangerFullAccess
};
```

**2. Registration in `mvp_tool_specs()`**

Add `MY_TOOL_SPEC` to the `vec![ ... ]` return value.

**3. Dispatch in `LiveCli::execute()`**

```rust
"my_tool" => {
    #[derive(Deserialize)]
    struct Input { param: String }
    let input: Input = serde_json::from_str(input_json)?;
    // ... implementation ...
    Ok(result_string)
}
```

Keep tool handlers pure where possible — accept text in, return text out. Side effects
(file writes, subprocess spawns) are expected but must respect the `required_permission`
level declared in the spec.

## Adding a slash command

Slash commands live in `crates/commands/src/lib.rs`.

**1. Add a `CommandManifestEntry`**

```rust
CommandManifestEntry {
    name: "/mycommand".to_string(),
    source: CommandSource::Builtin,
}
```

Register it in the manifest builder so it appears in `/help`.

**2. Add a `SlashCommandCategory` variant if needed**

Only add a new category if the command genuinely doesn't fit an existing one.

**3. Implement the handler**

Commands are dispatched in `fraude-cli` (`crates/fraude-cli/src/main.rs`) via a `match`
on the command name. Add a match arm that calls your handler and returns output as a
`String` to be printed.

Commands must not block indefinitely. If they need async work, spawn a thread and stream
output via the existing event channel.

## Adding a provider

Providers implement the `ProviderClient` trait in `crates/api/src/client.rs`. The trait
requires:

- `fn stream(&mut self, request: MessageRequest) -> Result<Vec<StreamEvent>>` — sends a
  request and returns the full event sequence.

Register the new provider in the model alias table (`crates/api/src/models.rs`) and wire
it in `fraude-cli`'s provider selection logic.

If the provider is OpenAI-compatible, prefer instantiating `OpenAiCompatClient` with the
appropriate base URL rather than writing a new client from scratch.

## Parity tracking

When implementing a feature that closes a gap listed in `PARITY.md`, update the relevant
section to reflect the new status. Keep the status labels consistent:

- `partial core only` — some functionality exists, important pieces missing
- `config-only; runtime behavior missing` — parsed/stored but not executed
- `missing` — not started
- `complete` — matches TypeScript reference behavior

## Style

- `cargo fmt` is enforced. Do not submit unformatted code.
- `cargo clippy -- -D warnings` is enforced. Fix all warnings, including pedantic ones
  that clippy surfaces with the workspace flags.
- No `unwrap()` in library code. Use `?` or explicit error handling.
- Error types: use `thiserror` for new error enums; do not introduce new `Box<dyn Error>`
  return types in library crates.
- Avoid `pub` on items that don't need to cross a crate boundary.
