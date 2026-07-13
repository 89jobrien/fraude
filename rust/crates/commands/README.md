Slash command registry and handler library. Implements /agents, /skills,
/model, /status, /commit, /pr, and other REPL commands.

## Overview

Registers and dispatches slash commands for the Fraude CLI REPL. Provides
command metadata (specs, summaries, aliases), parsing, suggestion, and help
rendering. Handles both builtin commands and plugin-managed commands.

## Core Types

**SlashCommand** — enum of all recognized commands. Parse from user input with
`SlashCommand::parse()`. Each variant carries optional arguments (model name,
branch action, git paths, etc.).

**SlashCommandSpec** — metadata for a command: name, aliases, summary, argument
hint, category, and resume support flag.

**CommandRegistry** — manifest of available commands with source tracking
(Builtin, InternalOnly, FeatureGated).

**SlashCommandCategory** — grouping for help display: Core, Workspace, Session,
Git, Automation.

## Usage

```rust
use commands::{SlashCommand, slash_command_specs, render_slash_command_help};

// Parse input
let input = "/model claude-opus-4-6";
if let Some(SlashCommand::Model { model }) = SlashCommand::parse(input) {
    println!("Switch to: {}", model.unwrap_or("default"));
}

// List specs
for spec in slash_command_specs() {
    println!("{}: {}", spec.name, spec.summary);
}

// Render help
println!("{}", render_slash_command_help());

// Tab completion
let suggestions = suggest_slash_commands("/com", 10);
// → ["/commit", "/commit-push-pr", "/compact", "/config", ...]
```

## Command Categories

**Core** — /help, /status, /compact, /model, /permissions, /cost
**Workspace** — /config, /memory, /init, /diff, /teleport, /version
**Session** — /clear, /resume, /export, /session
**Git** — /branch, /worktree, /commit, /commit-push-pr, /pr, /issue
**Automation** — /bughunter, /ultraplan, /debug-tool-call, /agents, /skills,
/plugin

## Resume Support

Commands flagged `resume_supported: true` can be replayed when loading a saved
session. Session persistence captures the command string and re-parses it.

## Suggestions

`suggest_slash_commands(input, limit)` uses Levenshtein distance to rank matches.
Scoring: exact match (0), prefix match (1), substring match (2), fuzzy (3+).
Distance threshold is 2.
