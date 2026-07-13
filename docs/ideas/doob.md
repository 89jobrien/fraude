# doob — Todo tool backend

## Gap filled

Fraude's `Todo*` tool family (`todo_read`, `todo_write`) is currently an in-memory store
that does not persist across sessions. `doob` is a full todo/task tracker with a SurrealDB
backend, GitHub sync, and a CLI. Pointing fraude's todo tools at doob gives persistent,
cross-session task tracking with no new storage code.

## What it would do

The model calls `todo_write` to create or update tasks; those tasks survive session end,
appear in `doob todo list`, and can sync to GitHub Issues. A user's in-flight work items
accumulate across coding sessions rather than resetting each time.

## Integration sketch

```
model calls todo_write { op: "create", content: "...", priority: "high" }
    │
    ▼
tools::LiveCli::execute("todo_write", input)
    │
    ├─ [current] write to in-memory Vec<TodoItem>   ← ephemeral
    └─ [proposed] shell out: doob todo add "<content>" --priority high
                      │
                      ▼
                  doob writes to SurrealDB
                  available via: doob todo list
                                 doob sync (→ GitHub Issues)
```

`todo_read` maps to `doob todo list --json --status pending`.
`todo_write` with `op: "complete"` maps to `doob todo complete <id>`.

## Fraude changes required

1. In `tools/src/lib.rs`, add a `DoobBackend` enum variant alongside the existing
   in-memory backend.
2. Detect doob on PATH at startup; fall back to in-memory if absent.
3. Map the existing `TodoItem` schema to doob's CLI flags (`--priority`, `--status`,
   `--tag`).
4. Add `todo_backend: "doob" | "memory"` to `.fraude/settings.json`.

## Session continuity bonus

On session start, `todo_read` can pre-populate the model's context with open doob items
tagged to the current repo, giving the model awareness of outstanding work without the user
having to re-state it.

## Dependencies

- `doob` binary on PATH
- `doob` SurrealDB instance running (managed by `doob serve` or system service)

## Reference

`~/dev/doob` — source repo. Key commands: `doob todo add`, `doob todo list`, `doob todo
complete`, `doob sync`.
