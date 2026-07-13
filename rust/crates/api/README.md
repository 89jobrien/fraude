HTTP client library for LLM providers (Fraude, Anthropic, X.AI). Abstracts
provider-specific APIs and authentication behind a unified client interface.

## Overview

Handles message requests and streaming responses from multiple LLM backends:

- Fraude API (Anthropic-compatible messages endpoint)
- X.AI Grok (OpenAI-compatible messages API)
- OpenAI (OpenAI-compatible messages API)

Provides model aliasing, automatic provider detection, and unified streaming event
types.

## Core Types

**ProviderClient** — enum over provider implementations (FraudeApi, Xai, OpenAi).
Creates a client from a model name, handles auth resolution, and dispatches
requests.

**MessageRequest** — complete message request with model, max_tokens, messages,
system prompt, tools, and streaming flag.

**MessageResponse** — non-streaming response with content blocks (text, tool use,
thinking), usage metadata, and stop reason.

**StreamEvent** — union of streaming events: MessageStart, MessageDelta,
ContentBlockStart, ContentBlockDelta, ContentBlockStop, MessageStop.

**MessageStream** — abstraction over provider-specific streams. Yields events via
`next_event()`.

**OAuthTokenSet** — OAuth2 token state (access token, refresh token, expiry).

## Usage

```rust
use api::{ProviderClient, MessageRequest, InputMessage};

// Create a client from model name (auto-detects provider and reads env auth)
let client = ProviderClient::from_model("claude-sonnet-4-6")?;

// Build a request
let request = MessageRequest {
    model: "claude-sonnet-4-6".to_string(),
    max_tokens: 1024,
    messages: vec![InputMessage::user_text("Hello")],
    system: None,
    tools: None,
    tool_choice: None,
    stream: true,
};

// Send non-streaming
let response = client.send_message(&request).await?;
println!("{}", response.total_tokens());

// Stream responses
let mut stream = client.stream_message(&request).await?;
while let Some(event) = stream.next_event().await? {
    println!("{:?}", event);
}
```

## Model Aliases

Recognized short names: opus, sonnet, haiku, grok, grok-3, grok-mini, grok-2.
Resolve with `resolve_model_alias()` or let ProviderClient handle it.

## Authentication

Reads from environment:

- `ANTHROPIC_API_KEY` + `ANTHROPIC_BASE_URL` for Fraude
- `XAI_API_KEY` + `XAI_BASE_URL` for X.AI
- `OPENAI_API_KEY` for OpenAI

Or provide explicit auth via `ProviderClient::from_model_with_default_auth()`.

## Streaming

Streaming events are SSE-encoded. The crate handles decoding via `SseParser`.
Events follow Anthropic's streaming contract (message.start → content_block.start
→ content_block.delta\* → content_block.stop → message.delta → message.stop).
