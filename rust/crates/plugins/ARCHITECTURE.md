## Module Breakdown

### lib.rs

Central coordinator and public exports. Implements `PluginManager` and
`PluginRegistry`. Re-exports public types.

Key functions:

- `builtin_plugins()`: Returns hardcoded example builtin plugin
- `load_plugin_from_directory()`: Load manifest from `.fraude-plugin/plugin.json`
  or `plugin.json`
- `discover_plugin_dirs()`: Recursively scan directory for manifest presence
- Plugin ID generation: `plugin_id()` format `{name}@{marketplace}`
- Install source parsing: `parse_install_source()` detects git URLs vs local
  paths

### hooks.rs

Hook execution infrastructure (exported):

- `HookEvent`: Enum of hook types (PreToolUse, PostToolUse, Init, Shutdown)
- `HookRunner`: Executes hooks with error handling
- `HookRunResult`: Captures stdout/stderr/exit code

## Data Flow

### Plugin Discovery

1. `discover_plugins()` called
2. Sync bundled plugins (copy new/updated to install root)
3. Discover installed plugins:
   - Scan install root for manifest presence
   - Load registry; match records to filesystem
   - Clean up stale registry entries (missing paths, bad manifests)
4. Discover external directory plugins (from config.external_dirs)
5. Merge and deduplicate by plugin ID
6. Return list of `PluginDefinition`

### Plugin Registration

1. `plugin_registry()` calls `discover_plugins()`
2. For each plugin, load enabled state from settings
3. Create `RegisteredPlugin` (wraps definition + enabled flag)
4. Sort by plugin ID
5. Return `PluginRegistry`

### Hook Aggregation

1. `aggregated_hooks()` iterates enabled plugins
2. Validate each plugin (check hook paths exist)
3. Merge hooks:
   - PreToolUse: collect all pre.\* hooks in order
   - PostToolUse: collect all post.\* hooks in order
4. Return merged `PluginHooks`

### Tool Aggregation

1. `aggregated_tools()` iterates enabled plugins
2. Validate each plugin (check tool paths exist)
3. Collect tools into vec; track seen names in map
4. Detect conflicts: same tool name from different plugins → error
5. Return tool vector

### Plugin Installation

1. Parse source (git URL or local path)
2. Materialize source:
   - If local: use as-is
   - If git: clone to temp directory
3. Load and validate manifest
4. Generate plugin ID: `{name}@external`
5. Create install directory; copy plugin
6. Create/update registry entry with:
   - Metadata (id, name, version, description)
   - Install path and source
   - Timestamps (unix ms)
7. Update settings.json: enable by default
8. Return outcome (plugin_id, version, path)

### Manifest Validation Pipeline

Raw JSON → `RawPluginManifest` → Validation → `PluginManifest`

1. Parse JSON to `RawPluginManifest`
2. Validate required fields:
   - name, version, description: non-empty
3. Validate permissions:
   - Parse each permission string (read/write/execute)
   - Check for duplicates
   - Collect errors
4. Validate hooks:
   - Iterate pre/post hook paths
   - Check existence (if relative, resolve to root)
5. Validate lifecycle:
   - Check init/shutdown command paths
6. Validate tools:
   - Check name non-empty and unique
   - Check description non-empty
   - Check command non-empty and exists
   - Check inputSchema is JSON object
   - Validate requiredPermission enum (read-only, workspace-write,
     danger-full-access)
7. Validate commands:
   - Check name non-empty and unique
   - Check description non-empty
   - Check command non-empty and exists
8. If any errors, return `ManifestValidation` error with full error list
9. Otherwise, return validated `PluginManifest`

### Tool Execution

1. Caller: `tool.execute(input_json)`
2. Serialize input to JSON string
3. Spawn subprocess:
   - Command from manifest
   - Args from manifest
   - Env: FRAUDE_PLUGIN_ID, FRAUDE_PLUGIN_NAME, FRAUDE_TOOL_NAME,
     FRAUDE_TOOL_INPUT, FRAUDE_PLUGIN_ROOT
   - cwd: plugin root
   - stdin: piped
   - stdout: piped
   - stderr: piped
4. Write input JSON to stdin
5. `wait_with_output()`: collect output
6. If exit code 0: return stdout
7. If exit code != 0: return error with stderr

### Settings Persistence

Settings stored at `{config_home}/settings.json`:

```json
{
  "enabledPlugins": {
    "plugin-id@marketplace": true,
    "other-plugin@external": false
  }
}
```

Read on manager creation; written on enable/disable/install/uninstall.

## Design Decisions

1. **Three-Tier Plugin Architecture**: Separates built-in (hardcoded),
   bundled (shipped), and external (user-installed). Each tier has different
   permissions/lifecycle guarantees.

2. **Manifest-Driven**: All plugin metadata, capabilities, and paths defined
   in JSON. No hardcoding. Enables plugin discovery and validation upfront.

3. **Path Resolution**: Relative paths in manifests resolved to plugin root.
   Absolute paths accepted as-is. Validates existence before execution to catch
   errors early.

4. **Lazy Validation**: Manifests validated only when loaded; not at parse
   time. Returns all errors at once (not first-error).

5. **Deduplication at Registry Level**: Tool name conflicts detected in
   `aggregated_tools()`, not in manager. Centralizes conflict resolution.

6. **Environment Variable Naming**: Plugin env vars prefixed `FRAUDE_*` to
   distinguish from user env. Input JSON also passed via env for convenience.

7. **Install Source Tracking**: Records git URL or local path used for
   installation. Enables `update()` to re-sync from source.

8. **Bundled Sync**: Auto-sync on every discovery. Ensures bundled plugins
   always in-sync with latest version shipped in binary.

9. **Registry Cleanup**: Stale entries (missing paths, unparseable manifests)
   auto-removed during discovery. No orphaned registry entries.

10. **Immutable Config**: `PluginManagerConfig` never mutated after creation.
    Enables concurrent reads without locks.

## Known Limitations

- **No Plugin Dependencies**: Plugins cannot depend on other plugins. Load
  order is alphabetical by ID.
- **No Plugin Unload**: Disabled plugins still loaded. No memory cleanup
  available; requires process restart for full unload.
- **No Crash Recovery**: If plugin tool crashes, no retry logic. Caller
  handles errors.
- **No Plugin Isolation**: Plugins can read/write any file reachable from
  their cwd. No sandboxing.
- **No Hot Reload**: Plugin changes (manifest edits, binary updates) require
  re-discovery. No live updates.
- **Hardcoded Environment Variable Names**: FRAUDE\_\* names not configurable.
  Plugin env schema fixed.
