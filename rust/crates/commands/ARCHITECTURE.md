# Architecture

## Module Hierarchy

```
commands/
├── lib.rs       Main types, parsing, spec registry
└── (no submodules — monolithic design)
```

## Design Patterns

**Registry as Static Array** — All command specs are compile-time constants in
SLASH_COMMAND_SPECS. No dynamic registration; commands are hardcoded and must
be updated in tandem with REPL implementation.

**Parse-Don't-Execute** — SlashCommand::parse() returns an enum; callers
dispatch to handlers. The crate does not execute commands—it only parses and
provides metadata.

**Categories for Grouping** — SlashCommandCategory classifies commands for help
rendering, filtering, and discovery. Categories are display-only and don't
affect parsing.

**Suggestion Engine** — Levenshtein distance ranks fuzzy matches. Aliases are
checked alongside primary names.

## Data Flow

```
User input ("/model opus")
  ↓
SlashCommand::parse()
  ↓
Split on whitespace, match first token
  ↓
Return enum variant with arguments
  ↓
Caller matches on variant, dispatches to handler
```

## Resume Semantics

Commands with `resume_supported: true` can be restored from session snapshots.
The REPL stores the command string (e.g., "/status") and re-parses on resume.
Commands like /pr or /teleport (resume_supported: false) are skipped during
resume because they depend on interactive state.

## Argument Parsing

Simple split-on-whitespace for positional args. Multi-word arguments (e.g., PR
context) use `remainder_after_command()` to capture everything after the command
name.

```rust
"/commit-push-pr add support for caching"
  → CommitPushPr { context: Some("add support for caching") }
```

## Spec-Driven Help

Help text is generated from specs at runtime. SlashCommandSpec is the source of
truth; the REPL queries it for summaries, aliases, and resumability.
