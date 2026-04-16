# LlmProvider Trait & Client Unification — Behavioral Test Results

**Agent**: mind-model-router
**Date**: 2026-04-16
**Ollama**: Running locally, model used: `phi3:mini` (fastest available, 3.8B Q4_0)
**Test file**: `src/codex-llm/tests/llm_provider_behavioral.rs`
**Run command**: `cargo test -p codex-llm --test llm_provider_behavioral -- --nocapture`

---

## Test Results: 5/5 PASS

| # | Test | Result | Time | Details |
|---|------|--------|------|---------|
| 1 | OllamaClient implements LlmProvider | **PASS** | 13.8s | Created OllamaClient, cast to `Box<dyn LlmProvider>`, called `chat()` with system+user messages. Got real response: "Hello!" |
| 2 | Mind uses Box<dyn LlmProvider> | **PASS** | ~14s | Created provider same way Mind::new() does (line 50-53), called `simple_chat()` mirroring Mind.think() (line 127). Got real response. |
| 3 | simple_chat convenience method | **PASS** | 13.8s | Used default `simple_chat()` from trait, sent "Reply with only the number 42", got back "42". Verified returns clean String, not JSON. |
| 4 | qwen-mind is just re-exports | **PASS** | 0ms | Compile-time verification. `cargo check -p qwen-mind` passes. `qwen-mind/src/llm.rs` is 10 lines of `pub use codex_llm::*` re-exports. No separate OllamaClient. |
| 5 | Rate limiter works through trait | **PASS** | 14.5s | Attached RateLimiter via `.with_rate_limiter()`, cast to trait, sent 3 requests. Rate limiter tracked all 3: 72 tokens in, 74 tokens out, $0.0001 est. cost. JSONL metrics file had 3 records. Circuit breaker stayed closed. |

**Total test suite time**: 14.87s (tests run in parallel, Ollama inference dominates)

---

## Blocker Analysis

### Can we test against Ollama Cloud models or only local?

**Both work.** `OllamaConfig::cloud(model, api_key)` sets base_url to `https://api.ollama.com` and includes Bearer token auth. The `LlmProvider` trait makes this transparent — same `chat()` call works for local and cloud. The only difference is `config.api_key` being `Some(key)` vs `None`.

**Current cloud models available**: `qwen3.5:cloud` (397B), `minimax-m2.7:cloud`, `gemma4:31b-cloud`

**Recommendation**: Add an `#[ignore]` integration test that hits Ollama Cloud (requires `OLLAMA_API_KEY` env var). Already gated by the `ollama_available()` check pattern.

### What happens when Ollama is down? Does the circuit breaker work through the trait?

**Yes, with a nuance.** The circuit breaker is inside `RateLimiter`, which is attached to `OllamaClient` (not to the trait). When Ollama is completely down:

1. **Connection refused**: OllamaClient's retry policy handles this — 3 retries with exponential backoff (1s, 2s, 4s), then returns `LlmError::Http(...)`. The rate limiter records the failure.
2. **HTTP 429/500/502/503/504**: Rate limiter increments consecutive error counter. After 5 consecutive errors, circuit breaker trips (60s cooldown). During cooldown, `check()` returns `Err(secs_remaining)` and OllamaClient fails fast without making HTTP requests.
3. **Through the trait**: The circuit breaker fires BEFORE the HTTP call inside `OllamaClient::chat()`, so callers using `Box<dyn LlmProvider>` get the protection transparently. No leak in the abstraction.

**Gap**: The `check()` is called inside OllamaClient's inherent `chat()` method. If someone implements a new `LlmProvider` (e.g., OpenAIClient), they'd need their own rate limiter integration — the trait doesn't enforce it. Consider adding an optional `rate_limiter()` method to the trait.

### What's needed to add an OpenAI-compatible provider?

**Moderate effort (~200 lines)**. The pieces needed:

1. **New struct** `OpenAiClient` implementing `LlmProvider` — needs `chat()` hitting `/v1/chat/completions`
2. **Response mapping**: OpenAI responses are already in the format codex-llm uses internally (`ChatResponse`, `ChatMessage`). The current types were designed for OpenAI compat, so the mapping would be nearly 1:1.
3. **Config**: API key, base URL, model name. Could reuse `OllamaConfig` or create `OpenAiConfig`.
4. **Rate limiter**: Attach via same `.with_rate_limiter()` pattern, or use OpenAI's `X-RateLimit-*` headers.
5. **Registration**: Add to `ModelRouter` as a new provider variant.

**The trait abstraction is ready** — `Box<dyn LlmProvider>` means Mind, ThinkLoop (once migrated), and all callers get the new provider for free.

### Can ThinkLoop accept Box<dyn LlmProvider> yet or does it still need concrete OllamaClient?

**NO — ThinkLoop still requires concrete OllamaClient.**

```rust
// think_loop.rs line ~85
pub struct ThinkLoop {
    client: OllamaClient,  // NOT Box<dyn LlmProvider>
    // ...
}
```

ThinkLoop constructs its own `OllamaClient::new(config.ollama.clone())` in `ThinkLoop::new()`. It calls `self.client.chat()` directly (the inherent method, not via trait dispatch).

**Migration path**:
1. Change `client: OllamaClient` → `client: Box<dyn LlmProvider>` in ThinkLoop struct
2. Accept `Box<dyn LlmProvider>` in ThinkLoop::new() instead of constructing internally
3. The `with_rate_limiter()` pattern would need to move to the caller (rate limiter is currently on OllamaClient, not the trait)
4. Tool schema conversion (`OllamaClient::tool_schemas()`) is a static method — move to a standalone function or the trait

**This is the NEXT unification target.** Mind is unified (uses trait). ThinkLoop is not yet. This means Cortex's main execution loop is still hardcoded to Ollama's native API.

---

## Architecture Observations

### Two Reasoning Paths (confirmed by tests)

| Path | Where | LLM Access | Complexity |
|------|-------|-----------|------------|
| `Mind.think()` | qwen-mind | `Box<dyn LlmProvider>` (trait) | Single LLM call via `simple_chat()` — no tool loop |
| `ThinkLoop.run_full()` | codex-llm | `OllamaClient` (concrete) | Full tool-calling loop with interceptors, memory, challenger |

The trait unification is **complete for Mind** and **incomplete for ThinkLoop**.

### Client Unification Status

| Component | Status | Evidence |
|-----------|--------|----------|
| `LlmProvider` trait | **DONE** | `provider.rs` — clean trait with `chat()`, `model_name()`, `simple_chat()` |
| `OllamaClient` impl | **DONE** | `ollama.rs:542-555` — delegates to inherent `chat()` |
| `Mind.llm` field | **DONE** | `mind.rs:30` — `Box<dyn LlmProvider>` |
| `qwen-mind/llm.rs` | **DONE** | 10-line re-export, no duplicate client |
| `ThinkLoop.client` | **NOT DONE** | Still `OllamaClient` concrete type |
| Rate limiter through trait | **WORKS** | Tested — RateLimiter tracks through `Box<dyn LlmProvider>` |
| Multiple providers | **NOT YET** | Only OllamaClient implements the trait |

### Recommended Next Steps

1. **Migrate ThinkLoop to `Box<dyn LlmProvider>`** — this is the critical remaining gap
2. **Add MockLlmProvider** for unit testing — enables testing ThinkLoop without Ollama
3. **Add OpenAI-compatible provider** — validates the trait is truly model-agnostic
4. **Consider `rate_limiter()` on the trait** — so all providers can expose circuit breaker state
