# slash — Composable slash-command parser

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude's slash-command layer is a hand-rolled `match` on command name strings in
`fraude-cli/src/main.rs`. Every new command requires manual argument parsing, help-text
registration, and category assignment. `slash` is an orchestration language and parser
designed exactly for this pattern — it provides composable pipelines, structured argument
validation, and declarative command definitions.

## What it would do

Replace the ad-hoc match dispatch with a `slash`-driven registry. Commands are declared
as `slash` definitions (name, args, flags, description), parsed uniformly, and dispatched
to handler functions. This reduces boilerplate for each new command from ~40 lines to ~10
and makes the missing command families (`/agents`, `/mcp`, `/tasks`, `/hooks`, `/plugin`)
straightforward to add.

## Integration sketch

```
user types: /model claude-opus-4-8 --save

[current]
fraude-cli/src/main.rs:
    match cmd {
        "/model" => { /* manual arg split, manual flag parse */ }
        ...
    }

[proposed]
slash::parse("/model claude-opus-4-8 --save")
    └─► SlashCommand { name: "model", args: ["claude-opus-4-8"], flags: { save: true } }
              │
              ▼
    commands::dispatch(parsed_command)
              │
              ▼
    handler fn model(args, flags, session) -> CommandResult
```

Command definitions live in `commands/src/lib.rs` as slash schema declarations. The
`/help` output is generated from the schema rather than maintained by hand.

## Fraude changes required

1. Add `slash` as a workspace dependency.
2. Replace the `match cmd_name { ... }` block in `fraude-cli/src/main.rs` with a
   `slash::Registry` that routes parsed commands to handler fns.
3. Convert each existing command handler to accept a `SlashCommand` struct rather than a
   raw string.
4. Remove per-command argument parsing boilerplate; let slash handle it.
5. Regenerate `/help` output from the registry schema.

## Payoff

Once the registry exists, adding the missing command families is a matter of writing a
handler fn and registering a schema — not adding a new match arm and manual parser.

## Dependencies

- `slash` crate from `~/dev/slash`

## Reference

`~/dev/slash` — source repo. Key types: `Registry`, `SlashCommand`, `CommandDef`.
