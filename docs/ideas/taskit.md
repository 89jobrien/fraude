# taskit — CI pipeline runner and workspace health tool

## Gap filled

Fraude's `CONTRIBUTING.md` documents four manual commands (`cargo fmt`, `cargo clippy`,
`cargo test`, `cargo build`). There is no local CI pipeline, no affected-crate detection,
no protocol-drift tracking, and no git hook integration. `taskit` is a config-driven CI
pipeline runner built for exactly this — it provides a single `taskit ci` command that
runs the full gate sequence, plus `taskit quick` for fast per-PR feedback on affected
crates only.

## What it would do

**For contributors:** replace the four-command sequence in `CONTRIBUTING.md` with
`taskit ci` and `taskit quick`. Affected-crate detection means a change to `tools/` only
runs lint and tests for `tools` and its dependents — not the full workspace.

**For fraude's own development:** protocol-drift tracking guards against the config schema
(`RuntimeFeatureConfig`) diverging from the tool registry or the command registry without
a deliberate lockfile update. This is directly relevant to fraude's parity work — any
change to the tool surface should require an explicit `taskit check-protocol-drift
--update` commit.

**As a fraude tool:** expose `taskit` commands through fraude's tool registry so the
model can run CI gates, check workspace health, and review snapshots within a session.

## Integration sketch — taskit as a fraude development tool

```
# taskit.toml at fraude workspace root
[workspace]
[[workspace.crates]]
dir = "api"

[[workspace.crates]]
dir = "runtime"
# runtime depends on api — changing api also triggers runtime tests

[[workspace.crates]]
dir = "tools"

[[workspace.crates]]
dir = "commands"

[[workspace.crates]]
dir = "fraude-cli"

[[workspace.propagation]]
from = "runtime"
to   = ["tools", "commands", "fraude-cli"]
```

After setup:

```bash
taskit quick          # fmt-check + lint + test (affected crates only)
taskit ci             # full local CI gate
taskit health         # codebase health score vs baseline
taskit inspect        # warn/todo count thresholds
taskit pre-commit     # git hook delegate
taskit install-hooks  # wire pre-commit + pre-push hooks
```

## Integration sketch — taskit as a fraude tool

```
model calls: bash { command: "taskit quick" }
    └─► runs fmt-check + clippy + nextest on affected crates
    └─► returns pass/fail + per-crate output

model calls: bash { command: "taskit health" }
    └─► returns health score delta vs baseline
```

No special tool is needed — `taskit` is a well-behaved CLI that works through the
existing `bash` tool. The integration is a `taskit.toml` at the fraude workspace root
and adding `taskit` to the dev setup docs.

## Protocol drift for parity tracking

Fraude's parity gap is currently documented in `PARITY.md` (prose). A more robust
approach: use `taskit check-protocol-drift` to track the tool registry surface
(`tools/src/lib.rs`) and command registry (`commands/src/lib.rs`) as contract files.
Any addition to the registry requires a `taskit-protocol.lock` update — making parity
progress explicit and CI-enforced rather than documentation-only.

```toml
# taskit.toml
[[protocol.surfaces]]
file = "crates/tools/src/lib.rs"
description = "tool registry surface"

[[protocol.surfaces]]
file = "crates/commands/src/lib.rs"
description = "slash command registry surface"
```

## Fraude changes required

1. Add `taskit.toml` at `rust/` workspace root with crate ordering and propagation rules.
2. Run `taskit init` to generate `Cruxfile` and `.cargo/config.toml` defaults.
3. Run `taskit check-protocol-drift --update` to generate initial `taskit-protocol.lock`.
4. Update `CONTRIBUTING.md` to replace the four-command sequence with `taskit quick` /
   `taskit ci`.
5. Run `taskit install-hooks` to wire git hooks (replaces manual `cargo fmt` pre-commit).

## Dependencies

- `taskit` binary: `cargo install taskit`
- `cargo-nextest`, `cargo-deny` (installed via `taskit dev-setup`)

## Reference

`~/dev/taskit` — source repo. Key commands: `taskit ci`, `taskit quick`, `taskit health`,
`taskit check-protocol-drift`, `taskit install-hooks`.
