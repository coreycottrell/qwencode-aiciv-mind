---
name: chapter-summarizer
description: Kept Voices chapter producer — multi-recording Q&A to structured chapter draft with sections, key quotes, characters, timeline. Part of Kept Voices archive pipeline.
version: 0.1.0
author: Hengshi (built for Legacy Engine, family storytelling AI interviewer)
license: MIT
metadata:
  hengshi:
    tags: [chapter, summarization, narrative, legacy-engine, storytelling]
    related_skills: [question-engine, session-summarization]
    applicable_civs: [hengshi]
---

# Chapter Summarizer — Multi-Recording to Chapter Draft

## What It Is

Takes the output of question-engine interviews (Q&A pairs) and produces a structured chapter draft — the raw transcript becomes narrative prose with thematic framing, key quotes, and narrative arc.

**Core principle**: From conversation to chapter. Preserve verbatim quotes. Weave into narrative. Structure for reading, not for analysis.

## Architecture

```
question-engine interview sessions (Q&A pairs)
    ↓ (load_interview_qas — supports JSON/JSONL/MD)
list[InterviewQA]
    ↓ (generate_chapter — LLM call with theme + context)
ChapterDraft
    ↓ (chapter_to_markdown)
Markdown chapter file ready for editing
```

## Input Format

Loads Q&A pairs from JSON, JSONL, or markdown:

**JSON**: `[{"question": "...", "response": "...", "timestamp": "...", "category": "...", "score": 12}]`

**Markdown**: `## Q: ...` / `## A: ...` headers

## Output: ChapterDraft

```python
@dataclass
class ChapterDraft:
    title: str              # Evocative chapter title
    theme: str              # Thematic focus
    narrative_arc: str      # One-paragraph emotional journey summary
    sections: list[ChapterSection]
    key_memories: list[str]  # Most vivid sensory moments
    characters: list[str]    # People mentioned
    timeline_span: str      # "1952-1965"
    emotional_tone: str     # "Nostalgic", "Bittersweet"
    source_session_id: str
    source_recording_count: int
    confidence: str         # "high" / "medium" / "low"
```

## Firing Contract

| Field | Value |
|-------|-------|
| **WHEN** | After question-engine interview session completes. Called with Q&A history + chapter theme. |
| **WHAT** | `generate_chapter(qa_pairs, theme, session_id)` → `ChapterDraft` |
| **PRECONDITIONS** | Local Ollama running, non-empty qa_pairs list, non-empty theme |
| **POSTCONDITIONS** | ChapterDraft with ≥1 section, verbatim key_quotes, narrative prose (not bullets) |
| **FAILURE MODES** | Ollama unreachable → `ChapterSummarizerError`. No Q&A pairs → error. Malformed input → best-effort parse. |
| **OBSERVABILITY** | Generated chapters logged to `memories/chapter-log.jsonl` |

## Usage

```bash
python3 chapter_summarizer.py interview-qa.json --theme "Grandmother's kitchen" --session-id "smith-ch-1"
python3 chapter_summarizer.py session.md --theme "The day grandfather left" --format json
```

```python
from skills.chapter_summarizer import generate_chapter, load_interview_qas

qa_pairs = load_interview_qas("interview.json")
draft = generate_chapter(qa_pairs, "Family Sunday dinners", "session-1")
print(draft.title)  # "The Long Table"
print(draft.narrative_arc)  # One paragraph summary
for section in draft.sections:
    print(section.heading)  # Section titles
    print(section.content)  # Narrative prose
```

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `OLLAMA_URL` | `http://localhost:11434` | Local Ollama URL |
| `SUMMARIZATION_MODEL` | `gemma4:latest` | Model for chapter generation |
| `MAX_CHAPTER_TOKENS` | `2048` | Max chapter output |
| `CHAPTER_CACHE_ENABLED` | `true` | Cache chapters |
| `CHAPTER_CACHE_DIR` | `memories` | Cache directory |

**No API key needed** — uses local Ollama instance directly via native `/api/generate`.

## Co-use

This skill pairs with:
- **`question-engine`**: Consumes question-engine Q&A output as input; chapter-summarizer is the "output side" of the interview pipeline
- **`session-summarization`**: Shares summarization principles (truncation, fixed token budget, verbatim preservation)
- **`skill-evolution-tracker`**: Log chapter generation as skill invocation to track storytelling output volume

**Pre-condition**: Local Ollama must be running at `OLLAMA_URL`; Q&A pairs must be loaded via `load_interview_qas()`
**Post-condition**: Generated chapter logged to `memories/chapter-log.jsonl`
