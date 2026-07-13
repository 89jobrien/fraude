No external standard. Server is custom-designed for Fraude agent loops.

## HTTP/REST Conventions

Follows common REST patterns:

- HTTP methods: POST (create), GET (retrieve), no PUT/PATCH (not needed for
  MVP)
- Status codes: 201 Created, 202 Accepted, 404 Not Found, 500 Internal Error
- JSON request/response bodies
- Standard error format: `{"error": "message"}`

Not strictly RESTful (URI resources are sessions, but message send uses
/sessions/{id}/message not POST body to /sessions/{id}). This is pragmatic
over doctrinaire.

## Server-Sent Events (SSE)

Follows RFC 8664 (Server-Sent Events):

- Event names: `text`, `tool_call`, `tool_result`, `metadata`, `snapshot`
- Event data: JSON serialized
- Standard `data:` framing
- Standard `event:` type field

No custom SSE extension or deviations.

## Session Persistence

No standard. In-memory only (no wire protocol for persistence). Implementers
adding persistence must define their own schema.

## Event Format

Events match `AssistantEvent` from runtime crate, converted to JSON:

```rust
pub enum AssistantEvent {
    Text(String),
    ToolCall(ToolUseBlock),
    ToolResult(ToolResultBlock),
    Metadata(TurnMetadata),
    Final(Crux<TurnSummary>),
}
```

Serialized as JSON with `type` discriminant:

```json
{"type": "text", "delta": "Hello"}
{"type": "tool_call", "tool": "bash", "input": {...}}
```

## Error Responses

Standard JSON error format:

```json
{ "error": "error message" }
```

Not aligned with any standard (no Problem Details RFC 9457, no JSON:API error
format). Minimal and pragmatic.

## Deployment

No mandated deployment model. Can be:

- Standalone HTTP server
- Deployed to cloud (AWS Lambda, Google Cloud Run, etc.)
- Behind reverse proxy (nginx, Cloudflare, etc.)
- In containerized orchestration (Docker, Kubernetes)

Stateless (except in-memory sessions), so horizontal scaling requires sticky
sessions or distributed state.

## Security Implications

This server has NO built-in security:

- No authentication (anyone can create/access sessions)
- No authorization (no per-user session isolation)
- No rate limiting
- No CORS policy
- No CSRF protection
- No input validation (beyond JSON schema)

Must be deployed behind a security layer (API gateway, proxy, auth service).
Suitable only for:

- Local development
- Internal network (behind VPN)
- Trusted environments with external auth

## Future Alignment

If standardization becomes necessary:

1. Add OpenAPI/Swagger schema for HTTP API
2. Adopt JSON:API or similar for error responses
3. Support OAuth 2.0 for authentication
4. Add distributed session storage (Redis)
5. Implement rate limiting per OAuth client ID
6. Add metrics endpoint (Prometheus format)
