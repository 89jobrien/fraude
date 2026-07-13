# TODO

- **Runtime input validation**: validate tool arguments against the JSON Schema before
  dispatch; return a structured `ToolError::InvalidInput` instead of letting handlers
  fail with opaque messages.
- **HTTP permission gate**: route outbound HTTP calls from custom tools through the
  permission system so users can allow/deny by domain.
- **Lazy plugin loading**: defer plugin tool aggregation until first use rather than
  loading all plugins at registry construction time.
- **Tool versioning**: add a `version` field to `ToolManifestEntry` so the LLM context
  can reflect which version of a tool is active when the schema changes.
- **Test harness**: add a mock `PluginManager` so registry unit tests can run without
  a real plugin installation.
