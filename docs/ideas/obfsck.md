# obfsck — Secret redaction as a PostToolUse hook

## Gap filled

Fraude parses `PreToolUse`/`PostToolUse` hooks from config but never executes them.
`obfsck` is a production-hardened secret-detection CLI already used as a pre-commit hook
in the workspace. Wiring it as the first `PostToolUse/Bash` hook gives fraude secret
redaction without writing any detection logic.

## What it would do

After every bash tool call, pipe the tool result through `obfsck redact` before it is
appended to the session transcript and sent back to the model. This prevents secrets that
appear in command output (env dumps, config files, `cat` of dotfiles) from leaking into
the conversation history or being echoed back in a subsequent model turn.

## Integration sketch

```
tools::LiveCli::execute("bash", input)
    └─► spawn subprocess, capture stdout+stderr
              │
              ▼
    [PostToolUse hook point — currently absent]
              │
              ▼
    obfsck::redact(output)   ← pipe through obfsck at --level standard
              │
              ▼
    redacted output appended to session
```

`obfsck` is invoked as a subprocess (`obfsck redact --level standard`) with the tool
result on stdin. The redacted stdout replaces the raw output. If `obfsck` is not on PATH,
the hook is skipped with a warning — not a hard failure.

## Fraude changes required

1. Implement the `PostToolUse` hook execution point in `runtime::ConversationRuntime::turn`
   (this is the prerequisite for any hook integration).
2. Add an `ObfsckRedactHook` struct implementing the hook trait.
3. Register it as the default `PostToolUse/Bash` hook when `obfsck` is detected on PATH.
4. Add `obfsck_redact: bool` to `.fraude/settings.json` to allow opt-out.

## Dependencies

- `obfsck` binary on PATH (installable from the `obfsck` crate in this workspace)
- Hook execution pipeline in `runtime` (see `PARITY.md` — currently config-only)

## Reference

`~/dev/obfsck` — source repo. CLI interface: `obfsck redact [--level minimal|standard|strict]`
