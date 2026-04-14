# AiCIV Mind — Proven Architecture

**Built and tested. Python first. Documents as memory. Hard delegation rules.**

## What's Proven ✅

```
Primary (conductor)
├── CAN: spawn TeamLeads, delegate to TeamLeads, coordinate
├── CANNOT: spawn Agents, execute tools
└── HARD: raises DelegationError if violated

TeamLead (coordinator)
├── CAN: spawn Agents, delegate to same-vertical Agents
├── CANNOT: spawn TeamLeads, execute tools
└── HARD: raises DelegationError if violated

Agent (executor)
├── CAN: bash, read, write, glob, grep, memory ops
├── CANNOT: spawn children, delegate to anyone
└── HARD: raises DelegationError if violated
```

## Memory: Documents Win

- Each memory = Markdown file with YAML front matter
- Edges = JSON index (`_edges.json`)
- Search = ripgrep (fast enough at our scale)
- Traverse = load edges, BFS from node
- **You can literally read the mind's thoughts as files**

## File Structure (per mind)

```
minds/{mind_id}/
├── _edges.json                    # Graph edges
├── {tier}/{category}/{id}.md     # Memories
scratchpads/{mind_id}/{date}.md    # Working notes
manifests/{mind_id}.json           # Identity + growth
fitness/{mind_id}.jsonl            # Performance scores
```

## Gentle API Rules

- 30s minimum between calls
- Never parallel — sequential only
- Retry with exponential backoff on 500
- Log every call to scratchpad

## Next: Build the Real System

### Phase 1: Core Mind System (DONE — proved above)
- [x] Mind classes with hard delegation
- [x] Document-based memory
- [x] Scratchpad, fitness, manifest
- [x] Gentle async API calls

### Phase 2: The Full Mind Hierarchy (TODO)
- [ ] Primary mind with Dream Mode
- [ ] 5 Team Leads (research, code, ops, architecture, qwen)
- [ ] Agents for each vertical
- [ ] Cross-mind communication

### Phase 3: Growth (TODO)
- [ ] Dream Mode: consolidate, archive, evolve manifest
- [ ] Pattern detection → specialist spawning
- [ ] Anti-pattern learning from failures
- [ ] Fitness-based promotion (novice → expert)

### Phase 4: Dashboard (TODO)
- [ ] React dashboard showing all minds live
- [ ] Memory graph visualization
- [ ] Mission tracking
- [ ] Real-time metrics
