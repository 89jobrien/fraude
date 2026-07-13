# hj — Handoff journal for session continuity

## Gap filled

Fraude saves sessions as raw JSON transcripts and resumes them with `--resume`. There is
no narrative summary of what happened, what was decided, or what is left to do. `hj`
(handoff journal) generates structured context snapshots — a prose summary with explicit
next-steps — that make resumption dramatically more useful than replaying a raw transcript.

## What it would do

At session end (or on `/compact`), fraude invokes `hj handoff` to write a `HANDOFF.md`
alongside the session JSON. On next `fraude --resume`, the handoff is prepended to the
system prompt so the model has immediate narrative context without re-reading the full
transcript.

## Integration sketch

```
Session end / /compact trigger
    │
    ▼
hj handoff --session <session.json> --out <session.HANDOFF.md>
    │
    ▼
HANDOFF.md written alongside session JSON

fraude --resume <session.json>
    │
    ▼
runtime::load_system_prompt()
    ├─ existing: load FRAUDE.md files
    └─ [proposed] if <session.HANDOFF.md> exists, prepend as context block
```

The handoff file is a lightweight Markdown document with sections: **What happened**,
**Decisions made**, **Open questions**, **Next steps**. The model reads it as part of the
system prompt and immediately knows the state of work.

## Fraude changes required

1. Add a `SessionEndHook` trait to `runtime` (parallel to the tool hook infrastructure).
2. Implement an `HjHandoffHook` that shells out to `hj handoff` at session end.
3. In `runtime::load_system_prompt`, check for a sidecar `<session>.HANDOFF.md` and
   inject it into the system prompt block when resuming.
4. Add `/handoff` as a slash command that triggers the hook manually mid-session.

## Dependencies

- `hj` binary on PATH
- Session JSON path must be stable (currently passed via `--resume`) so the sidecar
  filename can be derived

## Reference

`~/dev/hj` — source repo. Key command: `hj handoff`.
