# TODO

## Near-Term Improvements

1. **Provider-Specific Error Details** — Currently ApiError is generic. Add
   discriminated variants for provider-specific errors (auth failures, rate
   limits, malformed responses) to help callers implement retry logic and user
   feedback.

2. **Response Validation** — Validate MessageResponse before returning (e.g.,
   content blocks are non-empty, usage values are sensible). Surface validation
   errors instead of allowing garbage data to propagate.

3. **Connection Pooling Configuration** — Expose reqwest client pool size,
   timeout, and retry strategy as builder methods on ProviderClient instead of
   hard-coding defaults.

4. **Streaming Backpressure** — Add buffering and flow control to MessageStream.
   Currently callers must consume events in real-time or buffer client-side.

5. **Tool Definition Validation** — Validate tool input_schema (JSON Schema) at
   request time rather than delegating to the provider, catching malformed
   schemas before sending.
