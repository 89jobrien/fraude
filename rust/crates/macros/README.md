Procedural macros for tool specification and command generation. Derives JSON
Schema from Rust struct definitions and generates tool metadata automatically.

## Features

- **ToolSpec Derive**: Auto-generates `tool_spec() -> ToolSpec` from annotated
  structs. Maps Rust types to JSON Schema properties with field validation.
- **Type-to-Schema Mapping**: Built-in conversion for primitives (string, bool,
  integer, float), containers (Vec, Option), and generic JSON (Value).
- **Doc Comments**: Struct and field doc comments become schema descriptions.
- **Permission Attributes**: Tool permission level (ReadOnly, WorkspaceWrite,
  DangerFullAccess) specified via `#[tool]` attribute.
- **Slash Command Macro**: `slash!` declarative macro for generating
  `SlashCommandSpec` literals with less boilerplate.

## Features by Macro

### #[derive(ToolSpec)]

Generates `impl <Struct> { pub fn tool_spec() -> ToolSpec { ... } }` for input
structs.

Required attribute:

```rust
#[tool(
    name = "tool_name",
    description = "Human-readable description.",
    permission = "ReadOnly" | "WorkspaceWrite" | "DangerFullAccess"
)]
```

Type mapping:

- `String` → `{"type": "string"}`
- `bool` → `{"type": "boolean"}`
- `u32, i32, i64, u64, usize` → `{"type": "integer"}`
- `f32, f64` → `{"type": "number"}`
- `Option<T>` → T's schema (field omitted from `required`)
- `Vec<T>` → `{"type": "array", "items": T's schema}`
- `serde_json::Value` → `{}` (any)

Field doc comments:

```rust
/// Glob pattern, e.g. `**/*.rs`.
pub glob: String,
```

Becomes:

```json
"properties": {
  "glob": {
    "type": "string",
    "description": "Glob pattern, e.g. `**/*.rs`."
  }
}
```

### slash! Macro

Generates `SlashCommandSpec` literal:

```rust
const MY_CMD: SlashCommandSpec = slash! {
    name: "mycommand",
    aliases: ["mc", "my"],
    summary: "Does something useful.",
    argument_hint: "<target>",
    resume_supported: true,
    category: Workspace,
};
```

Optional fields: `argument_hint`, `aliases` (default to `None` and `&[]`).

## Public API

```rust
#[proc_macro_derive(ToolSpec, attributes(tool))]
pub fn derive_tool_spec(input: TokenStream) -> TokenStream

#[proc_macro]
pub fn slash(input: TokenStream) -> TokenStream
```

Both macros are re-exported by the `macros` crate for use in dependent crates.

## Example

```rust
use macros::ToolSpec;
use serde::{Deserialize, Serialize};

#[derive(ToolSpec, Deserialize, Serialize)]
#[tool(
    name = "file_search",
    description = "Search for files by name or content.",
    permission = "ReadOnly"
)]
pub struct FileSearchInput {
    /// Filename pattern (glob or regex).
    pub pattern: String,

    /// Search directory (defaults to cwd).
    pub path: Option<String>,

    /// Maximum results to return.
    pub limit: Option<u32>,
}

// Expands to:
impl FileSearchInput {
    pub fn tool_spec() -> ToolSpec {
        ToolSpec {
            name: "file_search",
            description: "Search for files by name or content.",
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "pattern": {
                        "type": "string",
                        "description": "Filename pattern (glob or regex)."
                    },
                    "path": {
                        "type": "string",
                        "description": "Search directory (defaults to cwd)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum results to return."
                    }
                },
                "required": ["pattern"],
                "additionalProperties": false
            }),
            required_permission: PermissionMode::ReadOnly,
        }
    }
}
```

## Dependencies

- `proc-macro2`, `quote`: Macro code generation
- `syn`: Parsing Rust AST (with `full` and `extra-traits` features)

## Limitations

- Does not support tuple structs or unit structs (named fields only)
- Does not support nested custom types (use `serde_json::Value` for untyped)
- Nested generics (e.g., `Vec<Vec<T>>`) treated as single-level arrays
- No custom schema overrides (e.g., `#[schema(pattern = "...")]`)
