## Module Breakdown

### lib.rs

Single module. Exports:

- `ToolSpec`: Static tool definition
- `GlobalToolRegistry`: Registry implementation
- `ToolManifestEntry`, `ToolSource`, `ToolRegistry`: Legacy types (may be
  deprecated)
- Functions: `mvp_tool_specs()`, `execute_tool()`, `permission_mode_from_plugin()`,
  `normalize_tool_name()`

## Data Flow

### Registry Creation

```
GlobalToolRegistry::with_plugin_tools(plugin_tools)
  ↓
Check plugin tools against builtin names
  ↓
Builtin names: read_file, write_file, edit_file, bash, grep_search, glob_search
  ↓
If any conflict: return error
  ↓
Check for duplicate plugin tool names (by name)
  ↓
If duplicates: return error
  ↓
Create GlobalToolRegistry { plugin_tools }
```

### Tool Normalization

```
normalize_allowed_tools(["read", "Write_File", "grep"])
  ↓
Split by comma/whitespace
  ↓
For each token:
  ↓
  Lowercase: "write_file"
  Replace `-` → `_`
  ↓
  Look up in name_map:
    alias "read" → canonical "read_file"
    canonical "write_file" → "write_file"
    canonical "grep_search" → lookup "grep" (not alias) → error
  ↓
Collect canonical names
  ↓
Return BTreeSet<String> or error
```

### Tool Definitions

```
definitions(allowed_tools)
  ↓
If allowed_tools is None: include all
  ↓
mvp_tool_specs() → builtin specs
  ↓
Filter by allowed_tools (if set)
  ↓
Convert ToolSpec → ToolDefinition {
  name,
  description,
  input_schema
}
  ↓
Append plugin tool definitions
  ↓
Filter plugin by allowed_tools
  ↓
Return Vec<ToolDefinition>
```

### Tool Execution

```
execute(name, input)
  ↓
Check if builtin (name in builtin set)
  ↓
If builtin:
  ↓
  execute_tool(name, input)
    ↓
    Dispatch to runtime function (execute_bash, read_file, etc.)
    ↓
    Return stdout string or error
  ↓
If not builtin:
  ↓
  Find plugin_tool by name
  ↓
  If not found: return "Unknown tool" error
  ↓
  plugin_tool.execute(input)
    ↓
    Spawn subprocess
    ↓
    Pass input JSON via stdin
    ↓
    Return stdout or error from stderr
```

### Permission Specs

```
permission_specs(allowed_tools)
  ↓
Map builtin tools → (name, PermissionMode)
  ↓
Filter by allowed_tools
  ↓
Map plugin tools → (name, permission_from_plugin(tool.required_permission()))
  ↓
Permission mode conversion:
  "read-only" → Allow
  "workspace-write" → Prompt
  "danger-full-access" → Prompt (or Deny if config)
  ↓
Return Vec<(String, PermissionMode)>
```

## Built-In Tool Specs

```rust
fn mvp_tool_specs() -> Vec<ToolSpec> {
  vec![
    ToolSpec {
      name: "read_file",
      description: "Read file contents",
      input_schema: json!({ "type": "object", "properties": { "path": ... } }),
      required_permission: Allow,
    },
    // ... (similar for write, edit, bash, grep, glob)
  ]
}
```

Each spec:

- Hardcoded name (immutable, used as canonical reference)
- Human-readable description (shown to LLM)
- JSON Schema (defines input structure for LLM tool calls)
- Permission requirement (used by runtime)

## Tool Name Normalization

Key insight: Tool names can be referred to by different forms:

- Canonical: `read_file` (as defined)
- Alias: `read` (short form)
- Variant: `Read_File`, `readFile`, `read-file` (case/format variants)

Algorithm:

1. Maintain canonical set (builtin + plugin names)
2. Maintain alias map (read→read_file, etc.)
3. When user specifies tool, normalize:
   - Lowercase
   - Replace `-` with `_`
   - Look up in alias map (first), then canonical
4. Return canonical name or error

This enables flexible user input while maintaining strict internal consistency.

## Permission Mode Conversion

Plugin tools declare permission as string:

- `read-only` → `Allow` (safe, always execute)
- `workspace-write` → `Prompt` (needs confirmation)
- `danger-full-access` → `Prompt` or `Deny` (depending on config)

Built-in tools have fixed modes:

- `read_file`: `Allow`
- `grep_search`: `Allow`
- `glob_search`: `Allow`
- `write_file`: `Prompt`
- `edit_file`: `Prompt`
- `bash`: `Prompt`

Conversion function `permission_mode_from_plugin()` maps string to
`PermissionMode`.

## Design Decisions

1. **Stateless Registry**: Built-in tools re-derived from specs, not cached.
   Enables regeneration; reduces memory.

2. **Plugin Tools Stored**: Plugin tools stored in registry (Vec). Enables
   fast lookup; immutable after construction.

3. **Normalization Layer**: Separate normalization step before execution.
   Centralizes name resolution; enables testing.

4. **Two-Level Lookup**: Builtin names hardcoded; plugin names dynamic. Enables
   both stability and extensibility.

5. **Alias Mapping**: Aliases (read→read_file) maintained in code. Enables
   UX-friendly short forms without bloating input spec.

6. **Error Collection**: Validation collects all errors at once
   (conflicts, duplicates). Single pass; user sees all issues.

7. **Optional Filtering**: `allowed_tools` parameter optional. Enables both
   "all tools" and "restricted set" modes for LLM.

## Known Limitations

- **No Tool Versioning**: Tools don't have versions. Can't offer multiple
  versions of same tool.
- **No Tool Deprecation**: No way to mark tools as deprecated. Removes tools
  breaks compatibility.
- **No Tool Priority**: No way to prefer one tool over another (e.g., prefer
  plugin bash over built-in).
- **No Dynamic Tool Discovery**: Built-in tools hardcoded. Adding new tool
  requires code change.
- **No Tool Metadata**: Tools only define name/description/schema. No author,
  license, changelog.
