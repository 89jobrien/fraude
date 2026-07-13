No external standard. Plugin system is custom-designed for Fraude.

## Custom Specification

The plugin architecture follows these internal design principles:

- **Manifest Format**: Superset of Claude Code plugin manifest. Compatible
  with Claude plugins but adds Fraude-specific fields (lifecycle, conditional
  permissions).
- **Permission Model**: Custom three-tier system (read/write/execute) with
  per-tool permission requirements (read-only, workspace-write,
  danger-full-access).
- **Marketplace IDs**: Format `{name}@{marketplace}` where marketplace is one
  of: builtin, bundled, external. Globally unique across Fraude instance.
- **Hook System**: Pre/post-tool-use hooks; init/shutdown lifecycle hooks.
  Execution order is registration order (no priority levels).
- **Tool Schema**: Input schema must be JSON Schema (draft 7 or later, though
  validation is deferred to caller).
- **Install Sources**: Git URLs (shallow clone) or local filesystem paths.

## Deviations

None; this is a custom system with no reference specification.

## Future Compatibility

If aligning with Claude Code plugins becomes necessary, the following changes
would be needed:

1. Drop Fraude-specific fields (lifecycle, marketplace ID suffix)
2. Rename CLAW*\* environment variables to CLAUDE*\*
3. Align permission model with Claude's read-only/workspace-write/danger-full
4. Support plugin versioning constraints (semver ranges)
5. Add marketplace registry protocol for install source validation
