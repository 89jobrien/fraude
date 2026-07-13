# Conformance

## Standards

This crate conforms to:

- **Anthropic Messages API** (https://docs.anthropic.com/messages/overview) —
  message request/response format, streaming protocol (server-sent events),
  content block types (text, tool_use, thinking)
- **OpenAI Chat Completions API** (https://platform.openai.com/docs/api-reference)
  — compatible message format used by X.AI Grok and OpenAI
- **SSE (Server-Sent Events)** spec for streaming

## Known Deviations

**Tool Choice** — OpenAI-compatible providers map Anthropic's ToolChoice enum
(auto, any, tool{name}) to OpenAI's tool_choice field. Not all providers support
"any".

**Thinking Content** — Anthropic's thinking blocks are transparently passed
through. OpenAI and X.AI support thinking via content_block.type="thinking".

**Stop Reason** — Anthropic uses "end_turn", "max_tokens", "stop_sequence",
"tool_use". OpenAI uses "stop", "length", "tool_calls". The crate normalizes to
Anthropic's values in MessageResponse.stop_reason.

**Request ID** — Only Fraude API and some OpenAI-compatible servers return a
request ID. MessageResponse.request_id is Option.

**Cache Tokens** — Only Anthropic supports cache_creation_input_tokens and
cache_read_input_tokens. These are included in Usage; defaults are 0 for other
providers.
