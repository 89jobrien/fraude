# Near-Term Improvements

1. **Session Cleanup**
   Add background task that periodically removes sessions older than N hours.
   Track last-accessed time; clean up idle sessions. Configurable via config.

2. **Graceful Shutdown**
   Implement shutdown signal handler (SIGTERM). Wait for in-flight turns to
   complete before closing server. Add timeout (e.g., 30s) to force quit if
   stuck.

3. **Metrics Endpoint**
   Add `/metrics` endpoint (Prometheus format) exporting: session count,
   message count, error rate, SSE subscriber count, event throughput.

4. **Broadcast Buffer Tuning**
   Currently hardcoded 64-event buffer. Add config option for capacity.
   Emit warning if events dropped due to slow subscriber.

5. **Session ID Obfuscation**
   Current ID format `session-1`, `session-2` is predictable. Use random
   UUID or nanoid for unpredictable session IDs. Add opt-in via config.
