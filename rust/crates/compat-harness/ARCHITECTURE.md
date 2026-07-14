# Architecture

## Overview

`compat-harness` is a thin extraction layer with no persistent state. It reads from the
filesystem, deserialises JSON, and returns plain data structures. There is no async code
and no caching.

## Module Layout

```
src/
  lib.rs   — UpstreamPaths, ExtractedManifest, extract_manifest
```

All logic lives in `lib.rs`. The crate is intentionally small; new upstream asset types
are added here rather than in the consuming crates.

## Data Flow

```
UpstreamPaths::detect()
  └─ resolves fraude installation root from PATH / well-known locations
        │
        ▼
extract_manifest(&paths)
  ├─ reads commands manifest JSON  → Vec<CommandManifestEntry>
  ├─ reads tools manifest JSON     → Vec<ToolManifestEntry>
  └─ reads bootstrap plan JSON     → BootstrapPlan
        │
        ▼
ExtractedManifest { commands, tools, bootstrap_plan }
```

## Design Decisions

- **No caching**: manifests are re-read each startup. Caching would complicate the
  lifecycle without measurable benefit given startup frequency.
- **Fail-fast**: `extract_manifest` returns `Err` if the upstream installation is absent
  or malformed. Callers decide whether to continue without upstream assets.
- **No transformation**: manifest entries are returned as-is, not adapted to Fraude's
  internal format at this layer. Adaptation is the consumer's responsibility.
