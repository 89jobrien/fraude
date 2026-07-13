## Standards & Protocols

### JSON Schema (Draft 7)

Generated schemas conform to JSON Schema Draft 7 (from serde_json):

- `type` field: string, boolean, integer, number, array, object
- `properties` object: map of field name → property schema
- `required` array: list of required field names
- `items` object: schema for array elements
- `description` string: field or object documentation
- `additionalProperties: false`: Rejects unknown fields

See: https://json-schema.org/draft-07/

### Rust Proc-Macro Convention

Follows Rust proc-macro 2.0 conventions:

- `#[proc_macro_derive(...)]`: Attribute-like derive macros
- `#[proc_macro]`: Function-like declarative macros
- Error handling via `syn::Error` (compile_error! at usage)
- Code generation via `quote!` (from `quote` crate)
- AST parsing via `syn::parse_macro_input!`

## Known Deviations

1. **Limited JSON Schema Features**: Does not generate `minLength`,
   `maxLength`, `pattern`, `minimum`, `maximum`, or other validation
   constraints. Only `type` and `description`.

2. **No Schema Overrides**: Cannot annotate fields with custom schema (e.g.,
   `#[schema(minimum = 0)]`). Generated schema is deterministic from type.

3. **No Enum Support**: Tool inputs cannot use Rust `enum` (only structs with
   named fields). Workaround: use string discriminator field.

4. **No Nested Custom Types**: Cannot nest custom structs (would require
   recursive schema generation). Use `serde_json::Value`.

5. **Flatten Attribute Ignored**: `#[serde(flatten)]` not handled; nested
   struct fields treated as regular fields (schema generation skips them).

6. **Required Array Only**: Schema uses `required` array. Does not generate
   `dependentRequired` or conditional requirements.

## No External Standard

Macros are Fraude-specific. Generated schema conforms to JSON Schema Draft 7,
but the macro itself (attributes, generated impl) is proprietary.
