Extensible plugin system for Fraude. Manages plugin discovery, lifecycle, hook
execution, and tool aggregation. Supports built-in, bundled, and external
plugins with manifest-based configuration.

## Features

- **Three-Tier Plugin Architecture**: Built-in (hardcoded), bundled (shipped),
  and external (user-installed)
- **Manifest-Based Configuration**: JSON manifest (`plugin.json`) declares
  metadata, permissions, hooks, lifecycle commands, and tools
- **Hook System**: Pre/post-tool-use hooks with path resolution and validation
- **Plugin Lifecycle**: Init and shutdown commands with error handling
- **Tool Aggregation**: Registry of plugin-provided tools with deduplication
  and permission checking
- **Install/Enable/Disable**: Full plugin lifecycle management via
  `PluginManager`
- **Permission Model**: Read, write, execute permissions per plugin; per-tool
  permission requirements (read-only, workspace-write, danger-full-access)
- **Registry Persistence**: Installed plugins tracked in JSON registry with
  install source (local path or git URL), versions, and timestamps
- **Path Resolution**: Hooks and tools resolve relative paths, validate
  existence before execution
- **Git URL Support**: Install plugins directly from git repositories with
  shallow clone

## Architecture

### lib.rs

Public exports: `PluginManager`, `PluginMetadata`, `PluginKind`, `PluginHooks`,
`PluginLifecycle`, `PluginRegistry`, `RegisteredPlugin`, `PluginTool`,
`PluginManifest`, error types.

### Plugin Kinds

- **Builtin**: Hardcoded example plugin; no filesystem operations
- **Bundled**: Shipped with Fraude; synced to install root on discovery
- **External**: User-installed plugins; full manifest validation

### PluginManager

Entry point for plugin operations:

- `new()`: Initialize with config (config home, install root, bundled root)
- `discover_plugins()`: Scan for builtin, bundled, and external plugins
- `plugin_registry()`: Discover and register plugins; return `PluginRegistry`
- `install()`: Clone git URL or copy local path; validate manifest; update
  registry
- `enable()` / `disable()`: Toggle plugin activation in settings
- `uninstall()`: Remove plugin (bundled plugins cannot be uninstalled)
- `update()`: Pull new version from source; update manifest
- `aggregated_hooks()`: Merge hooks from all enabled plugins
- `aggregated_tools()`: Collect tools from all enabled plugins; detect conflicts

### Manifest Validation

`PluginManifest` undergoes comprehensive validation:

- Required fields: name, version, description (non-empty)
- Permissions: Deduplicated, parsed (read/write/execute)
- Hooks: Path existence check (relative or absolute)
- Lifecycle: Init/shutdown command paths validated
- Tools: Input schema must be JSON object; required permission must be valid
- Commands: All fields non-empty; command paths validated
- Duplicates: Tool names and command names must be unique

### Tool Execution

`PluginTool::execute()`:

1. Serialize input to JSON
2. Spawn subprocess; pass plugin metadata via env vars (CLAW_PLUGIN_ID,
   CLAW_PLUGIN_NAME, CLAW_TOOL_NAME, CLAW_TOOL_INPUT)
3. Write input JSON to stdin
4. Capture stdout/stderr
5. Return stdout on success; stderr on failure

### Registry Persistence

`InstalledPluginRegistry` serialized to JSON at config home. Tracks:

- Plugin ID, name, version, description
- Install path (must exist)
- Install source (local path or git URL)
- Installed/updated timestamps (unix milliseconds)
- Kind (external, bundled, builtin)

Stale entries (missing install paths, unparseable manifests) cleaned up during
discovery.

### Plugin Synchronization

Bundled plugins auto-synced during discovery:

1. Scan bundled root for plugin directories
2. Load manifests; generate deterministic plugin IDs
3. Compare to registry: detect new, updated, or stale entries
4. Copy new/updated plugins to install root
5. Remove stale bundled entries from registry
6. Persist registry changes

## Public API

```rust
pub struct PluginManager { /* ... */ }
impl PluginManager {
    pub fn new(config: PluginManagerConfig) -> Self
    pub fn discover_plugins() -> Result<Vec<PluginDefinition>, PluginError>
    pub fn plugin_registry() -> Result<PluginRegistry, PluginError>
    pub fn aggregated_hooks() -> Result<PluginHooks, PluginError>
    pub fn aggregated_tools() -> Result<Vec<PluginTool>, PluginError>
    pub fn list_plugins() -> Result<Vec<PluginSummary>, PluginError>
    pub fn install(source: &str) -> Result<InstallOutcome, PluginError>
    pub fn enable(plugin_id: &str) -> Result<(), PluginError>
    pub fn disable(plugin_id: &str) -> Result<(), PluginError>
    pub fn uninstall(plugin_id: &str) -> Result<(), PluginError>
    pub fn update(plugin_id: &str) -> Result<UpdateOutcome, PluginError>
}

pub struct PluginRegistry { /* ... */ }
impl PluginRegistry {
    pub fn new(plugins: Vec<RegisteredPlugin>) -> Self
    pub fn plugins() -> &[RegisteredPlugin]
    pub fn get(plugin_id: &str) -> Option<&RegisteredPlugin>
    pub fn summaries() -> Vec<PluginSummary>
    pub fn aggregated_hooks() -> Result<PluginHooks, PluginError>
    pub fn aggregated_tools() -> Result<Vec<PluginTool>, PluginError>
    pub fn initialize() -> Result<(), PluginError>
    pub fn shutdown() -> Result<(), PluginError>
}
```

## Manifest Format

```json
{
  "name": "my-plugin",
  "version": "1.0.0",
  "description": "Description of plugin",
  "permissions": ["read", "write"],
  "defaultEnabled": true,
  "hooks": {
    "PreToolUse": ["./hooks/pre.sh"],
    "PostToolUse": ["./hooks/post.sh"]
  },
  "lifecycle": {
    "Init": ["./setup.sh"],
    "Shutdown": ["./cleanup.sh"]
  },
  "tools": [
    {
      "name": "my_tool",
      "description": "Tool description",
      "inputSchema": {"type": "object", "properties": {...}},
      "command": "./bin/tool",
      "args": ["--flag"],
      "requiredPermission": "workspace-write"
    }
  ],
  "commands": [
    {
      "name": "my-command",
      "description": "Command description",
      "command": "./bin/cmd"
    }
  ]
}
```

## Error Handling

- **IoError**: Filesystem operations (read manifest, spawn process, copy files)
- **JsonError**: Manifest parsing or registry serialization
- **ManifestValidation**: List of validation errors (empty fields, duplicates,
  missing paths, invalid permissions)
- **InvalidManifest**: Runtime errors (hook path not found, tool execution
  failed)
- **NotFound**: Plugin not in registry, manifest missing
- **CommandFailed**: Plugin hook/tool/lifecycle command exited non-zero

## Dependencies

- `serde` + `serde_json`: Manifest and registry serialization
- `std::process`: Plugin tool/lifecycle execution
- `std::fs`: File operations (copy, delete, read manifests)
- Re-exported: `HookRunner`, `HookEvent` from `hooks` module

## Testing

Tests cover:

- Manifest loading and validation (required fields, permissions, paths)
- Plugin discovery (builtin, bundled, external directories)
- Registry persistence (install, update, uninstall, enable/disable)
- Tool execution (stdin/stdout/stderr handling)
- Hook resolution (relative/absolute path conversion)
- Bundled plugin synchronization
- Error cases (missing manifest, invalid JSON, duplicate names)
