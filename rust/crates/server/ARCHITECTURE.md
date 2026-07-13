## Module Breakdown

### lib.rs

Single-file module:

- `AppState`: Shared state (sessions map + ID allocator)
- `Session`: In-memory session representation
- `SessionEvent`: Enum of broadcast event types
- Handlers: `create_session`, `list_sessions`, `get_session`,
  `send_message`, `stream_session_events`
- Route setup: `app(state) -> Router`
- Error utilities: `ApiError`, `ApiResult`, error response builder
- Constants: `BROADCAST_CAPACITY` (64 events per channel)

## Data Flow

### Session Creation

```
POST /sessions (empty body)
  ↓
create_session(state)
  ↓
Allocate ID (atomic increment)
  ↓
Create Session {
  id,
  created_at: now_unix_ms(),
  conversation: RuntimeSession::new(),
  events: broadcast::channel()
}
  ↓
Store in state.sessions (RwLock)
  ↓
Response: CreateSessionResponse { session_id }
```

### Message Send and Turn Execution

```
POST /sessions/{id}/message { "message": "..." }
  ↓
send_message(state, id, request)
  ↓
Lock sessions (read)
  ↓
Lookup session (or 404)
  ↓
Append UserMessage to session.conversation
  ↓
Spawn tokio::spawn(async {
    runtime.run_turn(message, model)
      ↓
    Stream AssistantEvent:
      - Text(delta)
      - ToolCall(...)
      - ToolResult(...)
      - Metadata(...)
    ↓
    For each event:
      route to session.events.send(SessionEvent::...)
  })
  ↓
Response: 202 Accepted (immediately)
  ↓
Background task continues
```

### Event Streaming

```
GET /sessions/{id}/events
  ↓
stream_session_events(state, id)
  ↓
Lookup session (or 404)
  ↓
Subscribe to session.events broadcast
  ↓
async_stream::stream! {
  loop {
    Select (broadcast recv or client close):
      recv → convert to SSE Event, yield
      close → break
  }
}
  ↓
Axum wraps stream as SSE response
  ↓
Client receives Server-Sent Events (chunked HTTP 200)
```

### Session Listing

```
GET /sessions
  ↓
list_sessions(state)
  ↓
Lock sessions (read)
  ↓
Map sessions → SessionSummary {
  id,
  created_at,
  message_count: session.conversation.messages.len()
}
  ↓
Sort by ID
  ↓
Response: ListSessionsResponse { sessions }
```

### Session Retrieval

```
GET /sessions/{id}
  ↓
get_session(state, id)
  ↓
Lock sessions (read)
  ↓
Lookup session (or 404)
  ↓
Clone session.conversation (RuntimeSession)
  ↓
Response: SessionDetailsResponse {
  id,
  created_at,
  session: RuntimeSession
}
```

## Concurrency Model

### Read-Write Lock

`AppState.sessions` protected by `Arc<RwLock<HashMap>>`:

- Multiple concurrent readers (GET /sessions, GET /sessions/{id})
- Single writer (POST /sessions, POST /sessions/{id}/message)
- No deadlock risk (always acquire, then release; no nested locks)

RwLock chosen to allow concurrent reads; Mutex would serialize all accesses.

### Broadcast Channel

Each session has independent `broadcast::Sender`:

- Sender cloned to runtime task
- Multiple receivers (SSE subscribers)
- Non-blocking send (dropping old events if buffer full)
- Subscribers see events in order

No coordination between sessions.

### Session ID Allocation

`AtomicU64` with `fetch_add()` lock-free allocation:

- Monotonically increasing IDs
- No duplicate IDs (atomic operation)
- Format: `session-{id}` for readability

## Design Decisions

1. **Stateless Server**: Sessions stored in-memory only. No persistence
   backend. Enables simple single-process deployment; trades off durability.

2. **Broadcast Channel per Session**: Each session independently publishes
   events. Enables multiple simultaneous subscribers; decouples turn
   execution from event consumption.

3. **RwLock for Sessions Map**: Prefer read-write lock over Mutex to allow
   concurrent GET operations while serializing writes. Most ops are reads
   (list, retrieve, stream).

4. **Async Streaming**: Turn execution spawned in background task, not
   blocking HTTP response. Client receives 202, then subscribes to /events
   stream for updates.

5. **Event Conversion in Handler**: `AssistantEvent` from runtime converted
   to `SessionEvent` in handler layer. Keeps runtime and server layers
   decoupled.

6. **JSON Serialization**: All events serialized to JSON for SSE. Runtime
   types don't know about HTTP format.

7. **No Session Timeout**: Sessions kept indefinitely. Cleanup left to caller
   (not implemented; could add background task).

8. **No Authentication**: No built-in auth/authz. Deploy behind proxy with
   auth layer (nginx, API gateway, etc.).

## Known Limitations

- **In-Memory Only**: Sessions lost on server restart
- **Single Process**: Load balancing requires session affinity
- **No Cleanup**: Sessions accumulate in memory over time
- **No Rate Limiting**: Clients can spam requests
- **No Authentication**: Anyone can access any session
- **Broadcast Buffer**: Fixed capacity (64 events); oldest discarded if full
  (clients may miss events if slow)
- **No Graceful Shutdown**: Active turns interrupted on server stop

## Future Enhancements

1. **Persistent Store**: Move sessions to Redis/PostgreSQL
2. **Session Timeout**: Periodically clean up old sessions
3. **Authentication Layer**: Integrate OAuth or API key auth
4. **Rate Limiting**: Per-session or per-IP limits
5. **Graceful Shutdown**: Complete in-flight turns before stopping
6. **Metrics**: Expose session count, events/sec, error rate
7. **Multi-Instance**: Use distributed broadcast (Redis Streams, Kafka)
