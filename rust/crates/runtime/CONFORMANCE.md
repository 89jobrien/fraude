No external standard. Runtime is custom-designed for Fraude agents.

## Crux Compatibility

Turn results wrapped in `Crux<TurnSummary>` from crux-types crate. Enables
consumption by downstream systems (devloop, etc.) that expect crux trace model.

Format:

```rust
pub struct Crux<T> {
    pub data: T,
    pub metadata: CruxMetadata,
}

pub struct TurnSummary {
    pub messages: Vec<ConversationMessage>,
    pub token_usage: TokenUsage,
    pub stop_reason: StopReason,
}
```

Downstream systems can:

- Reconstruct full conversation from `Crux<TurnSummary>.data.messages`
- Estimate cost from `TokenUsage`
- Detect turn completion via `stop_reason`
- Access timing/metadata via `CruxMetadata`

## LLM Provider Abstraction

Runtime delegates to `api` crate. Currently supports:

- Anthropic Claude (text/computer-use models)
- OpenAI (GPT-4, etc.)
- Custom OpenAI-compatible endpoints

Does NOT mandate specific provider; implements abstraction layer to add
providers without modifying runtime.

## Permission Model

Custom three-tier model:

- `Allow`: Always execute (no consent)
- `Deny`: Never execute (fail with PermissionDenied)
- `Prompt`: Interactive approval (blocks turn)
- `Policy`: Check policy file; prompt if undefined

Not aligned with any OS permission model (Linux DAC, SELinux, AppArmor, etc.).
No integration with system-level security. Purely runtime-level.

## File Operation Safety

No formal specification. Follows these principles:

- Reject absolute paths (except explicitly allowed by config)
- Reject `../` traversal outside workspace
- Reject hidden files (`.` prefix)
- Reject system paths (/etc, /usr, /System, etc.)
- Require explicit allowlist for sensitive dirs

This is a custom security model, not aligned with POSIX or OWASP.

## OAuth 2.0

Implements RFC 6749 (OAuth 2.0 Authorization Framework) + RFC 7636 (PKCE):

- Authorization Code flow with PKCE (S256 challenge)
- Refresh token grant
- Implicit trust of localhost redirect (no TLS verification)

Known deviations from strict OAuth:

- Loopback redirect over HTTP (RFC 8252 allows this)
- No server validation of redirect URI (assumes server is trusted)
- Token storage unencrypted on disk (relies on OS file permissions)

## Session Format

Sessions serialized as JSON (serde_json):

```json
{
  "messages": [
    {
      "role": "user",
      "content": [{ "type": "text", "text": "..." }]
    }
  ]
}
```

Compatible with Claude API message format (subset). Can import/export to/from
Claude API endpoints.

## LSP Integration

Uses `lsp-types` crate (Language Server Protocol specification). Full LSP 3.x
compliance. No custom extensions or deviations.

## MCP Integration

Uses Model Context Protocol. Follows MCP 1.0 specification for stdio and
managed proxy transports. No deviations.

## Future Alignment

If building federation with other systems:

1. Crux trace format is stable; no changes expected
2. Permission model unlikely to align with OS-level security (keep separate)
3. Config loading could standardize on TOML (currently YAML + env)
4. OAuth token storage should encrypt at rest (use keyring library)
