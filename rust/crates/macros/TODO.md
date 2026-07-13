# Near-Term Improvements

1. **Schema Customization Attributes**
   Add `#[schema(...)]` attribute to fields for constraints (minLength,
   pattern, minimum, maximum, etc.). Parse and inject into generated schema.

2. **Enum Support**
   Allow `enum` in tool inputs. Generate oneOf schema with discriminant
   variants. Requires recursive struct handling.

3. **Nested Type Flattening**
   Support `#[serde(flatten)]` for nested struct fields. Flatten their
   properties into parent schema (with name collision detection).

4. **Custom Type Predicates**
   Add `#[tool_spec(custom = "path::to::custom_schema_fn")]` to generate
   schema from a custom function instead of type.

5. **Better Error Messages**
   Include field names in compile errors (current errors point to struct
   level). Suggest nearby attribute names on typos.
