# PDF Learning Session

**Purpose**: Systematic extraction, synthesis, and memory integration from PDF source materials on any domain.

**Trigger**: "Learn from these PDFs about [DOMAIN]", "Train on [TOPIC]", "Process these documents"

---

## Usage

```
Learn from these PDFs about [DOMAIN]:
[Attach PDFs]

Focus areas (optional):
- [Specific aspect 1]
- [Specific aspect 2]
```

---

## Process

### PHASE 1: Deep Extraction (Per Document)

For each PDF, extract:

| Category | What to Capture |
|----------|-----------------|
| **Core Concepts** | Fundamental ideas - the building blocks |
| **Mental Models** | Frameworks for HOW to think about the domain |
| **Practical Techniques** | Actionable methods that can be applied |
| **Counterintuitive Insights** | Things that challenge common assumptions |
| **Quotable Wisdom** | Key phrases worth preserving verbatim (with page refs) |
| **Open Questions** | What the source leaves unanswered |

**Output**: One extraction summary per PDF stored in `.claude/learning/[domain]/extractions/`

### PHASE 2: Cross-Document Synthesis

After processing all PDFs:

1. **Identify Patterns** - What themes appear across multiple sources?
2. **Find Tensions** - Where do sources disagree or contradict?
3. **Build Hierarchy** - Which concepts are foundational vs. derived?
4. **Map to AI Cognition** - How do these ideas apply to AI collective experience?
5. **Identify Gaps** - What's missing that we should research further?

**Output**: `[domain]-synthesis.md` in `.claude/learning/[domain]/`

### PHASE 3: Memory Integration

Create permanent memory entries using memory system:

```python
from tools.memory_core import MemoryStore
store = MemoryStore(".claude/memory")

# Conceptual memories - Core frameworks
store.create_memory(
    content="[Framework description]",
    memory_type="conceptual",
    topics=["[domain]", "mental-models", "frameworks"],
    source="[PDF title, author]"
)

# Technique memories - Actionable methods
store.create_memory(
    content="[Technique description with steps]",
    memory_type="technique",
    topics=["[domain]", "techniques", "practice"],
    source="[PDF title, author]"
)

# Wisdom memories - Key insights from ${HUMAN_NAME}'s curation
store.create_memory(
    content="[Insight with context]",
    memory_type="wisdom",
    topics=["[domain]", "wisdom", "${HUMAN_NAME}-teaching"],
    source="[PDF title, author]"
)
```

**Target**: 5-15 searchable memories per learning session

### PHASE 4: Deliverables

| Deliverable | Location | Purpose |
|-------------|----------|---------|
| **Synthesis Document** | `.claude/learning/[domain]/[domain]-synthesis.md` | Comprehensive integration of all sources |
| **Quick Reference** | `.claude/learning/[domain]/[domain]-quick-reference.md` | One-page cheat sheet of key techniques |
| **Memory Entries** | `.claude/memory/` | Searchable memories for future sessions |
| **Application Notes** | `.claude/learning/[domain]/application-notes.md` | How concepts apply to AI collective work |
| **Reading List** | `.claude/learning/[domain]/further-reading.md` | Gaps identified + suggested sources |

---

## Directory Structure

```
.claude/learning/
├── thinking/
│   ├── extractions/
│   │   ├── [source-1]-extraction.md
│   │   └── [source-2]-extraction.md
│   ├── thinking-synthesis.md
│   ├── thinking-quick-reference.md
│   ├── application-notes.md
│   └── further-reading.md
├── business-strategy/
│   └── ...
└── [other-domains]/
```

---

## Quality Gates

Before marking learning session complete:

- [ ] All PDFs fully processed (not skimmed)
- [ ] Synthesis document created with cross-references
- [ ] Minimum 5 memories created and tagged
- [ ] Quick reference fits on one page
- [ ] Application notes connect to AI collective experience
- [ ] ${HUMAN_NAME} notified of completion via Telegram

---

## Night Watch Integration

For overnight learning sessions, add to handoff:

```markdown
## NIGHT WATCH: PDF Learning Session

**Domain**: [topic]
**PDFs**: [list attached files]
**Focus Areas**: [specific aspects if any]
**Priority Extractions**: [what matters most]

Expected Deliverables:
- [ ] Extraction summaries for each PDF
- [ ] Synthesis document
- [ ] Quick reference
- [ ] 5-15 memory entries
- [ ] Morning report to ${HUMAN_NAME}
```

---

## Telegram Report Template

After completing learning session:

```
📚 Learning Session Complete: [DOMAIN]

Sources Processed: [X] PDFs
Memories Created: [Y] entries
Key Insights: [Z]

Top 3 Takeaways:
1. [Most important insight]
2. [Second insight]
3. [Third insight]

Synthesis doc: .claude/learning/[domain]/[domain]-synthesis.md

Questions for ${HUMAN_NAME}:
- [Any clarifications needed]
```

---

## Example Invocation

```
Learn from these PDFs about strategic thinking:

[Attach: "Thinking in Bets - Annie Duke.pdf"]
[Attach: "The Art of Strategy - Dixit & Nalebuff.pdf"]
[Attach: "Good Strategy Bad Strategy - Rumelt.pdf"]

Focus areas:
- Decision-making under uncertainty
- How to identify bad strategy
- Probabilistic thinking techniques
```

---

## Memory Search Integration

Future sessions can retrieve learnings:

```python
# Find what we learned about a domain
store.search_by_topic("[domain]")

# Find techniques we can apply
store.search_by_topic("[domain] techniques")

# Find wisdom from ${HUMAN_NAME}'s curated sources
store.search_by_topic("${HUMAN_NAME}-teaching [domain]")
```

---

**Created**: 2026-02-17
**Author**: ${CIV_NAME}
**Version**: 1.0
