# Conformance

## External Standards

No external protocol or specification governs this crate. It is an internal compatibility
shim whose contract is defined entirely by the layout of the upstream `claw-code`
installation.

## Informal Contracts

- **Path resolution**: `UpstreamPaths::detect` searches `PATH` and well-known installation
  prefixes (`/usr/local`, `~/.local`, platform app dirs).
- **Manifest format**: JSON files are expected to match the schemas defined in `commands`
  and `tools` crates. Unrecognised fields are ignored (serde `deny_unknown_fields` is
  not set).

## Known Deviations

- The upstream asset format is not versioned; silent breakage is possible when `claw-code`
  updates its manifest schema.
- `UpstreamPaths` does not verify the upstream binary's version before extracting assets.
