# devloop — Council analysis as a fraude tool

> **Note:** Type names and API shapes in integration sketches below are illustrative —
> they describe proposed interfaces, not current APIs in these projects.

## Gap filled

Fraude has `/diff` and git awareness but no code review or multi-perspective analysis
capability. `devloop` runs council analysis — parallel, multi-perspective review of code
changes — as a container-orchestrated pipeline. Exposing it as a fraude built-in tool
turns fraude into a self-reviewing coding agent.

## What it would do

A `council_analysis` tool call triggers devloop's review pipeline on the current workspace
diff. The pipeline runs multiple analysis perspectives (correctness, security, performance,
test coverage) in parallel and returns a structured findings report. The model can then
act on the findings — fixing issues, writing tests, explaining decisions — within the same
session.

```
model: "review my changes before I commit"
    │
    ▼
model calls: council_analysis { scope: "staged", perspectives: ["correctness", "security"] }
    │
    ▼
tools::LiveCli::execute("council_analysis", input)
    │
    ▼
shell out: devloop analyze --scope staged --format json
    │
    ▼
structured findings returned as tool result:
{
  "findings": [
    { "perspective": "security", "severity": "high", "file": "src/auth.rs", "message": "..." },
    ...
  ]
}
    │
    ▼
model reads findings, proposes fixes, calls edit/bash tools to address them
```

## Integration sketch

A `/review` slash command wraps the tool for direct user invocation:

```
/review [--scope staged|unstaged|HEAD] [--perspectives correctness,security,perf]
```

This maps to `devloop analyze` with the appropriate flags and prints the findings report
to the terminal, formatted for human reading.

## Fraude changes required

1. Add `CouncilAnalysisTool` to `tools/src/lib.rs`:
   - Input schema: `{ scope, perspectives[], format }`
   - Implementation: shell out to `devloop analyze`, parse JSON output
   - Required permission: `ReadOnly` (analysis only; no writes)
2. Add `/review` slash command in `commands/src/lib.rs` that invokes the tool and formats
   output.
3. Detect devloop on PATH at startup; register the tool only when available.

## Payoff

This is the highest-leverage quality integration: the model can generate code, immediately
review it from multiple perspectives, and self-correct — all in one session, without the
user manually running a separate review step.

## Dependencies

- `devloop` binary on PATH with `analyze` subcommand support
- Docker/container runtime (devloop's analysis pipeline is containerised)

## Reference

`~/dev/devloop` — source repo. Key command: `devloop analyze`.
