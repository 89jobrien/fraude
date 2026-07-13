# Architecture

## Module Hierarchy

```
api/
├── client.rs        Provider abstraction and MessageStream enum
├── providers/
│   ├── mod.rs       Provider trait, registry, model resolution
│   ├── fraude_provider.rs    Fraude/Anthropic impl + OAuth
│   └── openai_compat.rs      X.AI and OpenAI impl
├── sse.rs           SSE frame parsing
├── types.rs         Message request/response types
└── error.rs         ApiError enum
```

## Design Principles

**Provider Abstraction** — A Provider trait abstracts send_message and
stream_message across implementations. ProviderClient wraps provider instances
and delegates via generic helper functions (send_via_provider, stream_via_provider).

**Type Unification** — MessageRequest, MessageResponse, and StreamEvent types
are provider-agnostic. Providers adapt their native formats (OpenAI's choice,
Anthropic's messages) to these shared types.

**Lazy Auth** — Authentication is read from environment or provided explicitly
at client creation. No global state.

**Streaming Isolation** — MessageStream enum holds provider-specific stream
types. Callers see a single next_event() interface.

## Data Flow

```
ProviderClient::stream_message(request)
  ↓
  Match provider kind → provider.stream_message(request)
  ↓
  Provider impl parses HTTP SSE, yields provider-specific stream
  ↓
  MessageStream wraps stream, exposes next_event()
  ↓
  Caller loop: stream.next_event() → StreamEvent enum
```

## Model Resolution

Model aliases are resolved by scanning MODEL_REGISTRY:

- Short names (opus → claude-opus-4-6, grok → grok-3)
- Provider detection checks model name, then env (ANTHROPIC_API_KEY first,
  then OPENAI_API_KEY, then XAI_API_KEY, defaults to Fraude)
- Max tokens derived from model name (opus=32k, others=64k)

## OAuth Token Management

Fraude provider supports OAuth2. Token state is stored in OAuthTokenSet
(access_token, refresh_token, expires_at). Expired tokens trigger refresh via
saved credentials.

## Error Handling

ApiError wraps provider-specific and HTTP errors. No panics; all errors are
propagated as Result<T, ApiError>.
