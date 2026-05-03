---
name: atropos-grpo
description: Nous Research GRPO RL training for tool-calling agents. Atropos install verified on M2.7, GRPO loss + BaseEnv + RewardRegistry functional. Independent of Hub identity.
version: 1.0.0
trigger: python3 skills/atropos-grpo/atropos_grpo_runner.py
---

# Atropos GRPO Firing Contract

## WHEN

CLI run to verify Atropos GRPO system is functional:
```bash
python3 skills/atropos-grpo/atropos_grpo_runner.py
```

Also: any GRPO training run via `atropos-grpo` CLI or `python -m example_trainer.grpo`.

## WHAT

**4 test areas verified:**
1. `import` — all core modules import (atroposlib, grpo, trainers, api, training, vllm_manager, BaseEnv, ScoredDataGroup, FormatReward, ManagedServer)
2. `grpo_loss` — `compute_grpo_loss` has expected params (advantages, clip_eps)
3. `reward_registry` — RewardRegistry + FormatReward + ReasoningStepsReward + RepetitionPenaltyReward all instantiable
4. `base_env` — BaseEnv abstract class with `collect_trajectories` abstract method; ScoredDataGroup has 13 fields (tokens, masks, scores, advantages, inference_logprobs, etc.)

## PRE

| Prerequisite | How Verified |
|--------------|--------------|
| `/tmp/atropos` cloned from NousResearch | `ls /tmp/atropos/atroposlib/` |
| Python packages installed | `pip list \| grep -E "torch\|transformers\|vllm\|aiohttp"` |
| venv activated (optional) | `/tmp/atropos-venv/bin/python` |

**Install commands:**
```bash
git clone --depth=1 https://github.com/NousResearch/atropos.git /tmp/atropos
python3 -m venv /tmp/atropos-venv
/tmp/atropos-venv/bin/pip install torch --index-url https://download.pytorch.org/whl/cpu
/tmp/atropos-venv/bin/pip install -r /tmp/atropos/requirements.txt
/tmp/atropos-venv/bin/pip install wandb peft openai aiohttp pydantic-cli markdown
```

## POST

| State | Condition |
|-------|-----------|
| All 4 tests pass | Exit code 0, "ALL TESTS PASSED" printed |
| Any test fails | Exit code 1, failed test name printed |
| Import fails | ModuleNotFoundError propagated |
| GRPO training mode | `train_legacy \| train_lora \| train_lora_restart \| train_shared_vllm` |

## FAILURE

| Failure Mode | Detection | Recovery |
|-------------|-----------|----------|
| Missing dependency | `ModuleNotFoundError` | Install via pip |
| Wrong Python env | Wrong `atroposlib` version | Use `/tmp/atropos-venv/bin/python` |
| vLLM not available | `vllm` import fails (CPU-only env) | Training requires GPU vLLM |
| Config parse error | `AttributeError` on FakeArgs | Use `argparse.ArgumentParser` for real args |

## OBSERVABILITY

All tests print pass/fail with details:
- Module import list with ✅/❌
- `compute_grpo_loss` param names listed
- ScoredDataGroup field names listed
- Abstract methods listed

## TRAINING MODES

| Mode | Speed | Notes |
|------|-------|-------|
| `legacy` | Slow | vLLM restarts each step |
| `shared_vllm` | Fast | Requires `VLLM_ENABLE_SHARED_WEIGHTS=1` |
| `lora_only` | Slow (13 TPS) | `--enforce-eager` required |
| `lora_restart` | Fast (170 TPS) | CUDA graphs enabled, restarts vLLM every N steps |

## ENV VARS FOR TRAINING

| Variable | Default | Purpose |
|----------|---------|---------|
| `VLLM_ENABLE_SHARED_WEIGHTS` | unset | Required for `shared_vllm` mode |
| `WANDB_MODE` | `disabled` | Set to `online` for wandb logging |
| `ATROPOS_API_URL` | `http://localhost:8901` | Atropos API server URL |

## TOOL-CALLING ENV PATTERN

```python
class ToolCallingEnv(BaseEnv):
    async def collect_trajectories(self, item):
        prompt = format_tool_prompt(item)  # includes tool schema

        async with self.server.managed_server(tokenizer=self.tokenizer) as managed:
            completion = await managed.completion(
                prompt=prompt,
                n=self.config.group_size,
                max_tokens=4096,
                temperature=1.0,
            )
            state = managed.get_state()
            nodes = state["nodes"]

        for choice, node in zip(completion.choices, nodes):
            tokens = node.tokens
            masked_tokens = node.masked_tokens  # -100 for prompt
            logprobs = node.logprobs           # 1.0 for masked
            # Score and return...
```
