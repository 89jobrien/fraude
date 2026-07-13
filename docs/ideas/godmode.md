# godmode — Skills, agents, and commands content layer

## Gap filled

Fraude has the infrastructure for skills, agents, and user-defined slash commands but
ships with zero content in any of these registries. Godmode is a library of 71 skills, 101 agent definitions, and 37 slash commands — exactly the content layer
fraude's infrastructure was built to serve.

## Three integration points

### 1. Skill registry — load `~/.claude/skills/`

Fraude's `SkillTool` discovers `SKILL.md` files in the local directory. Godmode's skills
live at `~/.claude/skills/` (symlinked from `~/dev/godmode/skills/`). Pointing fraude's
skill search path at this directory gives instant access to the full godmode skill library.

**Fraude change:** Add `~/.fraude/skills/` and `~/.claude/skills/` to the skill search
path in `tools/src/lib.rs`. Merge with local skill discoveries rather than replacing them.

**Skill frontmatter schema** (what fraude should parse):

```markdown
---
name: my-skill
description: One-line summary used for search and /skills list
when_to_use: |
  Trigger conditions — when the model should invoke this skill
---

Skill body...
```

### 2. Agent definitions — load `~/.claude/agents/`

Fraude's `/agents` command is a stub. Godmode's `agents/` directory has 30+ agent `.md`
files with structured prompts, tool restrictions, and domain prefixes (`dbg__`, `qual__`,
`plan__`, `git__`).

**Fraude change:** Implement `/agents list` by scanning `~/.fraude/agents/` and
`~/.claude/agents/` for `.md` files with agent frontmatter. Implement `/agents run <name>`
by loading the agent definition and spawning it via `AgentTool` (see `braid.md`).

**Agent frontmatter schema:**

```markdown
---
name: sentinel
description: Structured code reviewer — flags issues, does not fix
tools: [read, grep, glob, bash]
---

System prompt body...
```

### 3. User-defined slash commands — load `~/.claude/commands/`

Godmode's `commands/` directory contains `/gm-*` slash commands as markdown files.
Fraude's command dispatcher can load these as user-defined commands alongside built-ins.

**Fraude change:** At startup, scan `~/.fraude/commands/` and `~/.claude/commands/` for
`.md` files. Register each as a slash command that, when invoked, reads the markdown body
and runs it as a skill prompt with the user's arguments injected.

**Command discovery:** A `/commands` slash command lists available user-defined commands
alongside built-ins, grouped by source (built-in vs user-defined vs plugin).

## Hook scripts as reference implementation

Godmode's hook scripts (`post/post-bash-redact.nu`, `pre/pre-commit-cargo-lint.nu`, etc.)
are a working reference for the `PreToolUse`/`PostToolUse` pipeline that fraude currently
lacks. Before implementing fraude's hook executor, read these scripts to understand the
expected input/output contract, the event shape, and common failure modes.

Key scripts to study:

| Script                          | Hook type        | What it does                                     |
| ------------------------------- | ---------------- | ------------------------------------------------ |
| `pre/pre-commit-cargo-lint.nu`  | PreToolUse/Bash  | Blocks `git commit` calls that would fail clippy |
| `post/post-bash-redact.nu`      | PostToolUse/Bash | Redacts secrets from bash output                 |
| `post/post-edit-cargo-fmt.nu`   | PostToolUse/Edit | Runs `cargo fmt` after any Rust file edit        |
| `post/post-edit-cargo-check.nu` | PostToolUse/Edit | Runs `cargo check` after Rust edits              |

These scripts read Claude tool input from stdin as JSON and write a hook result to stdout.
Fraude's hook executor should implement the same contract.

## Agent orchestration patterns

Godmode's `gm-dispatch-all`, `parallel-agents`, and `moa` skills encode patterns for
multi-agent fan-out that fraude's future `/agents` system should support. Rather than
designing these patterns from scratch, implement them as named built-in agents sourced
from godmode.

## Cross-compatibility goal

The long-term goal is full skill/agent/command cross-compatibility: a skill written for
godmode/Claude Code works in fraude without modification, and vice versa. This requires
fraude to parse the same frontmatter schema and honour the same `when_to_use` trigger
conventions.

## Dependencies

- `~/.claude/skills/` populated (symlinked from `~/dev/godmode/skills/`)
- `~/.claude/agents/` populated (symlinked from `~/dev/godmode/agents/`)
- `~/.claude/commands/` populated (symlinked from `~/dev/godmode/commands/`)

## Reference

`~/dev/godmode` — source repo.

- `skills/` — 71 skill `.md` files
- `agents/` — 101 agent `.md` files with domain prefixes
- `commands/` — slash command `.md` files
- `hooks/` — git hook scripts (pre-commit, pre-push) — not Claude Code hooks
