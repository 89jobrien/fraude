## Standards & Protocols

### OAuth 2.0 + PKCE

Implements RFC 6234 (PKCE) for desktop app authorization flow:

- 128-char base64url-encoded code verifier
- SHA256 code challenge
- State parameter for CSRF protection
- Loopback redirect URI (http://127.0.0.1:PORT)

Fraude platform OAuth2 server expected to support `/oauth/authorize` and
`/v1/oauth/token` endpoints.

### JSON Session Format

Sessions serialized to JSON via `runtime::Session::save_to_path()`:

```
{
  "messages": [
    {"role": "user", "content": ...},
    {"role": "assistant", "content": ...}
  ]
}
```

Compatible with `runtime` crate's `ConversationRuntime` for resumption and
compaction.

### Markdown Output

Chat responses rendered as CommonMark (via `pulldown-cmark`). Code blocks
syntax-highlighted via `syntect` (Rust-aware theme).

### CLI Argument Convention

- Long flags with `=` (e.g., `--model=opus`) or separate (e.g., `--model opus`)
- Short flags for common args: `-p` (prompt), `-h` (help), `-V` (version)
- Subcommands: `init`, `login`, `logout`, `dashboard`, `prompt`, `agents`,
  `skills`

## Known Deviations

1. **Splash Command Hints**: Slash command argument hints are stored as
   `Option<&'static str>`. Should support rich schema (not implemented).

2. **Permission Prompt Format**: Text-based interactive prompt. LSP-compatible
   modal UI not yet implemented.

3. **Session ID Format**: Uses millisecond timestamp (`session-{millis}`). Does
   not follow UUID RFC 4122.

4. **Color Codes**: Uses ANSI 256-color escape sequences. Assumes terminal
   supports 256-color palette (may fail on older terminals).

5. **Markdown Streaming**: Processes line-by-line without buffer flushing
   between markdown blocks. Could lose output if process crashes mid-render.

6. **Tool Schema**: Uses JSON Schema draft 7 (via `serde_json`). Does not
   validate against strict JSON Schema spec.

## No External Standard

This crate does not conform to a single published standard. It implements
fragments of OAuth 2.0, JSON, CommonMark, and ANSI terminal specs, but is
primarily Fraude-specific (session format, slash command set, permission model).
