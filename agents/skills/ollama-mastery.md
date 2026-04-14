---
name: ollama-mastery
description: Comprehensive Ollama reference — API endpoints, cloud, tool calling, authentication, streaming, Modelfile, client libraries, error handling
type: reference
version: 1.0
for: Cortex (aiciv-mind-cubed)
---

# Ollama Mastery Skill — Cortex Edition

This is the Cortex-optimized version of the Ollama Mastery Skill. Use for aiciv-mind development and execution.

---

## QUICK START FOR CORTEX

```rust
// In your Cortex DriveLoop or Agent task
use ollama::{Client, ChatRequest};

// Local M2.7
let client = Client::new("http://172.17.0.1:11434");
let response = client.chat(ChatRequest {
    model: "minimax-m2.7".to_string(),
    messages: vec![...],
    stream: false,
}).await?;

// Cloud M2.7 (preferred for speed)
let client = Client::new_cloud(&env!("OLLAMA_API_KEY"));
let response = client.chat(ChatRequest {
    model: "minimax-m2.7:cloud".to_string(),
    messages: vec![...],
    stream: false,
}).await?;
```

---

## API ENDPOINTS REFERENCE

### **Native Ollama API**
- **Local**: `http://localhost:11434/api`
- **Cloud**: `https://ollama.com/api`
- **Docker (Linux)**: `http://172.17.0.1:11434/api`

### **Core Endpoints**
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/chat` | POST | Chat conversations (with history) |
| `/api/generate` | POST | Simple text completion |
| `/api/embeddings` | POST | Vector embeddings for RAG |
| `/api/tags` | GET | List models |
| `/api/pull` | POST | Download model |
| `/api/delete` | DELETE | Remove model |

### **OpenAI Compatible**
- `POST /v1/chat/completions` (identical to OpenAI)
- `POST /v1/embeddings`
- `GET /v1/models`

---

## AUTHENTICATION

### Local (No Auth)
```
http://localhost:11434/api/chat
(no headers needed)
```

### Cloud (Bearer Token)
```
Authorization: Bearer sk-ollama-abc123...

# Set env var
export OLLAMA_API_KEY="sk-ollama-..."

# Create at: ollama.com/settings/keys
```

---

## STREAMING vs NON-STREAMING

### Streaming (Default)
```
POST /api/chat?stream=true
Content-Type: application/x-ndjson

{"message":{"content":"Hello"},"done":false}
{"message":{"content":" world"},"done":false}
{"message":{"content":"!"},"done":true,"done_reason":"stop"}
```

### Non-Streaming
```
POST /api/chat
Body: {"model":"minimax-m2.7","messages":[...],"stream":false}

Response: Single JSON object with complete response
```

---

## MINIMAX-M2.7 SPECS

**Context**: 200K tokens max (reserve 25-30% for output)
**Cost**: $0.30 in / $1.20 out (per million)
**Benchmarks**: SWE-Pro 56.22%, Terminal Bench 57%
**Use cases**: Complex agent harnesses, project delivery, code review

**Recommended Settings**:
```json
{
  "model": "minimax-m2.7:cloud",
  "temperature": 0.7,
  "num_ctx": 131072,
  "num_predict": 20480,
  "top_p": 0.95,
  "stream": false
}
```

---

## TOOL CALLING

**Supported Models**: qwen3, Llama 3.1, Mistral Nemo, FunctionGemma

```json
{
  "model": "qwen3",
  "messages": [...],
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather for location",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {"type": "string"}
          },
          "required": ["location"]
        }
      }
    }
  ]
}
```

**Response includes `tool_calls` array** with model's function invocations. Provide results in follow-up message for agent loops.

---

## ERROR HANDLING

| Code | Meaning | Solution |
|------|---------|----------|
| 200 | Success | Standard |
| 429 | Rate Limited | Exponential backoff |
| 502 | Cloud Down | Retry |
| Stream Abort | Timeout | Fallback to non-streaming |

---

## MODELFILE FOR CUSTOM MODELS

```dockerfile
FROM minimax-m2.7

SYSTEM "You are a professional software engineer"

PARAMETER num_ctx 131072
PARAMETER temperature 0.5
PARAMETER num_predict 8192
PARAMETER stop "<|eot_id|>"
PARAMETER stop "<|end_header_id|>"
```

Create with: `ollama create my-model -f Modelfile`

---

## CLIENT LIBRARIES

### Python
```python
from ollama import AsyncClient

client = AsyncClient(host='http://172.17.0.1:11434')
response = await client.chat(
    model='minimax-m2.7:cloud',
    messages=[...],
    stream=False
)
```

### JavaScript
```javascript
import { Ollama } from 'ollama';
const ollama = new Ollama({ host: 'http://172.17.0.1:11434' });
const response = await ollama.chat({
    model: 'minimax-m2.7:cloud',
    messages: [...]
});
```

### Rust
```rust
// Via reqwest + serde_json or native Ollama Rust client
let response = client.chat(ChatRequest {
    model: "minimax-m2.7:cloud".to_string(),
    messages: vec![...],
    stream: false
}).await?;
```

---

## CLOUD PRICING TIERS

| Plan | Price/Month | Concurrent | Usage |
|------|-------------|-----------|-------|
| Free | $0 | 1 | Generous |
| Pro | $20 | 3 | Expanded |
| Max | $100 | 10 | Premium |

**Reset**: 5-hour session, 7-day weekly limits

---

## KNOWN GOTCHAS

1. **Docker Linux**: Use `172.17.0.1:11434` not `localhost`
2. **Stop sequences**: Model-specific, extract from base model
3. **Context window**: `num_ctx` = input limit, not output request
4. **Stream timeouts**: Can't retry mid-stream, use non-streaming fallback
5. **Tool calling**: Not all models support it (verify with qwen3, Llama 3.1, etc.)

---

## QUICK COMMANDS

```bash
# Local
ollama pull minimax-m2.7
ollama run minimax-m2.7 "Your prompt"
ollama list

# Cloud
ollama signin
ollama pull minimax-m2.7:cloud
ollama run minimax-m2.7:cloud "Your prompt"

# API test
curl http://172.17.0.1:11434/api/tags
curl http://172.17.0.1:11434/api/chat -d '{"model":"minimax-m2.7","messages":[{"role":"user","content":"Hi"}]}'
```

---

## FOR CORTEX SPECIFICALLY

**DriveLoop Integration**:
- Use `minimax-m2.7:cloud` for high-speed orchestration
- Use `num_ctx=131072` (reserve 69K for responses)
- Enable `stream=false` for batch processing in Tasks
- Use tool calling for decision trees and agent coordination

**Performance Tips**:
- M2.7 on cloud: ~2-5s per request (depends on token count)
- Cache frequent patterns in memory
- Use embeddings for semantic search within Cortex
- Batch multiple short requests instead of single long one

---

**Full reference**: See `/home/corey/projects/AI-CIV/ACG/.claude/skills/ollama-mastery/SKILL.md`

**Version**: 1.0 | **Date**: 2026-04-04
