# braid — Multi-agent orchestration

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude is a single-agent CLI. The TypeScript reference supports spawning subagents, running
tasks in parallel, and coordinating work across agent instances. `braid` is a multi-agent
orchestration platform in Rust that provides exactly this infrastructure.

## What it would do

Braid enables fraude to:

- Spawn named subagents with isolated tool scopes and session state
- Dispatch work items to agents in parallel (e.g. "review these 5 files simultaneously")
- Aggregate results from parallel agents into the parent session
- Persist inter-agent state across turns

This closes the `/agents` command family gap and enables the `AgentTool` — the most
significant missing tool in fraude's registry.

## Integration sketch

```
model calls AgentTool { prompt: "review src/lib.rs for bugs", agent: "reviewer" }
    │
    ▼
tools::LiveCli::execute("agent", input)
    │
    ▼
braid::spawn_agent(AgentSpec {
    prompt,
    tools: ["read", "grep"],          // restricted tool scope
    provider: parent.provider.clone(),
    session: None,                    // fresh session
})
    │
    ▼
braid runs subagent turn loop, streams events back to parent
    │
    ▼
AgentTool returns subagent's final message as tool result
```

For parallel dispatch (fan-out pattern):

```
braid::spawn_many(vec![spec_a, spec_b, spec_c])
    └─► runs concurrently, returns Vec<AgentResult>
```

## Fraude changes required

1. Add `braid` as a workspace dependency.
2. Implement `AgentTool` in `tools/src/lib.rs` using `braid::spawn_agent`.
3. Implement `/agents list` and `/agents run <name>` commands backed by braid's agent
   registry.
4. Add agent definitions to `.fraude/settings.json` under `"agents": { ... }` — same
   schema as Claude Code's agent definitions.
5. Wire braid's event stream into fraude's TUI `AgentEvent` channel so subagent progress
   is visible in the dashboard.

## Scope note

This is the largest integration in this list. The prerequisite is a stable single-agent
loop (all `PARITY.md` blocking items resolved) before adding multi-agent complexity.

## Dependencies

- `braid` crate from `~/dev/braid`

## Reference

`~/dev/braid` — source repo.
