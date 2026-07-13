## Module Breakdown

### lib.rs

Single file. Two proc-macros and supporting helpers.

## Derive Macro: ToolSpec

### Entry Point

`derive_tool_spec()` parses input token stream as `DeriveInput`:

1. Parse via `parse_macro_input!`
2. Delegate to `tool_spec_impl()`
3. Emit result or error

### Implementation: tool_spec_impl()

1. **Extract tool attribute**: Find `#[tool(...)]` on struct
   - Parse nested meta: `name = "...", description = "..., permission = "..."`
   - Return error if any required field missing
2. **Validate permission**: Map permission string to `PermissionMode` ident
   - Accepted: `"ReadOnly"`, `"WorkspaceWrite"`, `"DangerFullAccess"`
3. **Build field schema**: Call `build_field_schema()`
4. **Emit impl**: Generate `impl <Struct> { pub fn tool_spec() }` via `quote!`

### Helper: build_field_schema()

1. Extract named fields from struct
2. For each field:
   - Extract field name and doc comments
   - Call `unwrap_option()` to detect `Option<T>`
   - Call `type_to_schema()` to generate JSON Schema
   - If not optional, add to `required` list
3. Return vec of property fragments + required field names

### Helper: type_to_schema()

Recursively map Rust type to JSON schema (as `TokenStream2`):

1. Unwrap `Type::Path` to get last segment
2. Match segment ident:
   - Primitives: `String`, `str`, `bool`, `u8..u128`, `i8..i128`, `f32`, `f64`
   - Special: `Value` (any object), `Vec` (array with items schema)
3. Recursively call for generic args (e.g., `Vec<T>` → recurse on T)
4. Fallback: `{}` (any)

**Doc Comment Handling**: If description provided, inject into schema:

```rust
serde_json::json!({ "description": "...", "type": "..." })
```

### Helper: unwrap_option()

Detect `Option<T>` and extract inner type:

1. Match `Type::Path`
2. Find last segment with ident `"Option"`
3. Extract first generic arg as inner type
4. Return `(true, inner_type)` if found; `(false, input_type)` otherwise

### Helper: extract_doc_comment()

Collect doc comments from field/struct attributes:

1. Filter attributes where `path.is_ident("doc")`
2. Extract string literal from `Meta::NameValue`
3. Trim each line, join with spaces
4. Return concatenated result

## Declarative Macro: slash!

### Entry Point

`slash()` parses token stream as key-value pairs:

1. Parse as `Punctuated<FieldValue, Token![,]>`
2. Delegate to `slash_command_impl()`
3. Emit result or error

### Implementation: slash_command_impl()

1. Parse args via `parse_slash_command_args()`
2. Validate required fields: `name`, `summary`, `category`
3. Build `SlashCommandArgs` struct
4. Generate `SlashCommandSpec` literal via `quote!`

### Helper: parse_slash_command_args()

Extract key-value pairs:

1. Iterate `FieldValue` pairs
2. Match key (`syn::Member::Named`):
   - `"name"` → extract string literal
   - `"aliases"` → extract array of strings
   - `"summary"` → extract string literal
   - `"argument_hint"` → extract optional string
   - `"resume_supported"` → extract bool literal
   - `"category"` → extract path (identifier)
3. Validate required fields, return `SlashCommandArgs`

### Generated Code

```rust
SlashCommandSpec {
    name: "...",
    aliases: &["...", "..."],
    summary: "...",
    argument_hint: Some("..."),  // or None
    resume_supported: true,
    category: SlashCommandCategory::Workspace,
}
```

## Error Handling

- **Compilation Errors**: Return `syn::Error` with span, converted to
  `compile_error!` macro
- **Validation Errors**: Attribute parsing, missing fields, unknown permission
  - All surface as compile-time errors with helpful messages
- **Type Errors**: Unsupported types (e.g., custom structs) default to any
  schema (`{}`)

## Code Generation Strategy

All output via `quote!` macro, which produces clean `TokenStream2`:

- Field properties and required arrays inlined
- No string concatenation or format macros (avoids escaping bugs)
- Nested `json!` calls for schema objects (serde_json parses at compile time)

## Known Limitations

1. **No Schema Customization**: Cannot override generated schema with attributes
   (e.g., `#[schema(pattern = "...")]`)

2. **No Custom Type Support**: Custom structs / enums not supported. Use
   `serde_json::Value` as fallback.

3. **No Nested Generics**: `Vec<Vec<T>>` treated as single array (not
   array-of-arrays).

4. **Span Information**: Error spans point to struct/attribute, not exact field
   (could be more precise).

5. **No Trait Bounds**: Cannot add bounds like `where T: Serialize`. All fields
   assumed serializable.
