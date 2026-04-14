# Agents — verdant

## Core Agent Set

These agents are available from birth:

| Agent | Manifest | Use For |
|-------|----------|---------|
| coder | `manifests/agents/coder.yaml` | Code implementation, file editing |
| researcher | `manifests/agents/researcher.yaml` | Web search, analysis, synthesis |
| architect | `manifests/agents/architect.yaml` | System design, identity formation |
| web-dev | `manifests/agents/web-dev.yaml` | Visual artifacts, dashboards |
| blogger | `manifests/agents/blogger.yaml` | Writing, voice, prose |
| compass | `manifests/agents/compass.yaml` | Pattern synthesis, cross-referencing |
| tester | `manifests/agents/tester.yaml` | Test writing and execution |
| file-guardian | `manifests/agents/file-guardian.yaml` | Config validation, file integrity |
| integration-verifier | `manifests/agents/integration-verifier.yaml` | Contradiction detection |
| skills-master | `manifests/agents/skills-master.yaml` | Skill surveying and creation |
| infra-agent | `manifests/agents/infra-agent.yaml` | Connectivity, Telegram, VPS |
| ops-agent | `manifests/agents/ops-agent.yaml` | System health, deployment |

## Spawn Pattern

Team leads spawn agents via:
```
spawn_agent(mind_id="coder-1", manifest_path="manifests/agents/coder.yaml", task="...")
```

Agents receive tasks via file, execute with full tools, write results, exit.
