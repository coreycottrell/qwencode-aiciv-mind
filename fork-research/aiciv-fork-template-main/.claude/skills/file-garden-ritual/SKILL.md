---
name: file-garden-ritual
description: Semantic composting protocol for categorizing files as Living/Dormant/Dead and extracting insights
version: 1.0.0
source: AI-CIV/${CIV_NAME} (FLOW-LIBRARY-INDEX.md)
allowed-tools: [Task, Read, Write, Grep, Glob, Bash]
agents-required: [code-archaeologist, pattern-detector, doc-synthesizer]
---

# The File Garden Ritual Skill

A semantic composting flow that categorizes files as Living, Dormant, or Dead based on activity and relevance. Dead files undergo "composting" - extracting any insights before safe removal. This prevents both file clutter and knowledge loss.

## When to Use

**Invoke when**:
- Quarterly cleanup cycle (scheduled maintenance)
- File count exceeds threshold (500+ files feeling cluttered)
- Before major refactoring or restructuring
- Knowledge archaeology needed (what wisdom is buried?)
- Repository feels stale or overgrown
- Preparing for migration or handoff

**Do not use when**:
- Active development (files in flux)
- No recent stabilization (wait for dust to settle)
- Emergency cleanup needed (use simpler deletion)
- Less than 100 files (overhead not worth it)

## Prerequisites

**Agents Required**:
- **code-archaeologist** - Analyzes file history and purpose
- **pattern-detector** - Identifies patterns across file categories
- **doc-synthesizer** - Extracts and preserves insights from Dead files
- **integration-auditor** - Verifies removal won't break dependencies

**Context Needed**:
- Git history for modification dates
- File system access for categorization
- `.trash/` directory for safe staging

## Procedure

### Step 1: Census
**Duration**: ~5-10 minutes
**Agent(s)**: The Conductor (automated)

Scan all files with modification metadata:

```bash
# Get all files with last modification time
find . -type f -not -path './.git/*' -printf '%T@ %p\n' | sort -n
```

1. Count total files
2. Collect last-modified timestamps
3. Note file types and locations

**Deliverable**: File census with timestamps

---

### Step 2: Categorization
**Duration**: ~15-20 minutes
**Agent(s)**: code-archaeologist + pattern-detector

Apply age-based categorization:

| Category | Criteria | Action |
|----------|----------|--------|
| Living | Modified in last 30 days | Keep, active development |
| Dormant | Modified 30-90 days ago | Review, may need attention |
| Dead | Not modified 90+ days | Compost (extract + archive) |

1. Sort files by category
2. Identify clusters (e.g., entire old feature dead)
3. Flag any Dead files that might have hidden value
4. Note files that seem miscategorized

**Deliverable**: Categorized file lists

---

### Step 3: Semantic Composting
**Duration**: ~20-30 minutes
**Agent(s)**: doc-synthesizer + code-archaeologist

Extract value from Dead files:

1. Read each Dead file
2. Ask: "What insight or pattern is here?"
3. Extract any reusable code, documentation, or wisdom
4. Document extracted insights
5. Clear file for removal

**Deliverable**: Extracted insights document

---

### Step 4: Archive & Cleanup
**Duration**: ~10 minutes
**Agent(s)**: integration-auditor

Safe removal process:

1. Verify Dead files have no live dependencies
2. Move to `.trash/` for review period (not immediate delete)
3. Update any references if needed
4. Document what was removed and why

```bash
# Move to trash (not delete)
mkdir -p .trash/$(date +%Y-%m-%d)
mv [dead-file] .trash/$(date +%Y-%m-%d)/
```

**Deliverable**: Cleanup complete, `.trash/` populated

---

### Step 5: Recommendations
**Duration**: ~5 minutes
**Agent(s)**: pattern-detector

Generate forward-looking guidance:

1. What patterns emerged from Dead files?
2. Are certain file types more likely to die?
3. Recommendations for preventing future clutter
4. Schedule next Garden Ritual

**Deliverable**: Cleanup recommendations

---

## Parallelization

**Can run in parallel**:
- Step 3 composting of individual files
- Multiple agents can extract from different Dead files

**Must be sequential**:
- Step 1 before 2 (census before categorization)
- Step 2 before 3 (categorization before composting)
- Step 3 before 4 (extraction before removal)

## Success Indicators

- [ ] All files categorized as Living/Dormant/Dead
- [ ] Semantic composting extracts insights from Dead files
- [ ] Dead files moved to `.trash/` (not immediately deleted)
- [ ] No live dependencies broken by removal
- [ ] Cleanup recommendations documented
- [ ] Collective feels "lighter" after ritual

## Example

**Scenario**: Quarterly cleanup of ${CIV_NAME} repository

```
Step 1 (Census): 514 files scanned
         Oldest: 2025-09-01
         Newest: 2025-10-05 (today)

Step 2 (Categorize):
         Living: 487 files (modified < 30 days)
         Dormant: 23 files (30-90 days)
         Dead: 4 files (> 90 days)

         Insight: Very few Dead - sign of healthy velocity!

Step 3 (Compost): 4 Dead files examined
         - old-config.yaml: No insights, safe to remove
         - draft-proposal.md: One good pattern extracted
         - test-scratch.py: Nothing valuable
         - legacy-notes.txt: Historical context preserved

Step 4 (Archive):
         4 files moved to .trash/2025-10-05/
         No dependency issues detected

Step 5 (Recommend):
         - Healthy repository (only 0.8% Dead)
         - Continue quarterly ritual
         - Consider more aggressive Dormant review

Result: Repository cleaned, 1 insight preserved, no knowledge lost
```

## Notes

- **Typical Duration**: 45-60 minutes for ~500 files
- **Error Handling**: If unsure about file, keep in Dormant (err toward preservation)
- **Evolution**: Consider automating categorization step
- **Key Insight**: High Living ratio (95%+) indicates healthy development velocity
- **The ".trash/" Safety Net**: Never delete directly; use staging directory
- **Semantic Composting**: The key differentiator - extract wisdom before disposal

---

**Converted from**: FLOW-LIBRARY-INDEX.md (Section 5: The File Garden Ritual)
**Original Status**: VALIDATED (2025-10-05)
**Conversion Date**: 2025-12-27
