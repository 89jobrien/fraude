# gooey — Dashboard wired to the live agent loop

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude's ratatui dashboard (`fraude dashboard` / `fraude tui`) renders three panes
correctly but is driven by a scripted demo event sequence rather than the real agent loop.
The comment in `claw-cli/src/main.rs` notes the UI is driven by an `AgentEvent` stream
and "can be wired to the real agent loop without touching the rendering code." `gooey` is
an experimental TUI/UI framework that provides the reactive data pipeline layer the
dashboard needs.

## What it would do

Replace the scripted `AgentEvent` generator with a gooey reactive pipeline that bridges
the live `ConversationRuntime` event stream to the dashboard renderer. The three panes
become live:

- **Agent Pipeline** — shows real tool calls, results, and turn boundaries as they happen
- **Workspace** — recolors files as they are actually read/written by the agent
- **Live Diff** — streams the real file diff produced by `edit` tool calls

## Integration sketch

```
[current]
fraude dashboard
    └─► scripted_demo_events()
            └─► AgentEvent stream (fake)
                    └─► ratatui renderer

[proposed]
fraude dashboard
    └─► gooey::Pipeline::new()
            ├─ source: ConversationRuntime event channel (real)
            ├─ transform: RuntimeEvent → AgentEvent mapping
            └─ sink: ratatui renderer (unchanged)
```

The `RuntimeEvent` → `AgentEvent` mapping:

| RuntimeEvent              | AgentEvent                                           |
| ------------------------- | ---------------------------------------------------- |
| `TextDelta(s)`            | `Pipeline { step: "text", content: s }`              |
| `ToolUse { name, input }` | `Pipeline { step: name, content: input }`            |
| `ToolResult(output)`      | `Pipeline { step: "result", content: output }`       |
| `FileRead(path)`          | `Workspace { path, state: Scanning }`                |
| `FileWrite(path, diff)`   | `Workspace { path, state: Modified }` + `Diff(diff)` |

## Fraude changes required

1. Add `gooey` as a workspace dependency.
2. Replace `scripted_demo_events()` in `fraude-cli/src/main.rs` with a
   `gooey::Pipeline` connected to the runtime's event sender.
3. Emit `FileRead` and `FileWrite` events from the relevant tool handlers in
   `tools/src/lib.rs`.
4. Add a `RuntimeEvent` enum to `runtime` that the tool executor populates via a channel
   alongside the existing `AssistantEvent` stream.

## Dependencies

- `gooey` crate from `~/dev/gooey`
- A multi-producer event channel in `runtime` (e.g. `std::sync::mpsc` or `tokio::sync`)

## Reference

`~/dev/gooey` — source repo.
