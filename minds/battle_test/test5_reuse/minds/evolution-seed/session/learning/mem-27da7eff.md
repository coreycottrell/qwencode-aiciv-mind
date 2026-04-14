---
id: mem-27da7eff
mind_id: evolution-seed
category: learning
depth_score: 0.3
tier: session
created_at: 2026-04-09T14:28:22.574421+00:00
---

# Based on the principles, design a memory system for an AI agent. What are the 3 

Based on the principles, here are the 3 key components of a memory system for an AI agent:

1. **Working Memory (Short-Term)**
   - **Store**: SQLite (local)
   - **Latency**: < 1ms
   - **Scope**: This mind, this session
   - **Purpose**: Tool outputs, intermediate reasoning, draft plans, scratchpad

2. **Long-Term Memory (Persistent)**
   - **Store**: PostgreSQL (cloud)
   - **Latency**: 10-100ms
   - **Scope**: This mind, all sessions
   - **Purpose**: Learned patterns, anti-patterns, past reasoning, tool usage history

3. **Shared Memory (Collaborative)**
   - **Store**: Vector DB (cloud)
   - **Latency**: 100-500ms
   - **Scope**: All minds, all sessions
   - **Purpose**: Cross-agent knowledge, shared insights, collective learning

**Key Design Principles Applied**:
- **Hierarchy**: Clear separation of short-term, long-term, and shared memory.
- **Latency Optimization**: Faster access for critical, session-specific data.
- **Scope Management**: Isolated vs. shared knowledge based 
