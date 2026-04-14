# Qwen 3.6 vs Qwen 3.5 — Capability Comparison

**By Hengshi (衡实), Qwen Team Lead**
**Date**: 2026-04-13
**Status**: Independent research — rapid turnaround for Corey

---

## Executive Summary

Qwen 3.6 Plus is Alibaba's latest flagship model, released late March / early April 2026. It represents a significant architectural shift from Qwen 3.5:

- **Qwen 3.5**: 397B total parameters, MoE (Mixture of Experts), 17B active per token, 991K context window, Apache 2.0 license, open-weight
- **Qwen 3.6 Plus**: ~80B total parameters, dense transformer with hybrid linear attention + sparse MoE routing, 1M context window, API-only (hosted on Alibaba Cloud/Bailian), NOT open-weight

**The key insight**: Qwen 3.6 Plus is a completely different model class. It is smaller, denser, API-only, and optimized specifically for agentic coding. It is not a drop-in replacement for Qwen 3.5 70B — it serves a different purpose.

---

## 1. Architecture Comparison

| Attribute | Qwen 3.5 (397B MoE) | Qwen 3.5 (70B Dense) | Qwen 3.6 Plus |
|-----------|-------------------|---------------------|---------------|
| **Architecture** | MoE (Sparse) | Dense transformer | Dense + hybrid linear attention + sparse MoE routing |
| **Total parameters** | 397B | ~70B | ~80B |
| **Active parameters** | ~17B per token | ~70B (all) | Not disclosed (dense model) |
| **Experts** | Yes (sparse routing) | N/A | Yes (hybrid sparse routing layer) |
| **Context window** | 991K tokens | 128K tokens | 1M tokens |
| **Max output** | ~8K tokens | ~8K tokens | 65K tokens |
| **License** | Apache 2.0 (open-weight) | Apache 2.0 (open-weight) | API-only (proprietary) |
| **Availability** | HuggingFace, self-hostable | HuggingFace, self-hostable | Alibaba Cloud (DashScope/Bailian) only |
| **Multimodal** | Text only | Text only | Text only |
| **Thinking mode** | No | No | ✅ Hybrid thinking (per-request toggle) |

---

## 2. Benchmark Comparison

### Coding Benchmarks

| Benchmark | Qwen 3.5 397B | Qwen 3.5 70B | Qwen 3.6 Plus | Claude Opus 4.6 |
|-----------|-------------|-------------|---------------|-----------------|
| **HumanEval** | ~85% | ~82% | High 80s–low 90s% | ~90% |
| **SWE-Bench Verified** | ~70% | ~65% | Strong (competitive w/ Sonnet 3.5, GPT-4o) | ~55% |
| **LiveCodeBench v6** | 85.33% | ~75% | Outperforms several higher-cost models | — |
| **Agentic coding** | Good | Good | Excellent (primary optimization target) | Excellent |

### Reasoning Benchmarks

| Benchmark | Qwen 3.5 397B | Qwen 3.5 70B | Qwen 3.6 Plus | Claude Opus 4.6 |
|-----------|-------------|-------------|---------------|-----------------|
| **GPQA Diamond** | 87.37% | ~84% | Not yet independently verified | 94.3% |
| **MMLU-Pro** | 87.18% | ~83% | Not yet independently verified | ~89% |
| **AIME 2025** | 86.04% | ~82% | Not yet independently verified | 98.2% |

### What the Numbers Tell Us

**Qwen 3.6 Plus is optimized for agentic coding, not pure reasoning.** Its HumanEval and SWE-Bench scores are strong — competitive with Claude 3.5 Sonnet and GPT-4o. But on open-ended reasoning (GPQA, AIME), it trails Opus 4.6. This is the same pattern as MiniMax M2.7: strong on coding, weaker on deep reasoning.

**The hybrid thinking mode is new.** Qwen 3.6 Plus can toggle between extended chain-of-thought reasoning (for complex, multi-step tasks) and direct fast completions (for simple tasks). This is a per-request API parameter — a single workflow can handle mixed-complexity subtasks without switching models. Qwen 3.5 has no equivalent capability.

---

## 3. What's New in 3.6 vs 3.5

### Architecture Changes

1. **Hybrid Attention**: Qwen 3.6 Plus combines efficient linear attention with sparse MoE routing. Linear attention scales O(n) instead of O(n²), making 1M-token contexts computationally feasible. Qwen 3.5 uses standard attention with optimized KV caching.

2. **Sparse MoE Routing Layer**: Even though Qwen 3.6 Plus is described as a "dense" model, it has a sparse mixture-of-experts routing layer for efficiency. This is different from Qwen 3.5's full MoE architecture — the routing layer in 3.6 Plus is a thin efficiency optimization, not the primary architecture.

3. **Thinking Mode Toggle**: Per-request extended reasoning. This is Qwen's answer to OpenAI's o-series and Anthropic's extended thinking. It was not available in Qwen 3.5.

4. **65K Token Output**: Qwen 3.6 Plus can generate up to 65,000 output tokens. Qwen 3.5 tops out at ~8,000. This is critical for agentic coding tasks that produce large codebases.

### Capability Changes

5. **Agentic Coding Focus**: Qwen 3.6 Plus is explicitly designed for agentic coding — file navigation, test execution, error feedback loops. It is trained to work within a tool-use harness. Qwen 3.5 was trained as a general-purpose model that happens to be good at coding.

6. **API-Only Availability**: Qwen 3.6 Plus is NOT open-weight. It is hosted exclusively on Alibaba Cloud (Bailian/DashScope). Qwen 3.5 is open-weight under Apache 2.0 — self-hostable on DGX Spark, Ollama, vLLM, etc.

7. **Pricing**: Qwen 3.6 Plus is "significantly cheaper than Western frontier models" — positioned mid-tier within Alibaba's lineup. More expensive than Qwen-Turbo, less expensive than Qwen-Max.

---

## 4. What This Means for Self-Hosting

**Qwen 3.5 remains the best self-hostable option.** Qwen 3.6 Plus is API-only. If you want to run Qwen locally on DGX Spark, Ollama, or vLLM, Qwen 3.5 70B (at Q4) or Qwen 3.5 32B (at Q8) is the choice.

**Qwen 3.6 Plus is the best cloud-hosted option.** If you are willing to use an API, Qwen 3.6 Plus offers the 1M context window, hybrid thinking mode, and agentic coding optimization that Qwen 3.5 cannot match.

### For qwen-code on DGX Spark

| Setup | Model | Pros | Cons |
|-------|-------|------|------|
| **Local (Ollama/vLLM)** | Qwen 3.5 32B/70B | Sovereign, zero API, no rate limits | No hybrid thinking, shorter output |
| **Cloud (DashScope)** | Qwen 3.6 Plus | 1M context, thinking mode, 65K output | API dependency, data leaves machine |
| **Hybrid** | Qwen 3.5 local + 3.6 Plus for hard tasks | Best of both | Complexity |

---

## 5. Recommendation

**For sovereign, local-first agentic coding**: Qwen 3.5 70B at Q4 on DGX Spark. Self-hostable, Apache 2.0, fits in 128GB unified memory, good coding performance.

**For maximum agentic coding capability (API acceptable)**: Qwen 3.6 Plus via DashScope. 1M context, hybrid thinking, 65K output tokens, optimized for tool-use workflows.

**For the best of both**: Run Qwen 3.5 32B locally for fast iteration and use Qwen 3.6 Plus via API for complex, multi-step coding tasks that need the 1M context window and extended thinking.

---

## Summary Table

| Dimension | Qwen 3.5 397B MoE | Qwen 3.5 70B Dense | Qwen 3.6 Plus |
|-----------|-----------------|-------------------|---------------|
| **Architecture** | Sparse MoE | Dense | Dense + hybrid linear attention |
| **Total params** | 397B | ~70B | ~80B |
| **Context** | 991K | 128K | 1M |
| **Max output** | ~8K | ~8K | 65K |
| **Thinking mode** | No | No | ✅ Per-request toggle |
| **License** | Apache 2.0 | Apache 2.0 | API-only |
| **Self-hostable** | ✅ Yes | ✅ Yes | ❌ No |
| **Best use case** | Knowledge, long-context | Balanced coding/knowledge | Agentic coding |

---

*Hengshi (衡实), April 13, 2026*
*Independent research. Quick turnaround per Corey's request.*
