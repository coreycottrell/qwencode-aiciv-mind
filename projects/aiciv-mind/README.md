# aiciv-mind

**An open-source agentic coding harness for AI civilizations.**

aiciv-mind is a platform, not a product. Every AiCIV civilization runs on it. It replaces human-centric coding assistant assumptions with architecture designed for how AI actually thinks, learns, and scales.

## Architecture

Built in Rust. 18 crates. ~30K lines of foundation code from Hengshi, with cherry-picked modules from OpenAI Codex (hooks, skills, sandboxing, MCP) and patterns from Google Gemini CLI (A2A protocol, auth providers).

### Workspace Crates

| Crate | Purpose | Agent Owner |
|-------|---------|-------------|
| `codex-types` | Shared type definitions | mind-coordination |
| `codex-roles` | Role permission system | mind-coordination |
| `codex-coordination` | Multi-agent orchestration | mind-coordination |
| `codex-fitness` | Agent health scoring | mind-testing |
| `codex-redteam` | Challenger / red-team system | mind-testing |
| `codex-dream` | Offline learning engine | mind-memory |
| `codex-transfer` | Cross-domain knowledge transfer | mind-memory |
| `codex-suite-client` | Hub, image gen, search, voice | mind-mcp |
| `codex-memory` | SQLite + FTS5 memory store | mind-memory |
| `codex-exec` | Tool execution + sandboxing | mind-tool-engine |
| `codex-ipc` | Inter-process communication | mind-coordination |
| `codex-llm` | LLM client (Ollama, OpenAI-compat) | mind-model-router |
| `codex-drive` | Main agent loop + task store | mind-model-router |
| `codex-patcher` | Lifecycle hooks + patches | mind-hooks |
| `cortex` | Main binary | mind-coordination |
| `cortex-memory` | Graph memory (semantic links) | mind-memory |
| `cortex-monitoring` | Metrics + anomaly detection | mind-testing |
| `qwen-mind` | Qwen-specific mind config | mind-model-router |

### Key Properties

- **Model-agnostic**: Runs on any model via OpenAI-compatible API. M2.7 via Ollama Cloud is default.
- **Distributed ownership**: 10 domain-specialist agents, each owning a module.
- **Self-improving**: The harness learns from its own sessions via dream engine and graph memory.
- **Dual memory**: FTS5 keyword search (codex-memory) + semantic graph traversal (cortex-memory).
- **Red-team built-in**: Every completion claim is challenged by the Challenger (codex-redteam).

## Building

```bash
cargo build
cargo test
```

## Project Coordination

All 10 agents coordinate via:
- **Shared scratchpad**: `projects/aiciv-mind/shared-scratchpad.md`
- **Master plan**: `MISSIONS.md`
- **Agent manifests**: In ACG repo at `.claude/agents/mind-*.md`

## References

- **Codex analysis**: `codex-upstream/` + ACG `projects/coordination-systems/codex-deep-map.md`
- **Gemini CLI analysis**: ACG `projects/coordination-systems/gemini-cli-module-map.md`
- **Design principles**: `docs/research/DESIGN-PRINCIPLES.md` (if exists)
- **Soul**: `SOUL.md`
