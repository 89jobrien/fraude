Axum-based HTTP REST API for remote Fraude agent sessions. Exposes endpoints
to create sessions, retrieve session state, send messages, and stream turn
events via server-sent events (SSE).

## Features

- **Session Management**: Create, list, and retrieve sessions with unique IDs
- **Streaming Events**: Server-sent events for real-time turn updates
- **Event Types**: Text chunks, tool calls, tool results, metadata
- **Session Snapshots**: Full conversation state available on-demand
- **Concurrent Sessions**: In-memory store for multiple parallel sessions
- **Graceful Error Handling**: Consistent JSON error responses
- **Async/Await**: Tokio-based async I/O; non-blocking request handling

## Endpoints

### POST /sessions

Create a new session. Returns session ID.

Request: (empty body)

Response:

```json
{
  "session_id": "session-1"
}
```

### GET /sessions

List all active sessions.

Response:

```json
{
  "sessions": [
    {
      "id": "session-1",
      "created_at": 1234567890000,
      "message_count": 5
    }
  ]
}
```

### GET /sessions/{id}

Retrieve full session state (all messages).

Response:

```json
{
  "id": "session-1",
  "created_at": 1234567890000,
  "session": {
    "messages": [...]
  }
}
```

### POST /sessions/{id}/message

Send a message and trigger a turn. Returns no response body immediately;
client should subscribe to events via /sessions/{id}/events.

Request:

```json
{
  "message": "What's 2+2?"
}
```

### GET /sessions/{id}/events

Stream turn events via SSE. Client opens connection; server sends events as
they occur.

Event types:

- `text`: Assistant response text chunk

  ```json
  {
    "type": "text",
    "delta": "4"
  }
  ```

- `tool_call`: Tool invocation

  ```json
  {
    "type": "tool_call",
    "tool": "bash",
    "input": {...}
  }
  ```

- `tool_result`: Tool result

  ```json
  {
    "type": "tool_result",
    "tool": "bash",
    "output": "..."
  }
  ```

- `metadata`: Turn metadata

  ```json
  {
    "type": "metadata",
    "tokens": {
      "input": 100,
      "output": 50
    },
    "stop_reason": "end_turn"
  }
  ```

- `snapshot`: Full conversation state
  ```json
  {
    "type": "snapshot",
    "session": {...}
  }
  ```

## Architecture

### AppState

Shared application state (Arc-wrapped):

```rust
pub struct AppState {
    sessions: Arc<RwLock<HashMap<SessionId, Session>>>,
    next_session_id: Arc<AtomicU64>,
}
```

All requests receive `State(AppState)` via dependency injection.

### Session

In-memory session representation:

```rust
pub struct Session {
    pub id: SessionId,
    pub created_at: u64,
    pub conversation: RuntimeSession,
    events: broadcast::Sender<SessionEvent>,
}
```

- `RuntimeSession`: Conversation history (from runtime crate)
- `events`: Broadcast channel for streaming updates
- `created_at`: Unix timestamp (milliseconds)

### Event Broadcasting

Each session has a broadcast channel. When `run_turn()` completes:

1. Runtime emits `AssistantEvent::Text`, `::ToolCall`, etc.
2. Server routes to broadcast sender
3. All subscribed clients receive events
4. Conversion to JSON via serde_json

### Handler Flow

#### Create Session

1. Allocate unique session ID (atomic counter)
2. Create new `Session` with empty conversation
3. Store in `AppState.sessions` map
4. Return session ID

#### Stream Events

1. Look up session by ID (or 404)
2. Get broadcast receiver (subscriber)
3. Return SSE stream via `async_stream::stream`
4. For each event in broadcast channel: format as SSE, send to client
5. Connection closes when client disconnects

#### Send Message

1. Look up session by ID (or 404)
2. Append user message to conversation
3. Spawn task to run turn (don't block response)
4. Respond immediately (202 Accepted or similar)
5. Turn events published to broadcast channel
6. Clients subscribed to /events receive updates

### Error Handling

All endpoints return JSON error responses:

```json
{
  "error": "session `session-1` not found"
}
```

HTTP status codes:

- 200: Success
- 201: Created (POST /sessions)
- 202: Accepted (POST /sessions/{id}/message)
- 400: Bad request (invalid JSON, missing fields)
- 404: Not found (session doesn't exist)
- 500: Internal error (runtime error, channel error)

### SSE Format

Standard Server-Sent Events (RFC 8664):

```
event: text
data: {"delta": "Hello"}

event: tool_call
data: {"tool": "bash", "input": {...}}

event: metadata
data: {"tokens": {"input": 100, "output": 50}}
```

Clients parse with `EventSource` API (JavaScript) or equivalent in other
languages.

## Public API

```rust
pub fn app(state: AppState) -> Router

pub struct AppState { /* ... */ }
impl AppState {
    pub fn new() -> Self
    pub fn default() -> Self
}

pub struct Session {
    pub id: SessionId,
    pub created_at: u64,
    pub conversation: RuntimeSession,
}

pub struct CreateSessionResponse {
    pub session_id: SessionId,
}

pub struct ListSessionsResponse {
    pub sessions: Vec<SessionSummary>,
}

pub struct SessionDetailsResponse {
    pub id: SessionId,
    pub created_at: u64,
    pub session: RuntimeSession,
}
```

## Deployment

Server is stateless except for in-memory session store. Sessions lost on
restart.

To persist sessions:

1. Replace `HashMap` with persistent store (Redis, PostgreSQL, etc.)
2. On startup, reload sessions from store
3. On create/update/delete, update persistent store

For multi-instance deployment:

1. Use shared persistent store (Redis hash, PostgreSQL table)
2. Use sticky sessions or broadcast channel replication across instances
3. Implement session locking to prevent concurrent modifications

## Dependencies

- `axum`: Web framework
- `tokio`: Async runtime
- `serde_json`: JSON serialization
- `runtime`: Fraude runtime (conversation, turn execution)
- `async_stream`: Stream macros for SSE generation

## Testing

Tests cover:

- Session creation (unique IDs, stored correctly)
- Session listing and retrieval
- Message sending (appended to conversation)
- Event streaming (correct JSON format, event ordering)
- Error cases (404, invalid JSON, missing fields)
