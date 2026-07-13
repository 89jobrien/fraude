# crux — Typed agentic runtime for the conversation loop

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## What crux actually is

Crux is a typed runtime for building agentic workflows where every execution step is a
first-class value: recorded, serialised, and replayable after crashes. It is not a
hook framework — it is an entire execution model. Key capabilities:

- **`Crux<T>`** — every step produces a trace wrapping its result, agent name, causal
  chain, and child delegations
- **`CruxCtx`** — the execution context injected into agents; provides `step()`,
  `delegate()`, `speculate()`, `pipe()`, `join_all()`, `route_on_confidence()`
- **`TaskRegistry`** — crash-safe task lifecycle with checkpoint/resume; completed steps
  replay from cache rather than re-executing
- **`Planner` trait** — policy gate called before each step; returns `Allow`, `Deny`, or
  `Simulate` (enables dry-run mode, safety gates, permission prompts)
- **`#[crux::agent]`** proc macro — turns an `async fn` into a typed `Agent` impl with
  `CruxCtx` injected

## Gap filled in fraude

Fraude's conversation loop (`runtime::ConversationRuntime::turn`) is a bespoke
synchronous loop with no traceability, no crash recovery, no step-level policy gates, and
no structured execution model. The hook system (`PreToolUse`/`PostToolUse`) is the
narrowest version of the problem crux solves in full.

The three most immediately applicable crux capabilities for fraude:

### 1. Planner as the permission/hook layer

The `Planner` trait decides the fate of each step before it executes:

```
model issues ToolUse { name: "bash", input: "rm -rf /tmp/foo" }
    │
    ▼
CruxCtx::step("bash", closure)
    │
    ├─ Planner::next_action("bash", priority) called
    │     ├─ Allow(Action) → execute closure
    │     ├─ Deny { reason } → skip, return reason as tool_result
    │     └─ Simulate { output } → return fake output (dry-run / test mode)
    │
    ▼
tool_result appended to session
```

This replaces fraude's current `PermissionPolicy` check (a simple allow/deny) with a
full policy layer that also supports simulation — enabling a dry-run mode where the agent
plans without executing, which is a significant UX feature.

### 2. Typed step tracing for session transparency

Every tool call wrapped in `CruxCtx::step()` produces a `Step` record with:

- `name`, `kind`, `status`, `started_at`, `duration_ms`
- `input_hash`, `content_hash` — for replay matching
- `findings: Vec<CitedFinding>` — structured diagnostics

This gives fraude a structured execution log per session that is richer than the current
flat `ConversationMessage` transcript — the diff pane in the dashboard could show real
step timings and tool results from the trace rather than synthesising them from the
message stream.

### 3. Crash-safe sessions via TaskRegistry

Fraude's session persistence writes a JSON transcript at end-of-turn. If the process
dies mid-tool-call, the half-completed turn is lost. The `TaskRegistry` + `ReplayCache`
model checkpoints the trace after every step:

```
ConversationRuntime::turn()
    ├─ step 1: bash → completes → checkpoint to TaskRegistry
    ├─ step 2: read → completes → checkpoint to TaskRegistry
    └─ step 3: edit → process dies

fraude --resume <session>
    └─ resume_from(task_id) seeds ReplayCache with steps 1 and 2
    └─ step 1 and 2 replay from cache (no re-execution)
    └─ step 3 executes fresh
```

## Integration path

Full adoption (replacing `ConversationRuntime` with crux) is a large architectural
change. The practical path is incremental:

**Phase 1 (low effort):** Wrap each tool dispatch in `CruxCtx::step()` and use the
`Planner` trait as the permission/hook layer. This gives structured step tracing and
policy gates without touching the outer loop.

**Phase 2 (medium effort):** Wire the `Crux<T>` trace into the dashboard's `AgentEvent`
stream, replacing the scripted demo with real step-level events including timing and
confidence.

**Phase 3 (large):** Adopt `TaskRegistry` for crash-safe session checkpointing and
`#[crux::agent]` for each major tool handler, gaining full replay semantics.

## Speculation and branching

Crux's `speculate()` combinator runs multiple approaches and records winner + losers:

```rust
let result = x.speculate("choose_approach", vec![
    ("conservative_edit", conservative_closure),
    ("aggressive_refactor", aggressive_closure),
])
.pick_best_by(|r| r.confidence)
.await?;
```

For fraude this would enable a model to try multiple tool strategies and pick the best
result — a capability not expressible in the current single-path loop.

## YAML pipelines

Crux's `.crux` YAML pipeline format maps directly onto fraude's skill concept. A skill
written as a `.crux` file would be executable by the crux `Runner` rather than as a free-
form markdown prompt — giving skills typed inputs, structured steps, and a replay-safe
execution trace. This is a potential future direction for the fraude skill registry.

## Dependencies

- `crux` crate from `~/dev/crux`
- Crates: `crux-runtime`, `crux-types`, `crux-macros`, `crux-domain`, `crux-script`

## Reference

`~/dev/crux` — source repo. Key types: `Crux<T>`, `CruxCtx`, `Step`, `Agent`,
`Planner`, `TaskRegistry`, `ReplayCache`, `Budget`, `SpeculationBuilder`.
