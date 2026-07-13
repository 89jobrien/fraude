# TODO

- **Typed argument parsing**: replace ad-hoc string splitting with a typed argument parser
  that validates inputs before dispatch and produces structured errors.
- **Plugin-contributed commands**: expose a registration point so plugins can add slash
  commands without modifying the core registry.
- **Command aliases**: allow short aliases (e.g. `/m` for `/model`) configurable via
  `.fraude.json`.
- **Test coverage for dispatch**: `dispatch.rs` handler functions lack unit tests; add
  snapshot tests for each command's output against a mock runtime.
- **Completion metadata**: enrich `CommandManifestEntry` with argument schemas so the REPL
  can offer per-argument tab completion.
