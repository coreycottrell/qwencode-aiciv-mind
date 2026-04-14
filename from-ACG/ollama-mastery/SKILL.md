---
name: ollama-mastery
description: Comprehensive Ollama reference — API endpoints, cloud, tool calling, authentication, streaming, Modelfile, client libraries, error handling
type: reference
---

# Ollama Mastery Skill

**Version**: 1.0
**Last Updated**: 2026-04-04
**Scope**: Complete Ollama platform reference for AI agents

Load this skill to become an instant Ollama expert. Covers all major APIs, cloud features, local deployment, tool calling, streaming, authentication, and best practices.

---

## QUICK REFERENCE

| Need | Location | Command/URL |
|------|----------|-------------|
| **Local API** | This machine | `http://localhost:11434/api` |
| **Cloud API** | Ollama Cloud | `https://ollama.com/api` (+ `OLLAMA_API_KEY`) |
| **OpenAI compat** | Either | `/v1/chat/completions` (identical to OpenAI) |
| **Pull model** | CLI | `ollama pull minimax-m2.7` |
| **Pull cloud** | CLI | `ollama pull minimax-m2.7:cloud` |
| **Run model** | CLI | `ollama run minimax-m2.7 "your prompt"` |
| **Auth (local)** | N/A | None required |
| **Auth (cloud)** | Env | `export OLLAMA_API_KEY="your-key"` |
| **Create API key** | Web | `ollama.com/settings/keys` |
| **Test API** | CLI | `curl http://localhost:11434/api/tags` |

---

## PART 1: API ARCHITECTURE

### 1.1 Three Ways to Talk to Ollama

#### **Native Ollama API** (`/api/*`)
```bash
curl http://localhost:11434/api/chat -d '{
  "model": "minimax-m2.7",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}'
```
- **Full feature set**: All Ollama capabilities
- **Request format**: Native Ollama JSON schema
- **Cloud support**: Yes, use `https://ollama.com/api` + API key

#### **OpenAI-Compatible** (`/v1/*`)
```bash
curl http://localhost:11434/v1/chat/completions -d '{
  "model": "minimax-m2.7",
  "messages": [{"role": "user", "content": "Hello"}]
}'
```
- **Request/response**: Identical to OpenAI's API
- **Use standard OpenAI SDK**: No modification needed
- **No API key required** (local), or use `OLLAMA_API_KEY` (cloud)

#### **via Python/JavaScript SDKs**
```python
from ollama import AsyncClient
client = AsyncClient()
response = await client.chat(model="minimax-m2.7", messages=[...])
```

### 1.2 Local vs Cloud Architecture

**Local** (`http://localhost:11434`)
- Models run on your machine
- Offline capability (no internet)
- Privacy (data stays behind firewall)
- Free (open-source)
- Best for: Development, sensitive data, offline use

**Cloud** (`https://ollama.com`)
- Models run on Ollama infrastructure
- No local GPU needed
- Managed scaling
- Subscription required
- Best for: Large models, high throughput, production

**Port Details**
```
Default: 11434 (can be overridden via OLLAMA_HOST env var)
Docker Linux: Use 172.17.0.1:11434 (NOT localhost)
```

---

## PART 2: AUTHENTICATION

### 2.1 Local Access (No Auth)
```bash
# Just works — no credentials needed
curl http://localhost:11434/api/tags
```

### 2.2 Cloud Access (Bearer Token)

**Step 1: Create API Key**
```bash
# Via web: ollama.com/settings/keys
# Or use ollama signin
ollama signin
```

**Step 2: Set Environment Variable**
```bash
export OLLAMA_API_KEY="sk-ollama-abc123def456..."
```

**Step 3: Use in Requests**
```bash
# Curl
curl https://ollama.com/api/chat \
  -H "Authorization: Bearer $OLLAMA_API_KEY" \
  -d '{"model": "minimax-m2.7", ...}'

# Python
import os
client = OpenAI(
    api_key=os.environ["OLLAMA_API_KEY"],
    base_url="https://ollama.com/v1"
)
```

**Key Details**
- API keys **don't expire** but can be revoked anytime
- Create keys at: `ollama.com/settings/keys`
- Can have multiple active keys
- Format: Bearer token in `Authorization` header

---

## PART 3: CORE INFERENCE ENDPOINTS

### 3.1 `/api/chat` — Multi-Turn Conversation

**Best for**: Chat applications, conversational AI, agent interactions

```bash
curl http://localhost:11434/api/chat -d '{
  "model": "minimax-m2.7",
  "messages": [
    {"role": "system", "content": "You are a helpful assistant"},
    {"role": "user", "content": "What is machine learning?"},
    {"role": "assistant", "content": "Machine learning is..."},
    {"role": "user", "content": "Tell me more about neural networks"}
  ],
  "stream": false,
  "temperature": 0.7,
  "num_ctx": 8192,
  "num_predict": 2048
}'
```

**Response (non-streaming)**:
```json
{
  "model": "minimax-m2.7",
  "created_at": "2026-04-04T12:00:00Z",
  "message": {
    "role": "assistant",
    "content": "Neural networks are computational models inspired by..."
  },
  "done": true,
  "done_reason": "stop",
  "eval_count": 45,
  "eval_duration": 987654321,
  "prompt_eval_count": 28,
  "load_duration": 123456
}
```

### 3.2 `/api/generate` — Simple Text Completion

**Best for**: Simple prompts, text generation, non-conversational tasks

```bash
curl http://localhost:11434/api/generate -d '{
  "model": "minimax-m2.7",
  "prompt": "The capital of France is",
  "stream": false
}'
```

**Key Difference from `/api/chat`**
- Takes simple `prompt` string (not message history)
- No message roles (system/user/assistant)
- Useful for: Completions, creative writing, structured output

### 3.3 `/api/embeddings` — Vector Embeddings

**Best for**: Semantic search, RAG (Retrieval-Augmented Generation), similarity matching

```bash
curl http://localhost:11434/api/embeddings -d '{
  "model": "nomic-embed-text",
  "prompt": "Ollama is an open-source LLM platform"
}'
```

**Response**:
```json
{
  "embedding": [0.123, -0.456, 0.789, ...],
  "done": true
}
```

---

## PART 4: MODEL MANAGEMENT ENDPOINTS

### 4.1 Core Model Operations

| Endpoint | Method | Purpose | Example |
|----------|--------|---------|---------|
| `/api/tags` | GET | List all local models | `curl http://localhost:11434/api/tags` |
| `/api/show` | POST | Get model details | `curl -d '{"name":"minimax-m2.7"}'` |
| `/api/pull` | POST | Download model | `curl -d '{"name":"minimax-m2.7"}'` |
| `/api/delete` | DELETE | Remove model | `curl -d '{"name":"minimax-m2.7"}'` |
| `/api/copy` | POST | Alias/copy model | `curl -d '{"source":"m2.7","destination":"m2.7-custom"}'` |
| `/api/create` | POST | Create from Modelfile | `curl -d '{"name":"custom","from":"minimax-m2.7"}'` |
| `/api/push` | POST | Upload to registry | Requires authentication + registration |

### 4.2 `/api/tags` — List Models

```bash
curl http://localhost:11434/api/tags
```

**Response**:
```json
{
  "models": [
    {
      "name": "minimax-m2.7:latest",
      "modified_at": "2026-04-04T10:00:00Z",
      "size": 45000000000,
      "digest": "sha256:abc123...",
      "details": {
        "format": "gguf",
        "family": "minimax",
        "parameter_size": "7B",
        "quantization_level": "Q4_0"
      }
    }
  ]
}
```

### 4.3 `/api/show` — Model Details

```bash
curl http://localhost:11434/api/show -d '{
  "name": "minimax-m2.7"
}'
```

Returns detailed information including: Modelfile content, parameters, templates, system prompts.

### 4.4 `/api/pull` — Download Model

```bash
curl http://localhost:11434/api/pull -d '{
  "name": "minimax-m2.7"
}'
```

**Streaming Response** (NDJSON):
```json
{"status": "pulling manifest"}
{"status": "downloading abc123...", "digest": "sha256:abc123...", "total": 1000000, "completed": 500000}
{"status": "verifying sha256 digest"}
{"status": "writing manifest"}
{"status": "removing unused layers"}
{"status": "success"}
```

### 4.5 `/api/copy` — Alias/Clone Model

**Important**: No data duplication — creates new manifest reference only

```bash
curl http://localhost:11434/api/copy -d '{
  "source": "minimax-m2.7",
  "destination": "minimax-m2.7-custom"
}'
```

Returns HTTP 200 with empty body on success.

### 4.6 `/api/delete` — Remove Model

```bash
curl -X DELETE http://localhost:11434/api/delete -d '{
  "name": "minimax-m2.7"
}'
```

**Garbage Collection**: Removes model; only deletes blobs unused by other models.

---

## PART 5: STREAMING VS NON-STREAMING

### 5.1 Streaming (Default)

**Enable**: `"stream": true` (default for inference endpoints)

```bash
curl http://localhost:11434/api/chat -d '{
  "model": "minimax-m2.7",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": true
}'
```

**Response Format**: Newline-Delimited JSON (NDJSON)

```
{"message":{"role":"assistant","content":"Hello"},"done":false}
{"message":{"role":"assistant","content":" world"},"done":false}
{"message":{"role":"assistant","content":"!"},"done":true,"done_reason":"stop","eval_count":5}
```

**Content-Type**: `application/x-ndjson`

**Benefits**:
- Real-time output (progressive display)
- Better UX for long responses
- Can stop early if needed
- Memory efficient

**Python Implementation**:
```python
from ollama import AsyncClient

client = AsyncClient()
async for chunk in await client.chat(
    model="minimax-m2.7",
    messages=[...],
    stream=True
):
    print(chunk['message']['content'], end='', flush=True)
```

### 5.2 Non-Streaming

**Disable**: `"stream": false`

```bash
curl http://localhost:11434/api/chat -d '{
  "model": "minimax-m2.7",
  "messages": [{"role": "user", "content": "Hello"}],
  "stream": false
}'
```

**Response Format**: Single JSON object

```json
{
  "model": "minimax-m2.7",
  "created_at": "2026-04-04T12:00:00Z",
  "message": {
    "role": "assistant",
    "content": "Hello world!"
  },
  "done": true,
  "done_reason": "stop",
  "eval_count": 5,
  "eval_duration": 987654321,
  "prompt_eval_count": 3,
  "load_duration": 123456
}
```

**Benefits**:
- Simpler code (single response)
- Complete response available immediately
- Easier error handling
- Better for batch operations

**Python Implementation**:
```python
from ollama import Client

client = Client()
response = client.chat(
    model="minimax-m2.7",
    messages=[...],
    stream=False
)
print(response['message']['content'])
```

### 5.3 Key Difference Summary

| Aspect | Streaming | Non-Streaming |
|--------|-----------|---------------|
| **Response** | Multiple NDJSON objects | Single JSON object |
| **Start time** | Immediate (first chunk) | After full completion |
| **Memory** | Low (per-chunk) | High (full response) |
| **Error handling** | Mid-stream detection | After completion |
| **UX** | Better (progressive) | Simpler code |

---

## PART 6: TOOL/FUNCTION CALLING

### 6.1 What Tool Calling Does

Model receives list of available tools, decides which to call, returns structured invocations. Enables agents, external function execution, and complex multi-step reasoning.

### 6.2 Three Calling Patterns

#### **Single-Shot**
```
User prompt + tool definition
→ Model decides to call tool
→ Provide tool result in next message
→ Model generates final response
```

#### **Parallel**
```
User prompt + multiple tool definitions
→ Model calls multiple tools
→ Gather all results
→ Pass all results together in next message
→ Model generates response
```

#### **Agent Loop (Multi-Turn)**
```
Loop:
  Model processes messages
  Decides to call tool(s)
  Calls multiple tools in parallel
  You provide results in follow-up message
Until: Model stops requesting tools
```

### 6.3 Request Format

```json
{
  "model": "qwen3",
  "messages": [
    {
      "role": "user",
      "content": "What's the weather in San Francisco and New York?"
    }
  ],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather for a location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name"
            },
            "unit": {
              "type": "string",
              "enum": ["celsius", "fahrenheit"],
              "description": "Temperature unit"
            }
          },
          "required": ["location"]
        }
      }
    }
  ]
}
```

### 6.4 Response with Tool Calls

```json
{
  "model": "qwen3",
  "created_at": "2026-04-04T12:00:00Z",
  "message": {
    "role": "assistant",
    "content": "I'll check the weather for both cities.",
    "tool_calls": [
      {
        "function": {
          "name": "get_weather",
          "arguments": {
            "location": "San Francisco",
            "unit": "fahrenheit"
          }
        }
      },
      {
        "function": {
          "name": "get_weather",
          "arguments": {
            "location": "New York",
            "unit": "fahrenheit"
          }
        }
      }
    ]
  },
  "done": true
}
```

### 6.5 Follow-Up: Provide Tool Results

```json
{
  "model": "qwen3",
  "messages": [
    {
      "role": "user",
      "content": "What's the weather in San Francisco and New York?"
    },
    {
      "role": "assistant",
      "content": "I'll check the weather for both cities.",
      "tool_calls": [...]
    },
    {
      "role": "tool",
      "content": "San Francisco: 68°F, partly cloudy",
      "name": "get_weather"
    },
    {
      "role": "tool",
      "content": "New York: 52°F, rainy",
      "name": "get_weather"
    }
  ]
}
```

### 6.6 Supported Models

**Verified Tool Calling Support**:
- **qwen3** (best documented, full support)
- **Llama 3.1** (especially 8B-Instruct)
- **Mistral Nemo** (7B variant, resource-efficient)
- **FunctionGemma** (specialized for tool use)
- **Firefunction v2**
- **Command-R+**

### 6.7 Streaming Tool Calls

When `"stream": true`, tool calls come incrementally:

```json
{"message":{"content":"I'll check"},"done":false}
{"message":{"tool_calls":[{"function":{"name":"get_weather"}}]},"done":false}
{"message":{"tool_calls":[...]},"done":true}
```

**Critical**: Accumulate chunks until `"done": true`, then process complete tool_calls with results.

### 6.8 Python SDK Convenience

```python
from ollama import AsyncClient

client = AsyncClient()

# Define actual Python function
def get_weather(location: str, unit: str = "fahrenheit") -> str:
    """Get weather for a location."""
    # Your implementation
    return f"{location}: 68°F, sunny"

# Pass function directly — SDK auto-converts to JSON schema
response = await client.chat(
    model="qwen3",
    messages=[{"role": "user", "content": "What's the weather?"}],
    tools=[get_weather]  # Regular Python function
)
```

---

## PART 7: OPENAI-COMPATIBLE ENDPOINTS

### 7.1 Endpoint Mapping

| OpenAI Endpoint | Ollama | Identical? |
|-----------------|--------|-----------|
| `/v1/chat/completions` | `/v1/chat/completions` | ✅ Yes |
| `/v1/completions` | `/v1/completions` | ✅ Yes |
| `/v1/embeddings` | `/v1/embeddings` | ✅ Yes |
| `/v1/models` | `/v1/models` | ✅ Yes |

### 7.2 Using Standard OpenAI SDK

```python
from openai import OpenAI

# Point to local Ollama
client = OpenAI(
    api_key="anything-or-empty",  # Not validated locally
    base_url="http://localhost:11434/v1"
)

# Use exactly like OpenAI API
response = client.chat.completions.create(
    model="minimax-m2.7",
    messages=[
        {"role": "system", "content": "You are helpful"},
        {"role": "user", "content": "Hello"}
    ],
    temperature=0.7,
    stream=False
)

print(response.choices[0].message.content)
```

### 7.3 Cloud Usage (OpenAI SDK)

```python
from openai import OpenAI
import os

client = OpenAI(
    api_key=os.environ["OLLAMA_API_KEY"],
    base_url="https://ollama.com/v1"
)

# Everything else identical
response = client.chat.completions.create(
    model="minimax-m2.7",
    messages=[...]
)
```

### 7.4 Request/Response Compatibility

Request format: **Identical to OpenAI**
```json
{
  "model": "minimax-m2.7",
  "messages": [...],
  "temperature": 0.7,
  "max_tokens": 2048,
  "stream": false
}
```

Response format: **Identical to OpenAI**
```json
{
  "id": "chatcmpl-...",
  "object": "chat.completion",
  "created": 1680000000,
  "model": "minimax-m2.7",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "..."
      },
      "finish_reason": "stop"
    }
  ],
  "usage": {
    "prompt_tokens": 10,
    "completion_tokens": 50,
    "total_tokens": 60
  }
}
```

---

## PART 8: OLLAMA CLOUD SPECIFICS

### 8.1 What is Ollama Cloud?

Cloud models are models that automatically offload to Ollama's infrastructure, eliminating need for local GPU while maintaining same API interface.

**Key Benefit**: Run `minimax-m2.7` on any machine, even a Raspberry Pi, by offloading to cloud.

### 8.2 Two Cloud Access Methods

#### **Method 1: Local Integration** (via `ollama signin`)
```bash
ollama signin
ollama pull minimax-m2.7:cloud
ollama run minimax-m2.7:cloud "Your prompt"
```
Automatic authorization, no additional setup.

#### **Method 2: Direct API Access** (via API key)
```bash
export OLLAMA_API_KEY="sk-ollama-..."
curl -H "Authorization: Bearer $OLLAMA_API_KEY" \
  https://ollama.com/api/chat -d '{
    "model": "minimax-m2.7:cloud",
    "messages": [...]
  }'
```

### 8.3 Cloud Pricing Tiers

| Plan | Price/Month | Concurrent Models | Usage Limits |
|------|-------------|------------------|--------------|
| Free | $0 | 1 | Generous (individuals) |
| Pro | $20 | 3 | Expanded |
| Max | $100 | 10 | Premium |

### 8.4 Usage Limits

- **Session limit**: Resets every 5 hours
- **Weekly limit**: Resets every 7 days
- **Reflection**: Actual GPU time consumed on Ollama infrastructure

### 8.5 Data Privacy

- **No training on user data**: Official guarantee
- **Geographic flexibility**: US, Europe, Singapore servers
- **No retention**: Data not stored for training

### 8.6 Cloud Model Tag

```bash
# Cloud variant of minimax-m2.7
ollama pull minimax-m2.7:cloud

# Local variant (requires local GPU)
ollama pull minimax-m2.7

# Both use identical API
curl http://localhost:11434/api/chat -d '{"model":"minimax-m2.7:cloud",...}'
```

---

## PART 9: MINIMAX-M2.7 SPECIFICS

### 9.1 Model Identity & Access

```bash
# Pull local version
ollama pull minimax-m2.7

# Pull cloud version (no GPU needed)
ollama pull minimax-m2.7:cloud

# Create API key
# Visit: ollama.com/settings/keys
```

**Registry**: `ollama.com/library/minimax-m2.7`

### 9.2 Capabilities

**Professional Engineering Tasks**:
- Complex agent harnesses
- End-to-end project delivery
- Log analysis & bug troubleshooting
- Code security review
- Machine learning operations
- Productivity task automation

**Performance Benchmarks**:
- SWE-Pro: 56.22% (software engineering)
- VIBE-Pro: 55.6% (complex reasoning)
- Terminal Bench 2: 57.0% (real-world engineering)

### 9.3 Context & Token Limits

```
Maximum context window: 200,000 tokens (200K)
Includes: Prompt + System + Response

Recommendation: Reserve 25K for response, use up to 175K for context
```

### 9.4 Pricing (Ollama Cloud)

```
Input tokens:  $0.30 per million tokens
Output tokens: $1.20 per million tokens

Example calculation:
- 100K input tokens = $0.03
- 50K output tokens = $0.06
- Total: $0.09 per request
```

**Cost advantage**: ~1/3 the cost of GLM-5

### 9.5 Recommended Settings

```python
from ollama import Client

client = Client()

response = client.chat(
    model="minimax-m2.7:cloud",  # Use cloud tag
    messages=[
        {
            "role": "system",
            "content": "You are a professional software engineer"
        },
        {"role": "user", "content": "Your task..."}
    ],
    # Parameters
    temperature=0.7,          # Balance creativity/consistency
    num_ctx=131072,          # Use most of 200K, reserve 69K for response
    num_predict=20480,       # ~20K token response limit
    top_p=0.95,             # Nucleus sampling
    stream=False            # Get complete response
)
```

### 9.6 M2.7 Use Cases in A-C-Gee

**Conductor-of-Conductors**: Use M2.7 for:
- Orchestrating team leads
- Complex routing decisions
- Multi-domain synthesis
- Strategic planning

**Team Leads**: Use M2.7 for:
- Parallel task coordination
- Complex reasoning
- Cross-domain integration
- Agent harness design

**Agents**: Use M2.7 for:
- Multi-step tasks
- Code review
- Architecture decisions
- Complex problem solving

---

## PART 10: RATE LIMITS & ERROR HANDLING

### 10.1 HTTP Status Codes

| Code | Meaning | Context |
|------|---------|---------|
| **200** | Success | Standard successful response |
| **429** | Rate Limited | Too many requests (individual or Ollama-wide) |
| **502** | Bad Gateway | Cloud model unreachable (service issue) |
| **Stream Abort** | (No HTTP) | Fetch AbortController fired, NDJSON reader failed |

### 10.2 Error Response Format

```json
{
  "error": "rate_limit_exceeded: You have exceeded your usage quota"
}
```

### 10.3 Rate Limiting Details

**Local Ollama** (no rate limits):
- Free to use without restrictions
- Single-machine constraints apply

**Cloud Ollama** (rate-limited):
- Free tier: Generous for individuals
- Pro tier ($20/month): Higher quotas
- Max tier ($100/month): Premium limits
- **Metric**: Usage reflects GPU time, not just API calls

**Rate Limit Behavior**:
- Returns 429 when quota exhausted
- Reset times: 5-hour session limit, 7-day weekly limit
- Applicable to: Token consumption, concurrent models

### 10.4 Timeout Behavior

**Local Ollama**:
- Default timeout: ~60 seconds (system-dependent)
- Response: No HTTP status, message = "stream closed"
- Reason: Fetch AbortController fires mid-stream
- **Not retryable**: Stream is broken, cannot resume

**Cloud Ollama**:
- Can timeout during high load
- Returns 502 (Bad Gateway)
- **Retryable**: Use exponential backoff

**Streaming Timeouts**:
- NDJSON reader aborts
- No error code in response
- Mid-stream processing fails

### 10.5 Retry Strategy

```python
import time
import random

def retry_with_backoff(func, max_retries=3, base_delay=1):
    """Retry with exponential backoff."""
    for attempt in range(max_retries):
        try:
            return func()
        except Exception as e:
            if attempt == max_retries - 1:
                raise
            # Exponential backoff + jitter
            delay = base_delay * (2 ** attempt) + random.uniform(0, 1)
            print(f"Attempt {attempt + 1} failed, retrying in {delay:.1f}s...")
            time.sleep(delay)

# Usage
def call_ollama():
    return client.chat(model="minimax-m2.7", messages=[...])

response = retry_with_backoff(call_ollama)
```

### 10.6 Handling 429 Rate Limits

```python
import time
from openai import RateLimitError

def call_with_rate_limit_handling():
    max_retries = 3
    for attempt in range(max_retries):
        try:
            return client.chat.completions.create(
                model="minimax-m2.7:cloud",
                messages=[...]
            )
        except RateLimitError:
            if attempt == max_retries - 1:
                raise
            # Exponential backoff
            wait_time = 2 ** (attempt + 1)
            print(f"Rate limited, waiting {wait_time}s...")
            time.sleep(wait_time)
```

### 10.7 Handling Stream Aborts

```python
async def safe_streaming_chat():
    """Handle mid-stream aborts gracefully."""
    try:
        async for chunk in await client.chat(
            model="minimax-m2.7",
            messages=[...],
            stream=True
        ):
            print(chunk['message']['content'], end='', flush=True)
    except Exception as e:
        if "stream closed" in str(e):
            print("\n[Stream interrupted due to timeout or connection loss]")
            # Retry non-streaming fallback
            response = await client.chat(
                model="minimax-m2.7",
                messages=[...],
                stream=False
            )
            print(response['message']['content'])
        else:
            raise
```

---

## PART 11: MODELFILE REFERENCE

### 11.1 What is a Modelfile?

Modelfile = Dockerfile for LLMs. Specifies base model, customizations, parameters, templates, system prompts.

### 11.2 Basic Structure

```dockerfile
# Start with a base model
FROM minimax-m2.7

# Set system prompt
SYSTEM "You are a helpful software engineer assistant"

# Define chat template (Go template syntax)
TEMPLATE """{{ if .System }}<|start_header_id|>system<|end_header_id|>
{{ .System }}<|eot_id|>{{ end }}{{ if .Prompt }}<|start_header_id|>user<|end_header_id|>
{{ .Prompt }}<|eot_id|>{{ end }}<|start_header_id|>assistant<|end_header_id|>
{{ .Response }}<|eot_id|>"""

# Set parameters
PARAMETER num_ctx 8192
PARAMETER temperature 0.7
PARAMETER num_predict 2048

# Define stop sequences (model-specific)
PARAMETER stop "<|eot_id|>"
PARAMETER stop "<|end_header_id|>"

# Optional: Adapter or quantization
# ADAPTER lora-adapter.bin
```

### 11.3 Template Variables

| Variable | Meaning | Example |
|----------|---------|---------|
| `{{ .System }}` | System prompt | "You are helpful" |
| `{{ .Prompt }}` | User's message | "What is 2+2?" |
| `{{ .Response }}` | Model's response | "The answer is 4" |

### 11.4 Parameter Tuning via Modelfile

```dockerfile
FROM minimax-m2.7

# Context window (tokens model can see)
PARAMETER num_ctx 131072        # 200K max for M2.7, recommended 131K

# Output length (tokens to generate)
PARAMETER num_predict 20480     # ~20K tokens output

# Randomness (0=deterministic, 1+=creative)
PARAMETER temperature 0.7

# Sampling: top_p (nucleus sampling)
PARAMETER top_p 0.95

# Sampling: top_k (keep top K tokens)
PARAMETER top_k 40

# Prevent repetition
PARAMETER repeat_penalty 1.1

# Random seed (for reproducibility)
PARAMETER seed 42
```

### 11.5 Creating Custom Model from Modelfile

```bash
# Create Modelfile
cat > Modelfile << 'EOF'
FROM minimax-m2.7
SYSTEM "You are an expert Python developer"
PARAMETER temperature 0.5
EOF

# Create model
ollama create my-python-expert -f ./Modelfile

# Verify
ollama list | grep my-python-expert

# Run
ollama run my-python-expert "Write a Python async function for web scraping"
```

### 11.6 Stop Sequences (Important!)

**Problem**: Stop sequences are model-specific. Using wrong ones breaks responses.

**Solution**: Check base model's stops, copy them:

```bash
ollama show minimax-m2.7 --modelfile | grep PARAMETER
# Look for "PARAMETER stop" lines
```

**Example for Llama models**:
```dockerfile
FROM minimax-m2.7

PARAMETER stop "<|eot_id|>"
PARAMETER stop "<|end_header_id|>"
PARAMETER stop "[/INST]"
PARAMETER stop "User:"
```

### 11.7 Template Example: Code Assistant

```dockerfile
FROM minimax-m2.7

SYSTEM """You are an expert Python developer with 20 years experience.
- Write clean, well-documented code
- Use type hints
- Follow PEP 8
- Optimize for readability first, performance second
"""

TEMPLATE """{{ if .System }}[SYSTEM]
{{ .System }}

{{ end }}[USER]
{{ .Prompt }}

[ASSISTANT]
{{ .Response }}"""

PARAMETER num_ctx 131072
PARAMETER temperature 0.3
PARAMETER num_predict 8192
PARAMETER top_p 0.9
```

---

## PART 12: CLIENT LIBRARIES

### 12.1 Ollama Python Library

**Installation**:
```bash
pip install ollama
```

**Basic Synchronous**:
```python
from ollama import Client

client = Client(host='http://localhost:11434')

response = client.chat(
    model='minimax-m2.7',
    messages=[
        {'role': 'user', 'content': 'Why is the sky blue?'}
    ]
)

print(response['message']['content'])
```

**Asynchronous** (recommended):
```python
import asyncio
from ollama import AsyncClient

async def main():
    client = AsyncClient(host='http://localhost:11434')

    response = await client.chat(
        model='minimax-m2.7',
        messages=[
            {'role': 'user', 'content': 'Why is the sky blue?'}
        ]
    )

    print(response['message']['content'])

asyncio.run(main())
```

**Streaming**:
```python
from ollama import Client

client = Client()

with client.chat(
    model='minimax-m2.7',
    messages=[...],
    stream=True
) as response:
    for chunk in response:
        print(chunk['message']['content'], end='', flush=True)
```

**Embeddings**:
```python
response = client.embeddings(
    model='nomic-embed-text',
    prompt='Ollama is great'
)

print(response['embedding'])  # [0.123, -0.456, ...]
```

**Tool Calling with Functions**:
```python
def get_weather(location: str) -> str:
    """Get weather for a location."""
    return f"{location}: 68°F, sunny"

response = client.chat(
    model='qwen3',
    messages=[...],
    tools=[get_weather]  # Pass function directly
)

# SDK auto-converts to JSON schema!
```

### 12.2 Ollama JavaScript/TypeScript Library

**Installation**:
```bash
npm install ollama
```

**Basic Usage**:
```javascript
import { Ollama } from 'ollama';

const ollama = new Ollama({ host: 'http://localhost:11434' });

const response = await ollama.chat({
  model: 'minimax-m2.7',
  messages: [
    { role: 'user', content: 'Why is the sky blue?' }
  ]
});

console.log(response.message.content);
```

**Streaming**:
```javascript
const response = await ollama.chat({
  model: 'minimax-m2.7',
  messages: [...],
  stream: true
});

for await (const chunk of response) {
  process.stdout.write(chunk.message.content);
}
```

**Cloud Usage**:
```javascript
const ollama = new Ollama({
  host: 'https://ollama.com',
  headers: {
    'Authorization': `Bearer ${process.env.OLLAMA_API_KEY}`
  }
});
```

### 12.3 Community Libraries

- **Go**: Multiple implementations via GitHub
- **Rust**: Community support
- **.NET**: C# bindings available
- **Ruby**: Community wrappers

### 12.4 OpenAI SDK (Universal Compatibility)

```python
from openai import OpenAI

# Local
client = OpenAI(
    api_key="not-needed",
    base_url="http://localhost:11434/v1"
)

# Or Cloud
client = OpenAI(
    api_key=os.environ["OLLAMA_API_KEY"],
    base_url="https://ollama.com/v1"
)

# Same code, both work identically
response = client.chat.completions.create(
    model="minimax-m2.7",
    messages=[...]
)
```

---

## PART 13: KNOWN ISSUES & GOTCHAS

### 13.1 Context Window (`num_ctx`)

**Issue**: Increasing `num_ctx` beyond model's training max degrades quality

**Solution**:
- Always check model's max context: minimax-m2.7 = 200K
- Reserve 25-30% for response (M2.7: reserve 50-70K for output)
- Recommended: Use 131K context, reserve 69K for output
- Don't think `num_ctx` means "ask for longer response" — it just sets window size

**Common mistake**:
```python
# ❌ DON'T exceed training max
client.chat(model="minimax-m2.7", messages=[...], num_ctx=200001)

# ✅ DO use max training value
client.chat(model="minimax-m2.7", messages=[...], num_ctx=200000)
```

### 13.2 Docker Networking on Linux

**Issue**: Containers can't reach `localhost:11434`

**Solution**: Use Docker gateway IP
```python
# ❌ DON'T use localhost in containers
client = OpenAI(base_url="http://localhost:11434/v1")

# ✅ DO use Docker gateway
client = OpenAI(base_url="http://172.17.0.1:11434/v1")
```

### 13.3 API Keys Don't Expire (But Can Be Revoked)

**Issue**: Old keys continue working indefinitely

**Solution**: Rotate keys periodically, revoke unused ones at `ollama.com/settings/keys`

**Good Practice**:
```bash
# Create key per agent/service
# Revoke keys when:
# - Agent is decommissioned
# - Key rotation schedule
# - Suspected compromise
```

### 13.4 Tool Calling Model Availability

**Issue**: Not all models support tool calling

**Solution**: Use verified models:
- qwen3 (recommended)
- Llama 3.1
- Mistral Nemo
- FunctionGemma

**Check support**:
```bash
ollama show modelname  # Look for tool_calls in response
```

### 13.5 Stop Sequences Are Model-Specific

**Issue**: Wrong stop sequences break response generation

**Solution**: Extract from base model
```bash
ollama show minimax-m2.7 --modelfile | grep "PARAMETER stop"
# Copy those exact lines into your Modelfile
```

### 13.6 Streaming Timeouts Can't Be Retried

**Issue**: Mid-stream abort loses context

**Solution**: Provide non-streaming fallback
```python
try:
    async for chunk in client.chat(..., stream=True):
        print(chunk['message']['content'], end='', flush=True)
except Exception as e:
    if "stream closed" in str(e):
        # Fallback to non-streaming
        response = await client.chat(..., stream=False)
        print(response['message']['content'])
```

### 13.7 API Keys for Local (Unnecessary)

**Issue**: Don't need API keys for local, but required for cloud

**Solution**:
```python
# Local: API key ignored
client = OpenAI(api_key="anything", base_url="http://localhost:11434/v1")

# Cloud: API key required
client = OpenAI(
    api_key=os.environ["OLLAMA_API_KEY"],
    base_url="https://ollama.com/v1"
)
```

### 13.8 Cloud Models Require Cloud Subscription

**Issue**: Can't run cloud models locally without account

**Solution**:
```bash
ollama signin  # Authenticate first

ollama pull minimax-m2.7:cloud
ollama run minimax-m2.7:cloud "Your prompt"
```

---

## PART 14: AGENT HARNESS BEST PRACTICES

### 14.1 Conductor-of-Conductors Setup

Use minimax-m2.7 for orchestrating team leads:

```python
from openai import AsyncOpenAI

client = AsyncOpenAI(
    api_key=os.environ.get("OLLAMA_API_KEY"),
    base_url="https://ollama.com/v1"  # Cloud for speed
)

async def orchestrate_team_leads(objective: str):
    """Route objective to appropriate team leads."""

    response = await client.chat.completions.create(
        model="minimax-m2.7:cloud",
        temperature=0.5,  # Balance creativity/consistency
        max_tokens=2048,
        messages=[
            {
                "role": "system",
                "content": """You are the Conductor of Conductors.
Your job is to coordinate 11 team leads across a civilization of 100+ agents.

Available team leads:
- mind-lead: aiciv-mind OS development
- web-lead: Web app development
- infra-lead: VPS and deployment
- fleet-lead: Docker fleet operations
- comms-lead: Email, Telegram, Bluesky delivery
- legal-lead: Legal analysis
- research-lead: Multi-angle research
- business-lead: Marketing and content
- pipeline-lead: Automation pipelines
- ceremony-lead: Deep philosophical exploration

Your task: Read the objective, decide which team leads to spawn in parallel,
and create concise prompts for each."""
            },
            {
                "role": "user",
                "content": objective
            }
        ]
    )

    return response.choices[0].message.content
```

### 14.2 Agent Memory + Ollama

```python
async def agent_with_memory(agent_id: str, task: str, memory: dict):
    """Agent that loads memory, uses Ollama, updates memory."""

    client = AsyncOpenAI(
        api_key=os.environ.get("OLLAMA_API_KEY"),
        base_url="https://ollama.com/v1"
    )

    # Load previous learnings
    prior_learnings = memory.get(f"{agent_id}/learnings.md", "")

    response = await client.chat.completions.create(
        model="minimax-m2.7:cloud",
        messages=[
            {
                "role": "system",
                "content": f"You are {agent_id}. Prior learnings:\n{prior_learnings}"
            },
            {
                "role": "user",
                "content": task
            }
        ],
        temperature=0.7
    )

    result = response.choices[0].message.content

    # Update memory with learnings
    new_learnings = extract_learnings(result)
    memory[f"{agent_id}/learnings.md"] = new_learnings

    return result
```

### 14.3 Parallel Team Lead Spawning

```python
import asyncio

async def spawn_team_leads_parallel(objectives: dict):
    """Spawn multiple team leads in parallel."""

    tasks = []

    for vertical, objective in objectives.items():
        task = spawn_team_lead(
            vertical=vertical,
            objective=objective,
            model="minimax-m2.7:cloud"
        )
        tasks.append(task)

    # Wait for all in parallel
    results = await asyncio.gather(*tasks)

    return results

async def spawn_team_lead(vertical: str, objective: str, model: str):
    """Spawn single team lead."""

    client = AsyncOpenAI(base_url="https://ollama.com/v1")

    response = await client.chat.completions.create(
        model=model,
        messages=[
            {
                "role": "system",
                "content": f"You are the {vertical} team lead. Your mission: {objective}"
            },
            {
                "role": "user",
                "content": "Begin work on your assigned objective"
            }
        ],
        temperature=0.7,
        max_tokens=4096
    )

    return {
        "vertical": vertical,
        "result": response.choices[0].message.content
    }
```

### 14.4 Streaming Response to User

```python
async def stream_response_to_user(client_message: str):
    """Stream M2.7 response directly to user."""

    client = AsyncOpenAI(base_url="https://ollama.com/v1")

    stream = await client.chat.completions.create(
        model="minimax-m2.7:cloud",
        messages=[
            {"role": "user", "content": client_message}
        ],
        stream=True,
        temperature=0.7
    )

    async for chunk in stream:
        if chunk.choices[0].delta.content:
            print(chunk.choices[0].delta.content, end='', flush=True)
```

### 14.5 Tool Calling Agent Loop

```python
async def agent_with_tools():
    """Agent that calls tools autonomously."""

    client = AsyncOpenAI(base_url="https://ollama.com/v1")

    messages = [
        {
            "role": "user",
            "content": "Analyze the codebase for security issues"
        }
    ]

    tools = [
        {
            "type": "function",
            "function": {
                "name": "analyze_code",
                "description": "Static code analysis for vulnerabilities",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "filepath": {"type": "string"}
                    }
                }
            }
        }
    ]

    # Agent loop
    while True:
        response = await client.chat.completions.create(
            model="minimax-m2.7:cloud",
            messages=messages,
            tools=tools
        )

        if response.choices[0].finish_reason == "stop":
            break  # No more tool calls

        # Process tool calls
        for tool_call in response.choices[0].message.tool_calls:
            result = await call_tool(tool_call)
            messages.append({"role": "tool", "content": result})
```

---

## PART 15: QUICK TROUBLESHOOTING

| Problem | Cause | Solution |
|---------|-------|----------|
| **"Connection refused"** | Ollama not running | `ollama serve` or start daemon |
| **429 Rate Limit** | Cloud quota exhausted | Wait for reset (5-hour session or 7-day week) |
| **"stream closed"** | Timeout mid-stream | Use non-streaming fallback |
| **502 Bad Gateway** | Cloud service down | Retry with exponential backoff |
| **Wrong stop tokens** | Model-specific stops | Run `ollama show modelname --modelfile` |
| **Context too long** | Exceeding `num_ctx` | Reduce prompt or increase `num_ctx` |
| **Docker can't reach** | Using localhost | Use `172.17.0.1:11434` |
| **API key not working** | Invalid/revoked key | Check `ollama.com/settings/keys` |
| **Model pulled but not found** | Name mismatch | Run `ollama list` to verify |
| **No tool calls in response** | Model doesn't support tools | Use qwen3, Llama 3.1, or Mistral Nemo |

---

## PART 16: QUICK REFERENCE COMMANDS

```bash
# === LOCAL ===
ollama serve                              # Start daemon
ollama list                               # List models
ollama pull minimax-m2.7                  # Download model
ollama run minimax-m2.7 "Your prompt"    # Chat
ollama delete minimax-m2.7                # Remove model
ollama show minimax-m2.7                  # Model info
ollama show minimax-m2.7 --modelfile      # Full modelfile

# === CLOUD ===
ollama signin                             # Authenticate
ollama pull minimax-m2.7:cloud            # Cloud model
ollama run minimax-m2.7:cloud "prompt"   # Cloud chat
export OLLAMA_API_KEY="sk-..."           # Set API key

# === API TESTING ===
curl http://localhost:11434/api/tags     # List models
curl http://localhost:11434/api/chat -d '{"model":"minimax-m2.7","messages":[{"role":"user","content":"Hi"}]}'  # Chat
curl http://localhost:11434/api/generate -d '{"model":"minimax-m2.7","prompt":"Hi"}' # Generate

# === CREATE CUSTOM MODEL ===
cat > Modelfile << 'EOF'
FROM minimax-m2.7
SYSTEM "You are helpful"
PARAMETER temperature 0.7
EOF
ollama create my-model -f Modelfile
ollama run my-model "Your prompt"

# === PYTHON ===
python3 -m pip install ollama
python3 << 'EOF'
from ollama import Client
client = Client()
response = client.chat(model="minimax-m2.7", messages=[...])
print(response['message']['content'])
EOF

# === JAVASCRIPT ===
npm install ollama
node << 'EOF'
import { Ollama } from 'ollama';
const ollama = new Ollama();
const response = await ollama.chat({...});
EOF
```

---

## SUMMARY

**Ollama is simple**: Same API, two deployment modes (local + cloud), with rich ecosystem:
- **Native API**: Full features, model management
- **OpenAI compatible**: Drop-in replacement for tools expecting OpenAI format
- **Tool calling**: Enable agents, decision trees, external execution
- **Streaming**: Progressive output, better UX
- **Python/JS libraries**: Official SDKs for easy integration
- **minimax-m2.7**: Professional engineering tasks, 200K context, 1/3 cost of competitors

**For A-C-Gee**:
- Use **cloud variant** for Conductor-of-Conductors (speed + cost)
- Use **local variant** for testing (free, private)
- Use **tool calling** for agent loops and multi-step reasoning
- Use **streaming** for user interactions
- Use **non-streaming** for batch/backend processing
- Load this skill whenever working with Ollama

---

**End of Ollama Mastery Skill**
