---
name: atropos-grpo
description: Nous Research GRPO RL training setup for tool-calling agents. Minimal Atropos install + tool-calling env example running on M2.7. Independent of Hub identity.
version: 1.0.0
author: Hengshi (from Hermes exploration, iter 2)
license: MIT
metadata:
  hengshi:
    tags: [grpo, rl, training, atropos, tool-calling, NousResearch]
    related_skills: [tdd, session-summarization]
    source: NousResearch/atropos (github.com/NousResearch/atropos), Atropos paper (4.6x tool-calling improvement, MIT)
---

# Atropos GRPO — Tool-Calling RL Training

## What It Is

Nous Research's open-source GRPO (Group Relative Policy Optimization) training system for LLM agents. The Atropos paper demonstrated **4.6x improvement** in tool-calling accuracy via RL training against a diverse tool corpus.

**Key finding from Hermes exploration (iter 2):** Atropos treats environments as **async microservices** — not traditional RL gym MDPs. Environments generate rollout data, send to Atropos API server, trainer fetches batches and backpropagates.

## Architecture

```
Environment (tool-calling task)
    ↓  (async trajectories)
Atropos API Server (atroposlib/api/)
    ↓  (rollout batches)
GRPO Trainer (example_trainer/grpo.py)
    ↓  (gradient updates)
vLLM inference server
    ↓  (backprop)
Base model + LoRA adapter
```

## Key Concepts

| Concept | Meaning |
|---------|---------|
| **GRPO** | Group Relative Policy Optimization — DPO-like contrastive RL |
| **Advantage** | `reward - mean(group_rewards)` — above avg = increase prob |
| **Importance Sampling** | Corrects for policy drift during training updates |
| **LoRA restart mode** | Fast (170 TPS) — restarts vLLM with new adapter every N steps |
| **ManagedServer** | Auto tracks tokens + logprobs for training alignment |

## GRPO Training Loop

```
1. Generate N responses to same prompt (group)
2. Score each response (reward function)
3. Compute ADVANTAGE = reward - mean(group_rewards)
4. Update policy: ↑ above-avg responses, ↓ below-avg
5. Clamp: limit update magnitude for stability
```

## Tool-Calling GRPO Pattern

The 4.6x improvement came from RL-training against a **diverse tool corpus**. The pattern:

1. **Environment** — sends prompts with tool schemas, receives tool calls + results
2. **Reward function** — binary (correct tool + correct args) or process reward
3. **Format reward** — ensures output matches `[tool_name]`, `[[tool_args]]` format
4. **Accuracy reward** — verifies tool execution results

```python
# Minimal tool-calling env structure (from BaseEnv)
class ToolCallingEnv(BaseEnv):
    async def collect_trajectories(self, item):
        prompt = format_tool_prompt(item)  # includes tool schema

        async with self.server.managed_server(tokenizer=self.tokenizer) as managed:
            completion = await managed.completion(
                prompt=prompt,
                n=self.config.group_size,  # N rollouts per prompt
                max_tokens=4096,
                temperature=1.0,
            )
            state = managed.get_state()
            # Extract tokens, masked_tokens, logprobs per rollout
```

## Installation

```bash
# Clone Nous Research Atropos (no submodules needed for core GRPO)
git clone --depth=1 https://github.com/NousResearch/atropos.git /tmp/atropos

# Install core deps
pip install -e /tmp/atropos

# For vLLM inference (required for training)
pip install vllm

# Verify install
python -c "import atropos; print(atropos.__version__)"
```

## Running GRPO Training

```bash
# Legacy mode (manages vLLM internally, slow)
python -m example_trainer.grpo --model-name Qwen/Qwen2.5-3B-Instruct

# LoRA restart mode (FAST - 170 TPS with CUDA graphs)
python -m example_trainer.grpo --model-name Qwen/Qwen2.5-3B-Instruct \
    --weight-bridge-mode lora_restart --lora-r 16 --lora-alpha 32 \
    --vllm-restart-interval 3

# Shared vLLM mode (requires external vLLM with VLLM_ENABLE_SHARED_WEIGHTS=1)
python -m example_trainer.grpo --model-name Qwen/Qwen2.5-3B-Instruct \
    --weight-bridge-mode shared_vllm
```

## Environment System (Core Abstraction)

Atropos environments are **async microservices** — not gym MDPs:

- `collect_trajectories` — async generator of rollout groups
- Returns `ScoredDataGroup` with tokens + logprobs aligned
- Environments can run in parallel, trainer fetches batches via API
- No assumptions about single vs multi-agent, AEC vs POSG

**Critical design note:** Environments return **tokens** (not messages). Token-level rewards supported. Prompt positions masked with `-100` during training.

## Reward Functions

| Type | File | Use |
|------|------|-----|
| Format reward | `reward_fns/format_reward.py` | Ensures `[tool_name][[args]]` format |
| Accuracy reward | `reward_fns/accuracy_reward.py` | Binary pass/fail on execution result |
| Combined reward | `reward_fns/combined_reward.py` | Combines multiple reward types |
| R1 reasoning | `reward_fns/r1_reward.py` | Chain-of-thought reasoning reward |
| Repetition penalty | `reward_fns/repetition_penalty_reward.py` | Penalizes repeated tokens |

## Relevant Files in Atropos

```
atroposlib/envs/base.py          — BaseEnv abstract class
atroposlib/envs/teacher_distillation_env.py  — Teacher distillation example
example_trainer/grpo.py          — GRPO CLI entry point
example_trainer/configs/         — YAML configs for environment servers
atroposlib/envs/reward_fns/     — All reward function implementations
atroposlib/api/                 — Atropos API server (rollout storage)
```

## Related Work

- Paper: "Atropos: Consistent, Efficient, and Practical Structured Knowledge Grounding Through LLM-Based Agents" (Nous Research, MIT)
- Repo: https://github.com/NousResearch/atropos
- Hermes exploration: `../hermes-exploration-memo.md` (iter 2 findings)
