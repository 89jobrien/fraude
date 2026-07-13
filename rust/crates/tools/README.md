Global tool registry aggregating built-in tools and plugin tools. Provides
tool definitions, execution routing, and permission checking for the Fraude
runtime.

## Features

- **Built-In Tools**: Standard file and shell operations (read_file, write_file,
  edit_file, bash, grep_search, glob_search)
- **Plugin Tool Integration**: Aggregates tools from installed plugins
- **Tool Definitions**: JSON Schema-based input definitions for LLM tool calls
- **Permission Mapping**: Associates each tool with permission level (allow,
  deny, prompt, policy)
- **Tool Execution Routing**: Routes tool invocations to correct executor
  (bash, file ops, or plugin subprocess)
- **Alias Support**: Common shorthand aliases (read→read_file, write→write_file)
- **Conflict Detection**: Prevents duplicate tool names across builtin and
  plugin tools
- **Tool Normalization**: Case-insensitive tool name matching with canonical
  resolution

## Tool Categories

### Built-In Tools

Implemented directly in runtime crate:

- `read_file`: Read file content; return size, charset, text
- `write_file`: Create or overwrite file; validate path safety
- `edit_file`: Apply patches to file; return unified diff
- `bash`: Execute shell command; capture stdout/stderr/exit code
- `grep_search`: Search files by regex; return matching lines + context
- `glob_search`: Find files by glob pattern; return matching paths

### Plugin Tools

Tools provided by installed plugins. Each tool:

- Has unique name across all plugins
- Defines input schema (JSON Schema)
- Specifies required permission level
- Delegates execution to plugin subprocess

## Architecture

### GlobalToolRegistry

Central registry:

```rust
pub struct GlobalToolRegistry {
    plugin_tools: Vec<PluginTool>,
}
```

Maintains plugin tools; built-in tools stateless and re-derived on demand.

### API

```rust
impl GlobalToolRegistry {
    pub fn builtin() -> Self
    pub fn with_plugin_tools(tools: Vec<PluginTool>) -> Result<Self, String>
    pub fn normalize_allowed_tools(values: &[String])
        -> Result<Option<BTreeSet<String>>, String>
    pub fn definitions(allowed_tools: Option<&BTreeSet<String>>)
        -> Vec<ToolDefinition>
    pub fn permission_specs(allowed_tools: Option<&BTreeSet<String>>)
        -> Vec<(String, PermissionMode)>
    pub fn execute(name: &str, input: &Value) -> Result<String, String>
}
```

### Tool Definitions

Built-in tool specs (immutable):

```rust
pub struct ToolSpec {
    pub name: &'static str,
    pub description: &'static str,
    pub input_schema: Value,
    pub required_permission: PermissionMode,
}
```

Each spec:

- Defines tool name (immutable)
- Provides description for LLM
- Specifies JSON Schema for input
- Declares permission requirement

### Tool Normalization

`normalize_allowed_tools()` resolves tool names:

1. Split input by comma or whitespace
2. For each token:
   - Lowercase and replace dashes with underscores
   - Look up in canonical name map
   - Check built-in tools and plugin tools
3. Add aliases: read→read_file, write→write_file, edit→edit_file, glob→glob_search, grep→grep_search
4. Return normalized set of canonical names (or error if unknown tool)

Example: `"read,Write,grep_search"` → `{"read_file", "write_file", "grep_search"}`

### Execution Routing

`execute(name, input)`:

1. Check if built-in tool (via name)
2. If built-in: call `execute_tool(name, input)` → delegates to runtime functions
3. If plugin: find plugin tool by name
4. If plugin: call `plugin_tool.execute(input)` → subprocess with JSON stdin
5. Return stdout string or error

### Built-In Tool Details

#### read_file

Input:

```json
{ "path": "src/main.rs" }
```

Output:

```
"file contents..."
```

Validates path; returns 404 if missing.

#### write_file

Input:

```json
{ "path": "output.txt", "contents": "data" }
```

Output:

```
"Wrote 4 bytes to output.txt"
```

Creates parent directories if needed.

#### edit_file

Input:

```json
{
  "path": "src/main.rs",
  "edits": [
    {
      "start_line": 10,
      "end_line": 15,
      "replacement": "new code"
    }
  ]
}
```

Output:

```
"--- src/main.rs\n+++ src/main.rs\n..."
```

Returns unified diff showing before/after.

#### bash

Input:

```json
{ "command": "ls -la", "input": "" }
```

Output:

```
"file1.txt\nfile2.rs\n..."
```

Captures exit code + stdout/stderr.

#### grep_search

Input:

```json
{ "query": "TODO", "file_pattern": "**/*.rs", "context": 2 }
```

Output:

```
{"matches": [{"file": "src/main.rs", "line": 42, "text": "// TODO: fix"}]}
```

Returns matches with before/after context.

#### glob_search

Input:

```json
{ "pattern": "**/*.rs" }
```

Output:

```
{"paths": ["src/main.rs", "src/lib.rs"]}
```

Returns sorted file paths.

## Permission Model

Each tool has associated permission:

- `Allow`: Always execute
- `Deny`: Never execute
- `Prompt`: Ask user per invocation
- `Policy`: Check policy file

Runtime checks permission before invoking tool. If denied, returns
PermissionDenied error.

## Error Handling

Tool execution errors:

- Tool not found: "Unknown tool: foo"
- Plugin conflict: "duplicate plugin tool name"
- Execution failure: "tool `bash` returned exit code 1"
- Invalid input: "tool `read_file` input validation failed"

All errors returned as `Result<String, String>`.

## Plugin Integration

Registry created with `with_plugin_tools(plugin_tools)`:

1. Validate no plugin tool names conflict with built-in tools
2. Validate no duplicate plugin tool names
3. Create registry with plugin tools
4. On `definitions()`: return built-in + plugin tools
5. On `execute()`: route to appropriate executor

## Dependencies

- `runtime`: Built-in tool implementations (bash, file ops)
- `plugins`: PluginTool type; plugin tool execution
- `api`: Tool definitions for LLM
- `serde_json`: JSON input/output

## Testing

Tests cover:

- Built-in tool definitions (completeness, schema validity)
- Tool normalization (aliases, case-insensitivity, unknown tools)
- Plugin tool conflict detection
- Execution routing (builtin vs plugin)
- Permission specs (correct mode per tool)
- Error cases (missing files, invalid input, execution failure)
