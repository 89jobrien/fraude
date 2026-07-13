# compat-harness

Extracts tool and command manifests from an upstream `claw-code` repository installation
and makes them available to Fraude's runtime. Acts as the bridge between the upstream
binary's bundled assets and Fraude's plugin and command registries.

## Purpose

`claw-code` ships its tool definitions, slash-command specs, and bootstrap configuration
as embedded assets. `compat-harness` resolves the on-disk paths of these assets, reads
and deserialises them, and returns a unified `ExtractedManifest` that Fraude can ingest
at startup.

## Usage

```rust
use compat_harness::{UpstreamPaths, extract_manifest};

let paths = UpstreamPaths::detect()?;
let manifest = extract_manifest(&paths)?;

// manifest.commands — Vec<CommandManifestEntry>
// manifest.tools    — Vec<ToolManifestEntry>
// manifest.bootstrap_plan — BootstrapPlan
```

## When to use

Only needed when running Fraude alongside an existing `claw-code` installation.
In standalone mode this crate is not required.
