# looprs — Agent loop reference implementation

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude's conversation loop lives in `runtime::ConversationRuntime`. It works, but it is a
bespoke implementation with known divergences from reference behavior (JSON output
cleanliness, tool-result formatting edge cases). `looprs` is a purpose-built Rust LLM
agent loop CLI with multi-turn sessions, tool dispatch, and multi-provider support — it
exists to be the reference loop implementation.

## Two integration paths

### Path A — Benchmark and diff (lower effort)

Run looprs and fraude against the same prompt+tool sequence and diff their session
transcripts. Use the diff to identify where fraude's loop diverges from expected behavior.
This is a pure testing/validation use — no code integration required.

```bash
looprs run --prompt "list files in src/" --tools bash,read --session looprs.json
fraude prompt "list files in src/" --session fraude.json
diff <(jq '.messages' looprs.json) <(jq '.messages' fraude.json)
```

Useful for closing the JSON output cleanliness gap documented in `PARITY.md` without
rewriting fraude's loop.

### Path B — Adopt looprs primitives as a library (higher effort)

Extract looprs's loop primitives as a library crate and use them in fraude's `runtime`
as the turn-execution engine. Fraude's `ConversationRuntime` becomes a thin wrapper
around looprs's `AgentLoop`.

```
[current]
runtime::ConversationRuntime::turn()  ← bespoke loop

[proposed]
runtime::ConversationRuntime::turn()
    └─► looprs::AgentLoop::step(session, tools, provider)
            └─► returns: StepResult { events, tool_calls, usage }
```

## Recommendation

Start with Path A — it closes immediate parity gaps at zero integration cost. Move to
Path B if the loop continues to diverge after targeted fixes, or if looprs adds features
(retry logic, parallel tool calls, speculative execution) that fraude would want.

## Fraude changes required (Path A only)

1. Add a `fraude bench` subcommand that runs a prompt through both loops and diffs the
   output.
2. Add looprs session format as an accepted input to `--resume` (or add a converter).

## Dependencies

- `looprs` binary on PATH (Path A)
- `looprs` as a library crate dependency (Path B)

## Reference

`~/dev/looprs` — source repo.
