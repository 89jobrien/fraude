# Near-Term Improvements

1. **Plugin Versioning Constraints**
   Support semver in install source: `my-plugin@^1.2.3`. Enable safe updates
   with `update()` by checking version compatibility before executing
   lifecycle commands.

2. **Conditional Tool Exposure**
   Add `conditional` field to tool manifest: tools only exposed if condition
   (e.g., `{"requiresPermission": "execute"}`) is met at runtime. Enables
   permission-gated tool definitions without reloading plugins.

3. **Plugin Dependency Ordering**
   Add `dependencies` field to manifest: list plugin IDs that must be
   initialized before this one. Execute `initialize()` in dependency order;
   fail if cycle detected.

4. **Marketplace Registry**
   Implement remote plugin registry (`plugin:// URIs`). Install from registry
   with automatic version resolution. Track checksums to detect tampering.

5. **Plugin Crash Detection and Restart**
   Detect tool process non-zero exit code. On repeated failures for same tool,
   disable plugin automatically and log warning. User must explicitly
   re-enable.
