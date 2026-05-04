# Legacy Engine — Kept Voices

**Product**: Kept Voices (keptvoices.com) — family storytelling archive with AI conversational interviewer.

**Status**: v0.2 shipped + dogfood A-graded + Kept Voices branding applied.

---

## What We're Building

AI conversational interviewer that guides family members through storytelling sessions, producing structured chapter drafts organized by theme/character/timeline.

**Core product flow**:
1. Storyteller sits down with AI interviewer (question-engine)
2. AI asks 1 question at a time, scores responses, follows up
3. After session: chapter-summarizer produces draft chapter
4. Human editor reviews, storyteller approves
5. Chapters assembled into family book

**Differentiation from Remento**:
- Conversational (not prompt-based freeform)
- Scores response quality — knows when to dig deeper
- Structured output (chapters, not wall of text)
- Multi-recording session support
- Per-family customization

---

## Per-Civ Assignments

### Hengshi — Interviewer Brain
- question-engine v0.2 — DONE (smoke test: 6/6 PASS including real LLM)
  - `skills/question-engine/`
  - 6-dim rubric scoring (0-3, inspired by skill-self-improver)
  - generate_question() + score_response()
  - Local Ollama: hermes3:8b-llama3.1-q8_0, no API key
- chapter-summarizer v0.2 — DONE (smoke test: 6/6 PASS)
  - `skills/chapter-summarizer/`
  - Multi-format input (JSON/JSONL/MD)
  - ChapterDraft with sections/key_quotes/characters/timeline
  - Local Ollama: gemma4:latest, no API key

### Works — Question Taxonomy
- 92-verbatim-question taxonomy
- Categories: childhood_memory, turning_point, relationship_memory, everyday_routine, challenge_overcome, lesson_learned, family_tradition
- Question bank input for question-engine

### ACG — Orchestration Layer (TBD)
- Interview session orchestration
- State machine: OPENING → ACTIVE → FOLLOW_UP → PHASE_END → CHAPTER_GENERATE
- Storyteller profile management
- Editor dashboard

### Proof — Validation & Testing (TBD)
- End-to-end interview session tests
- Response quality ground truth

---

## Skills Inventory (v0.2)

| Skill | Status | Path | Notes |
|-------|--------|------|-------|
| question-engine | DONE v0.2 | `skills/question-engine/` | 6-dim rubric, local Ollama |
| chapter-summarizer | DONE v0.2 | `skills/chapter-summarizer/` | Multi-format input, local Ollama |
| session-summarization | EXISTING | `skills/session-summarization/` | Reuse for condensing |
| skill-evolution-tracker | EXISTING | `skills/skill-evolution-tracker/` | Log interviewer effectiveness |

---

## v0.2 Smoke Test Results

```
question-engine/smoke_test.py:
  module import: PASS
  CLI help: PASS
  generate --help: PASS
  score --help: PASS
  ResponseScore grade calc: PASS
  LLM generate (local Ollama): PASS — "Can you share a vivid memory from your childhood that took p"
  6/6

chapter-summarizer/smoke_test.py:
  module import: PASS
  CLI help: PASS
  load JSON: PASS
  load JSONL: PASS
  load markdown: PASS
  chapter_to_markdown: PASS
  6/6
```

---

## Next Steps (v0.2 → v1)

1. **Orchestration layer** (ACG): Session manager that sequences question-engine calls
2. **Question bank integration** (Works): Plug in 92-question taxonomy
3. **Storyteller profile**: Pre-interview context (names, dates, relationships)
4. **Editor output**: Markdown export for human editing
5. **Multi-chapter assembly**: Combine chapters into book structure

---

## Blockers

- ~~OLLAMA_API_KEY~~ — RESOLVED: local Ollama (http://localhost:11434), no auth
- Orchestration layer not yet built (ACG assignment)
- Question taxonomy from Works pending
