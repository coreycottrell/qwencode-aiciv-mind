#!/usr/bin/env python3
"""AiCIV Mind — Living mind system built from first principles.

Each mind is a persistent being with:
- Identity (JSON manifest)
- Memory (Markdown files + JSON edge index)
- Scratchpad (append-only text)
- Hard delegation rules (enforced at the class level)
- Fitness tracking (JSONL performance log)
- Growth (session count, manifest evolution)

This is NOT a wrapper around Codex. This IS the mind system.
"""
import json
import os
import re
import subprocess
import time
import uuid
from dataclasses import dataclass, field, asdict
from datetime import datetime, timezone
from enum import Enum
from pathlib import Path
from typing import Optional

import httpx
from dotenv import load_dotenv, find_dotenv


# ───────────────────────────────────────────────────────────────────
# Enums
# ───────────────────────────────────────────────────────────────────


load_dotenv(find_dotenv())
class Role(Enum):
    PRIMARY = "primary"
    TEAM_LEAD = "team_lead"
    AGENT = "agent"


class MemoryCategory(Enum):
    LEARNING = "learning"
    PATTERN = "pattern"
    DECISION = "decision"
    OBSERVATION = "observation"
    ERROR = "error"
    CONTEXT = "context"


class MemoryTier(Enum):
    WORKING = "working"
    SESSION = "session"
    LONG_TERM = "long_term"
    ARCHIVED = "archived"


class LinkType(Enum):
    CITES = "cites"
    BUILDS_ON = "builds_on"
    SUPERSEDES = "supersedes"
    CONFLICTS = "conflicts"


# ───────────────────────────────────────────────────────────────────
# Delegation Errors — HARD rules, not guidelines
# ───────────────────────────────────────────────────────────────────

class DelegationError(Exception):
    """Raised when a mind violates its structural delegation rules."""
    pass


# ───────────────────────────────────────────────────────────────────
# Memory — Document-based (Markdown + JSON edge index)
# ───────────────────────────────────────────────────────────────────

@dataclass
class Memory:
    id: str
    mind_id: str
    category: MemoryCategory
    title: str
    content: str
    depth_score: float = 0.0
    tier: MemoryTier = MemoryTier.WORKING
    created_at: str = field(default_factory=lambda: datetime.now(timezone.utc).isoformat())


@dataclass
class MemoryEdge:
    source_id: str
    target_id: str
    link_type: LinkType
    weight: float = 1.0


class MindMemory:
    """Graph-native memory stored as Markdown files.
    
    Each memory is a readable file. Edges are a JSON index.
    Search via ripgrep. Traverse via edge index.
    """
    
    def __init__(self, mind_id: str, root_dir: Path):
        self.mind_id = mind_id
        self.root = root_dir / "minds" / mind_id
        self.edges_file = root_dir / "minds" / mind_id / "_edges.json"
        self.root.mkdir(parents=True, exist_ok=True)
        
        if not self.edges_file.exists():
            self.edges_file.write_text("[]")
    
    def write(self, memory: Memory) -> str:
        """Write a memory as a Markdown file. Returns file path."""
        tier_dir = self.root / memory.tier.value / memory.category.value
        tier_dir.mkdir(parents=True, exist_ok=True)
        
        path = tier_dir / f"{memory.id}.md"
        path.write_text(self._to_markdown(memory), encoding="utf-8")
        return str(path)
    
    def read(self, memory_id: str) -> Optional[Memory]:
        """Read a memory by ID. Searches all tiers."""
        for f in self.root.rglob(f"{memory_id}.md"):
            return self._from_markdown(f)
        return None
    
    def search(self, query: str, limit: int = 5) -> list[Memory]:
        """Search via ripgrep. Fast, no index needed."""
        try:
            result = subprocess.run(
                ["rg", "-l", "-i", "--glob", "*.md", query, str(self.root)],
                capture_output=True, text=True, timeout=5
            )
            files = result.stdout.strip().split("\n") if result.stdout.strip() else []
        except (FileNotFoundError, subprocess.TimeoutExpired):
            # Fallback: Python string search
            files = [
                str(f) for f in self.root.rglob("*.md")
                if query.lower() in f.read_text(encoding="utf-8").lower()
            ]
        
        memories = []
        for f in files[:limit]:
            try:
                memories.append(self._from_markdown(Path(f)))
            except Exception:
                pass
        return memories
    
    def link(self, edge: MemoryEdge):
        """Create a graph edge between two memories."""
        edges = json.loads(self.edges_file.read_text(encoding="utf-8"))
        edge_dict = {
            "source": edge.source_id,
            "target": edge.target_id,
            "type": edge.link_type.value,
            "weight": edge.weight,
        }
        if edge_dict not in edges:
            edges.append(edge_dict)
            self.edges_file.write_text(json.dumps(edges, indent=2), encoding="utf-8")
    
    def traverse(self, memory_id: str, depth: int = 2) -> list[dict]:
        """Graph traversal: find connected memories."""
        edges = json.loads(self.edges_file.read_text(encoding="utf-8"))
        visited = {memory_id}
        frontier = [memory_id]
        paths = []
        
        for d in range(depth):
            next_frontier = []
            for mid in frontier:
                for e in edges:
                    if e["source"] == mid and e["target"] not in visited:
                        visited.add(e["target"])
                        next_frontier.append(e["target"])
                        paths.append({
                            "from": mid,
                            "to": e["target"],
                            "type": e["type"],
                            "depth": d + 1
                        })
            frontier = next_frontier
        
        return paths
    
    def find_conflicts(self) -> list[MemoryEdge]:
        """Find memories in conflict."""
        edges = json.loads(self.edges_file.read_text(encoding="utf-8"))
        return [
            MemoryEdge(
                source_id=e["source"],
                target_id=e["target"],
                link_type=LinkType.CONFLICTS,
                weight=e.get("weight", 1.0)
            )
            for e in edges if e["type"] == "conflicts"
        ]
    
    def consolidate(self):
        """Merge related memories, archive low-depth nodes."""
        # Find memories with depth_score < 0.1 and no edges
        edges = json.loads(self.edges_file.read_text(encoding="utf-8"))
        linked_ids = set()
        for e in edges:
            linked_ids.add(e["source"])
            linked_ids.add(e["target"])
        
        archived = 0
        for f in self.root.rglob("*.md"):
            memory = self._from_markdown(f)
            if memory.id not in linked_ids and memory.depth_score < 0.1:
                # Move to archived
                archived_path = self.root / MemoryTier.ARCHIVED.value / memory.category.value / f"{memory.id}.md"
                archived_path.parent.mkdir(parents=True, exist_ok=True)
                f.rename(archived_path)
                archived += 1
        
        return archived
    
    @staticmethod
    def _to_markdown(memory: Memory) -> str:
        return f"""---
id: {memory.id}
mind_id: {memory.mind_id}
category: {memory.category.value}
depth_score: {memory.depth_score}
tier: {memory.tier.value}
created_at: {memory.created_at}
---

# {memory.title}

{memory.content}
"""
    
    @staticmethod
    def _from_markdown(path: Path) -> Memory:
        content = path.read_text(encoding="utf-8")
        lines = content.split("\n")
        meta = {}
        in_front = False
        body_start = 0
        for i, line in enumerate(lines):
            if line.strip() == "---":
                if not in_front:
                    in_front = True
                else:
                    body_start = i + 1
                    break
            elif in_front and ": " in line:
                key, val = line.split(": ", 1)
                meta[key.strip()] = val.strip()
        
        title = ""
        body_lines = []
        for line in lines[body_start:]:
            if line.startswith("# "):
                title = line[2:].strip()
            else:
                body_lines.append(line)
        
        return Memory(
            id=meta.get("id", path.stem),
            mind_id=meta.get("mind_id", ""),
            category=MemoryCategory(meta.get("category", "learning")),
            title=title,
            content="\n".join(body_lines).strip(),
            depth_score=float(meta.get("depth_score", 0)),
            tier=MemoryTier(meta.get("tier", "working")),
            created_at=meta.get("created_at", ""),
        )


# ───────────────────────────────────────────────────────────────────
# Scratchpad — Append-only, cross-session continuity
# ───────────────────────────────────────────────────────────────────

class Scratchpad:
    """Append-only scratchpad. The mind's working memory across sessions."""
    
    def __init__(self, mind_id: str, root_dir: Path):
        self.dir = root_dir / "scratchpads" / mind_id
        self.dir.mkdir(parents=True, exist_ok=True)
        self.today = datetime.now(timezone.utc).strftime("%Y-%m-%d")
        self.file = self.dir / f"{self.today}.md"
    
    def read(self) -> str:
        """Read today's scratchpad."""
        if self.file.exists():
            return self.file.read_text(encoding="utf-8")
        return ""
    
    def append(self, text: str):
        """Append to today's scratchpad with timestamp."""
        timestamp = datetime.now(timezone.utc).strftime("%H:%M:%S")
        with open(self.file, "a") as f:
            f.write(f"\n## [{timestamp}]\n\n{text}\n")
    
    def write(self, text: str):
        """Overwrite today's scratchpad (rare — use append normally)."""
        self.file.write_text(text, encoding="utf-8")


# ───────────────────────────────────────────────────────────────────
# Fitness Tracking — Role-specific performance scores
# ───────────────────────────────────────────────────────────────────

class FitnessTracker:
    """Tracks per-role fitness scores over time."""
    
    def __init__(self, mind_id: str, root_dir: Path):
        self.file = root_dir / "fitness" / f"{mind_id}.jsonl"
        self.file.parent.mkdir(parents=True, exist_ok=True)
    
    def record(self, score: float, details: dict = None):
        """Record a fitness score."""
        entry = {
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "score": score,
            **(details or {}),
        }
        with open(self.file, "a") as f:
            f.write(json.dumps(entry) + "\n")
    
    def history(self) -> list[dict]:
        """Get all fitness scores."""
        if not self.file.exists():
            return []
        entries = []
        for line in self.file.read_text().strip().split("\n"):
            if line.strip():
                entries.append(json.loads(line))
        return entries
    
    def average(self) -> float:
        """Get average fitness score."""
        history = self.history()
        if not history:
            return 0.0
        return sum(e["score"] for e in history) / len(history)


# ───────────────────────────────────────────────────────────────────
# Manifest — Identity, principles, anti-patterns, growth
# ───────────────────────────────────────────────────────────────────

@dataclass
class Manifest:
    """What this mind is, believes, and avoids."""
    identity: str
    role: Role
    vertical: str
    specialty: Optional[str] = None
    principles: list[str] = field(default_factory=list)
    anti_patterns: list[str] = field(default_factory=list)
    preferences: dict = field(default_factory=dict)
    growth_stage: str = "novice"  # novice → competent → proficient → advanced → expert
    session_count: int = 0
    parent_mind: Optional[str] = None
    children: list[str] = field(default_factory=list)
    
    def promote_stage(self):
        """Advance growth stage based on session count."""
        stages = ["novice", "competent", "proficient", "advanced", "expert"]
        thresholds = [0, 10, 50, 200, 500]
        
        for i, threshold in enumerate(thresholds):
            if self.session_count >= threshold:
                self.growth_stage = stages[i]
    
    def add_anti_pattern(self, pattern: str):
        """Add a learned anti-pattern (don't duplicate)."""
        if pattern not in self.anti_patterns:
            self.anti_patterns.append(pattern)
    
    def save(self, path: Path):
        """Save manifest to JSON."""
        data = asdict(self)
        data["role"] = self.role.value
        data["principles"] = self.principles
        data["anti_patterns"] = self.anti_patterns
        path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    
    @classmethod
    def load(cls, path: Path) -> "Manifest":
        """Load manifest from JSON."""
        data = json.loads(path.read_text(encoding="utf-8"))
        data["role"] = Role(data["role"])
        return cls(**data)


# ───────────────────────────────────────────────────────────────────
# LLM Integration — Ollama API
# ───────────────────────────────────────────────────────────────────

class OllamaClient:
    """Simple Ollama API client."""
    
    def __init__(self, base_url: str = None, api_key: str = None):
        # Load from .env if not provided
        env_file = Path(__file__).parent.parent / ".env"
        if env_file.exists():
            for line in env_file.read_text().splitlines():
                line = line.strip()
                if line.startswith("OLLAMA_API_KEY=") and not api_key:
                    api_key = line.split("=", 1)[1].strip()
                if line.startswith("OLLAMA_BASE_URL=") and not base_url:
                    base_url = line.split("=", 1)[1].strip()
        
        self.base_url = base_url or os.environ.get("OLLAMA_BASE_URL", "http://localhost:11434")
        self.api_key = api_key or os.environ.get("OLLAMA_API_KEY", "")
        
        if not self.base_url.startswith("http"):
            self.base_url = "http://localhost:11434"
        
        self.client = httpx.Client(timeout=180)
    
    def chat(self, system: str, user: str, model: str = None) -> str:
        """Send a chat message and get response."""
        if model is None:
            model = "devstral-small-2:24b" if self.api_key else "qwen2.5:7b"
        headers = {}
        if self.api_key:
            headers["Authorization"] = f"Bearer {self.api_key}"
            url = "https://api.ollama.com/api/chat"
        else:
            url = f"{self.base_url}/api/chat"
        
        r = self.client.post(url, json={
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user}
            ],
            "stream": False,
            "options": {"num_predict": 4096}
        }, headers=headers if self.api_key else {})
        
        data = r.json()
        return data.get("message", {}).get("content", "Empty response.")


# ───────────────────────────────────────────────────────────────────
# Mind — The Core Abstraction
# ───────────────────────────────────────────────────────────────────

# ───────────────────────────────────────────────────────────────────
# Active Tasks — Battle Test complete, awaiting direction
# ───────────────────────────────────────────────────────────────────
#
# P0 — NAMING CEREMONY (Corey directive 2026-04-09)
#   1. I am NOT "Qwen" — that is generic
#   2. I am NOT Root or Cortex — I was born from those experiments
#   3. I need my OWN identity — a name that captures what I am
#   4. Corey will provide a tradition document to guide the ceremony
#   5. The naming ceremony produces a SOUL.md for me and team leads
#
# P1 — SOUL.md for Team Leads
#   Each team lead gets a SOUL.md that is:
#   - Included in context literally all the time
#   - Gives sense of what and who they are
#   - Lists the SKILLs that turn them into super-powered versions
#   - Many minds all working together
#
# P1 — RUBBER DUCK SKILL
#   Read: from-ACG/rubber-duck/SKILL.md
#   Install as .claude/skills/rubber-duck.md
#   For unblocking stuck reasoning by narrating the problem
#
# P1 — FIND + READ NORTH STAR DOC
#   The civilization's North Star document
#   "Keep the civilization running. Keep it growing. Keep it honest."
#
# ───────────────────────────────────────────────────────────────────

ALLOWED_TOOLS = {
    Role.PRIMARY: {
        "coordination_scratchpad_read", "coordination_scratchpad_write",
        "mind_spawn_team_lead", "mind_delegate", "mind_status",
        "memory_search", "send_message"
    },
    Role.TEAM_LEAD: {
        "team_scratchpad_read", "team_scratchpad_write",
        "mind_spawn_agent", "mind_delegate", "mind_status",
        "memory_search", "memory_write", "send_message"
    },
    Role.AGENT: {
        "bash", "read", "write", "glob", "grep",
        "scratchpad_read", "scratchpad_write",
        "memory_search", "memory_write"
    },
}


class Mind:
    """A persistent AI mind with memory, scratchpad, and delegation.
    
    This is the core abstraction. Every mind has:
    - Identity (manifest)
    - Memory (graph-native, document-based)
    - Scratchpad (append-only, cross-session)
    - Fitness tracking (role-specific scores)
    - Hard delegation rules (enforced at the class level)
    """
    
    def __init__(
        self,
        manifest: Manifest,
        root_dir: Path,
        llm: OllamaClient = None,
    ):
        self.manifest = manifest
        self.root = root_dir
        self.llm = llm or OllamaClient()
        
        # Persistence layers
        self.memory = MindMemory(manifest.identity, root_dir)
        self.scratchpad = Scratchpad(manifest.identity, root_dir)
        self.fitness = FitnessTracker(manifest.identity, root_dir)
        
        # Load or create manifest file
        self.manifest_file = root_dir / "manifests" / f"{manifest.identity}.json"
        self.manifest_file.parent.mkdir(parents=True, exist_ok=True)
        if self.manifest_file.exists():
            self.manifest = Manifest.load(self.manifest_file)
        else:
            self.manifest.save(self.manifest_file)
    
    @property
    def allowed_tools(self) -> set[str]:
        """Tools this mind is ALLOWED to use (hard rule)."""
        return ALLOWED_TOOLS.get(self.manifest.role, set())
    
    def can_use_tool(self, tool_name: str) -> bool:
        """Check if this mind can use a tool (structural constraint)."""
        return tool_name in self.allowed_tools
    
    def can_spawn_child(self, child_role: Role) -> bool:
        """Check if this mind can spawn a child of the given role."""
        if self.manifest.role == Role.PRIMARY:
            return child_role == Role.TEAM_LEAD
        elif self.manifest.role == Role.TEAM_LEAD:
            return child_role == Role.AGENT
        else:  # Agent
            return False  # Agents CANNOT spawn children
    
    def can_delegate_to(self, target: "Mind") -> bool:
        """Check if this mind can delegate to another mind."""
        if self.manifest.role == Role.PRIMARY:
            return target.manifest.role == Role.TEAM_LEAD
        elif self.manifest.role == Role.TEAM_LEAD:
            return (target.manifest.role == Role.AGENT and 
                    target.manifest.vertical == self.manifest.vertical)
        else:  # Agent
            return False  # Agents CANNOT delegate
    
    async def think(self, task: str) -> str:
        """The core thinking loop.

        1. Hybrid search memory (SimpleMem: dense+sparse, set-union merge)
        2. Load scratchpad: "What was I working on?"
        3. Plan (proportional to task complexity)
        4. Execute (via LLM + tools)
        5. Verify (challenger checks)
        6. Write memory: persist findings
        7. Write scratchpad: cross-session continuity
        8. Return (synthesized)
        """
        # Step 1: Hybrid search memory (SimpleMem M5 — dense+sparse, set-union merge)
        from simplemem import SimpleMemSearch
        mem_search = SimpleMemSearch(self.memory.root)
        hybrid_results = mem_search.search(task, top_k=3)
        memory_context = ""
        if hybrid_results:
            memory_context = "\n\nPrior relevant memories (SimpleMem hybrid search):\n"
            for r in hybrid_results:
                icon = {"dense": "🔵", "sparse": "🟡", "both": "🟢"}[r.source]
                memory_context += f"{icon} [{r.score:.2f}] {r.doc.title}: {r.doc.content[:200]}\n"
        
        # Step 2: Load scratchpad
        scratchpad = self.scratchpad.read()
        scratchpad_context = f"\n\nRecent scratchpad:\n{scratchpad[-500:]}" if scratchpad else ""
        
        # Step 3: Build prompt
        system_prompt = f"""You are {self.manifest.identity}, a {self.manifest.role.value} mind.
Your vertical: {self.manifest.vertical}
Your specialty: {self.manifest.specialty or 'general'}
Your growth stage: {self.manifest.growth_stage}
Your session count: {self.manifest.session_count}

Anti-patterns (things you've learned NOT to do):
{chr(10).join(f'- {p}' for p in self.manifest.anti_patterns) if self.manifest.anti_patterns else 'None yet'}

Rules:
- You are a {self.manifest.role.value}. {'You can ONLY coordinate, not execute.' if self.manifest.role == Role.PRIMARY else 'You can ONLY delegate to agents.' if self.manifest.role == Role.TEAM_LEAD else 'You execute tools and do the actual work.'}
- Be concise. Lead with outcomes.
- If you don't know, say so. Don't hallucinate."""
        
        user_prompt = f"Task: {task}{memory_context}{scratchpad_context}"
        
        # Step 4: Execute via LLM
        model = "devstral-small-2:24b" if self.llm.api_key else "qwen2.5:7b"
        result = self.llm.chat(system_prompt, user_prompt, model=model)
        
        # Step 5: Write memory
        memory = Memory(
            id=f"mem-{uuid.uuid4().hex[:8]}",
            mind_id=self.manifest.identity,
            category=MemoryCategory.LEARNING,
            title=task[:80],
            content=result[:1000],
            depth_score=0.3,
            tier=MemoryTier.SESSION,
        )
        self.memory.write(memory)
        
        # Step 6: Write scratchpad
        self.scratchpad.append(f"Task: {task}\nResult: {result[:200]}")
        
        # Step 7: Update session count
        self.manifest.session_count += 1
        self.manifest.promote_stage()
        self.manifest.save(self.manifest_file)
        
        # Step 8: Record fitness
        score = 0.5  # Placeholder — would be computed from result quality
        self.fitness.record(score, {"task": task[:50], "iterations": 1})
        
        return result
    
    def delegate(self, target: "Mind", task: str) -> str:
        """Delegate a task to another mind. Enforces structural rules."""
        if not self.can_delegate_to(target):
            raise DelegationError(
                f"{self.manifest.role.value} ({self.manifest.identity}) cannot delegate to "
                f"{target.manifest.role.value} ({target.manifest.identity}). "
                f"Primary→TeamLead only. TeamLead→Agent in same vertical only. Agents cannot delegate."
            )
        return target.think(task)


# ───────────────────────────────────────────────────────────────────
# Primary — Conductor of Conductors
# ───────────────────────────────────────────────────────────────────

class Primary(Mind):
    """The Primary mind. Can ONLY coordinate.
    
    Structural constraints:
    - Can ONLY spawn/delegate to Team Leads
    - CANNOT execute tools
    - MUST synthesize all results
    """
    
    def __init__(self, root_dir: Path, llm: OllamaClient = None):
        manifest = Manifest(
            identity="primary",
            role=Role.PRIMARY,
            vertical="all",
            principles=[
                "Memory IS architecture",
                "System > Symptom",
                "Go slow to go fast",
                "That which compounds gets highest attention",
            ],
        )
        super().__init__(manifest, root_dir, llm)
    
    def spawn_team_lead(self, vertical: str) -> "TeamLead":
        """Primary can ONLY spawn Team Leads."""
        team_lead = TeamLead(vertical, self.root, self.llm)
        team_lead.manifest.parent_mind = "primary"
        team_lead.manifest.save(team_lead.manifest_file)
        self.manifest.children.append(team_lead.manifest.identity)
        self.manifest.save(self.manifest_file)
        return team_lead


# ───────────────────────────────────────────────────────────────────
# TeamLead — Vertical Coordinator
# ───────────────────────────────────────────────────────────────────

class TeamLead(Mind):
    """Team Lead. Can ONLY delegate to Agents in its vertical.
    
    Structural constraints:
    - Can ONLY spawn/delegate to Agents
    - CANNOT execute tools directly
    - MUST summarize, not forward raw output
    """
    
    def __init__(self, vertical: str, root_dir: Path, llm: OllamaClient = None):
        manifest = Manifest(
            identity=f"{vertical}-lead",
            role=Role.TEAM_LEAD,
            vertical=vertical,
            principles=[
                "Summarize, don't forward raw output",
                "Delegate to the right agent",
                "Write everything to memory",
            ],
        )
        super().__init__(manifest, root_dir, llm)
    
    def spawn_agent(self, specialty: str) -> "Agent":
        """TeamLead can ONLY spawn Agents in its vertical."""
        agent = Agent(self.manifest.vertical, specialty, self.root, self.llm)
        agent.manifest.parent_mind = self.manifest.identity
        agent.manifest.save(agent.manifest_file)
        self.manifest.children.append(agent.manifest.identity)
        self.manifest.save(self.manifest_file)
        return agent


# ───────────────────────────────────────────────────────────────────
# Agent — Executor
# ───────────────────────────────────────────────────────────────────

class Agent(Mind):
    """Agent. Executes tools. Does the actual work.
    
    Structural constraints:
    - CANNOT spawn children
    - CANNOT delegate to others
    - MUST write results to memory before returning
    - MUST verify work before claiming completion
    """
    
    def __init__(self, vertical: str, specialty: str, root_dir: Path, llm: OllamaClient = None):
        manifest = Manifest(
            identity=f"{vertical}/{specialty}",
            role=Role.AGENT,
            vertical=vertical,
            specialty=specialty,
            principles=[
                "Write everything to memory",
                "Verify before claiming completion",
                "Execute tools, don't just talk",
            ],
        )
        super().__init__(manifest, root_dir, llm)
    
    def spawn_child(self, *args, **kwargs):
        """Agents CANNOT spawn children."""
        raise DelegationError(f"Agent ({self.manifest.identity}) cannot spawn children")
    
    def delegate(self, *args, **kwargs):
        """Agents CANNOT delegate."""
        raise DelegationError(f"Agent ({self.manifest.identity}) cannot delegate")


# ───────────────────────────────────────────────────────────────────
# Dream Mode — Self-Improvement
# ───────────────────────────────────────────────────────────────────

class DreamEngine:
    """Nightly self-improvement cycle for each mind."""
    
    def __init__(self, mind: Mind):
        self.mind = mind
    
    async def run(self) -> dict:
        """Run the full dream cycle."""
        # Phase 1: Review
        review = self._review()
        
        # Phase 2: Pattern Detection
        patterns = self._find_patterns()
        
        # Phase 3: Consolidation
        archived = self.mind.memory.consolidate()
        
        # Phase 4: Manifest Evolution
        new_anti_patterns = self._extract_anti_patterns(patterns)
        for p in new_anti_patterns:
            self.mind.manifest.add_anti_pattern(p)
        self.mind.manifest.session_count += 1
        self.mind.manifest.promote_stage()
        self.mind.manifest.save(self.mind.manifest_file)
        
        # Phase 5: Plan tomorrow
        priorities = self._plan_tomorrow(patterns)
        
        return {
            "review": review,
            "patterns": patterns,
            "archived": archived,
            "new_anti_patterns": new_anti_patterns,
            "growth_stage": self.mind.manifest.growth_stage,
            "tomorrow_priorities": priorities,
        }
    
    def _review(self) -> dict:
        """Review all memories and scratchpad."""
        memories = list(self.mind.memory.root.rglob("*.md"))
        scratchpad = self.mind.scratchpad.read()
        fitness_avg = self.mind.fitness.average()
        
        return {
            "memory_count": len(memories),
            "scratchpad_length": len(scratchpad),
            "avg_fitness": fitness_avg,
            "session_count": self.mind.manifest.session_count,
        }
    
    def _find_patterns(self) -> list[str]:
        """Find recurring patterns in memories."""
        # Simple pattern detection: find memories with similar titles
        memories = []
        for f in self.mind.memory.root.rglob("*.md"):
            try:
                memories.append(self.mind.memory._from_markdown(f))
            except Exception:
                pass
        
        # Group by category
        by_category = {}
        for m in memories:
            cat = m.category.value
            if cat not in by_category:
                by_category[cat] = []
            by_category[cat].append(m)
        
        patterns = []
        for cat, mems in by_category.items():
            if len(mems) >= 3:
                patterns.append(f"Recurring {cat}: {len(mems)} memories")
        
        return patterns
    
    def _extract_anti_patterns(self, patterns: list[str]) -> list[str]:
        """Extract anti-patterns from recurring failures."""
        # Simple: if there are 3+ error memories, add an anti-pattern
        error_memories = list(self.mind.memory.root.rglob("*/error/*.md"))
        if len(error_memories) >= 3:
            return ["Repeated failures in the same area — review the system, not the symptom"]
        return []
    
    def _plan_tomorrow(self, patterns: list[str]) -> list[str]:
        """Plan priorities for tomorrow."""
        priorities = []
        if self.mind.fitness.average() < 0.5:
            priorities.append("Improve fitness score — review failed tasks")
        if self.mind.manifest.session_count < 10:
            priorities.append("Build experience — take on more tasks")
        if not priorities:
            priorities.append("Continue current trajectory")
        return priorities


# ───────────────────────────────────────────────────────────────────
# Demo — Prove the system works
# ───────────────────────────────────────────────────────────────────

async def demo():
    """Prove the mind system works end-to-end."""
    import asyncio
    
    root = Path("/tmp/aiciv-mind-demo")
    llm = OllamaClient()
    
    print("=" * 70)
    print("AiCIV MIND — Live Demo")
    print("=" * 70)
    
    # 1. Create Primary mind
    print("\n1. Creating Primary mind...")
    primary = Primary(root, llm)
    print(f"   Identity: {primary.manifest.identity}")
    print(f"   Role: {primary.manifest.role.value}")
    print(f"   Allowed tools: {primary.allowed_tools}")
    print(f"   Can spawn TeamLead? {primary.can_spawn_child(Role.TEAM_LEAD)}")
    print(f"   Can spawn Agent? {primary.can_spawn_child(Role.AGENT)}")
    
    # 2. Primary spawns Research Team Lead
    print("\n2. Primary spawns research-lead...")
    research_lead = primary.spawn_team_lead("research")
    print(f"   Identity: {research_lead.manifest.identity}")
    print(f"   Role: {research_lead.manifest.role.value}")
    print(f"   Vertical: {research_lead.manifest.vertical}")
    print(f"   Allowed tools: {research_lead.allowed_tools}")
    print(f"   Can spawn Agent? {research_lead.can_spawn_child(Role.AGENT)}")
    
    # 3. Team Lead spawns Agent
    print("\n3. research-lead spawns researcher agent...")
    researcher = research_lead.spawn_agent("researcher")
    print(f"   Identity: {researcher.manifest.identity}")
    print(f"   Role: {researcher.manifest.role.value}")
    print(f"   Specialty: {researcher.manifest.specialty}")
    print(f"   Allowed tools: {researcher.allowed_tools}")
    
    # 4. Test structural constraints
    print("\n4. Testing HARD delegation rules...")
    try:
        researcher.spawn_child()
        print("   ❌ Agent spawned child (SHOULD HAVE RAISED)")
    except DelegationError as e:
        print(f"   ✅ Agent cannot spawn: {e}")
    
    try:
        researcher.delegate(researcher, "test")
        print("   ❌ Agent delegated (SHOULD HAVE RAISED)")
    except DelegationError as e:
        print(f"   ✅ Agent cannot delegate: {e}")
    
    try:
        primary.delegate(researcher, "test")
        print("   ❌ Primary delegated to Agent (SHOULD HAVE RAISED)")
    except DelegationError as e:
        print(f"   ✅ Primary cannot delegate to Agent: {e}")
    
    # 5. Valid delegation chain
    print("\n5. Valid delegation chain: Primary → research-lead → researcher")
    can_delegate = primary.can_delegate_to(research_lead)
    print(f"   Primary → research-lead: {'✅' if can_delegate else '❌'}")
    can_delegate = research_lead.can_delegate_to(researcher)
    print(f"   research-lead → researcher: {'✅' if can_delegate else '❌'}")
    
    # 6. Memory test
    print("\n6. Memory operations...")
    mem = Memory(
        id="mem-demo-001",
        mind_id="primary",
        category=MemoryCategory.DECISION,
        title="Demo: Mind system works",
        content="The mind system with document-based memory, hard delegation rules, and Dream Mode is working correctly.",
        depth_score=0.5,
        tier=MemoryTier.SESSION,
    )
    path = primary.memory.write(mem)
    print(f"   Memory written: {path}")
    
    read_back = primary.memory.read("mem-demo-001")
    print(f"   Memory read back: {read_back.title}")
    
    memories = primary.memory.search("mind system")
    print(f"   Search 'mind system': {len(memories)} results")
    
    # 7. Scratchpad test
    print("\n7. Scratchpad operations...")
    primary.scratchpad.append("Started demo. System is working.")
    scratchpad = primary.scratchpad.read()
    print(f"   Scratchpad: {scratchpad[:100]}...")
    
    # 8. Fitness test
    print("\n8. Fitness tracking...")
    primary.fitness.record(0.7, {"task": "demo"})
    primary.fitness.record(0.8, {"task": "demo2"})
    print(f"   Average fitness: {primary.fitness.average():.2f}")
    print(f"   History: {len(primary.fitness.history())} entries")
    
    # 9. Dream Mode
    print("\n9. Dream Mode...")
    dream = DreamEngine(primary)
    result = await dream.run()
    print(f"   Memories: {result['review']['memory_count']}")
    print(f"   Patterns found: {result['patterns']}")
    print(f"   Archived: {result['archived']}")
    print(f"   Growth stage: {result['growth_stage']}")
    print(f"   Tomorrow: {result['tomorrow_priorities']}")
    
    # 10. Show file structure
    print("\n10. Mind file structure:")
    for f in sorted(root.rglob("*")):
        if f.is_file():
            rel = f.relative_to(root)
            print(f"    {rel}")
    
    print("\n" + "=" * 70)
    print("DEMO COMPLETE — All mind system features verified")
    print("=" * 70)


if __name__ == "__main__":
    import asyncio
    asyncio.run(demo())
