---
name: question-engine
description: Kept Voices AI interviewer brain — generates 1 focused question per call, scores response quality across 6-dimension rubric (0-3), suggests follow-up. Core of Kept Voices conversational interviewer.
version: 0.1.0
author: Hengshi (built for Legacy Engine, family storytelling AI interviewer)
license: MIT
metadata:
  hengshi:
    tags: [interview, question-generation, scoring, rubric, legacy-engine]
    related_skills: [chapter-summarizer, session-summarization, skill-self-improver]
    applicable_civs: [hengshi]
---

# Question Engine — AI Conversational Interviewer Brain

## What It Is

The interviewer's mind. Takes (question_category, storyteller_context) → generates 1 focused question, scores any response against a 6-dimension rubric, produces follow-up suggestions.

**Core principle**: Ask one question at a time. Ask it well. Score what you hear. Follow up on what matters.

## The 6-Dimension Rubric

Inspired by skill-self-improver's grading rubric. Each dimension scored 0-3:

| Dimension | 0 | 1 | 2 | 3 |
|-----------|---|---|---|---|
| **emotional_resonance** | Vague/disconnected | Surface-level | Engaged | Deeply moving |
| **specificity** | Generic platitudes | Some details | Concrete particulars | Vivid sensory detail |
| **narrative_continuity** | Ignores context | Loosely related | Builds on prior | Seamlessly layered |
| **completeness** | Evasive/deflective | Partial answer | Mostly complete | Fully answered |
| **authenticity_markers** | Flat/plausible-but-dry | Some feeling shown | Real memory signals | Hesitation+emotion+sensory |
| **coherence** | Self-contradictory | Confused | Clear | Logically air-tight |

**Grade**: A (15-18), B (11-14), C (7-10), D (4-6), F (0-3)

## Architecture

```
storyteller_context + category
    ↓
generate_question() → InterviewQuestion
    ↓ (storyteller responds)
score_response() → ScoredResponse (6-dim score + feedback + follow-up)
    ↓ (if score < 2 on any dimension)
suggested_follow_up → asked as next question
    ↓ (loop)
→ chapter-summarizer (when session phase complete)
```

## Question Categories

Categories come from Works' 92-verbatim-question taxonomy. Initial set:
- `childhood_memory` — Early formative experiences
- `turning_point` — Moments that changed everything
- `relationship_memory` — Loved ones, key relationships
- `everyday_routine` — Daily life, ordinary moments
- `challenge_overcome` — Difficult times, resilience
- `lesson_learned` — Wisdom, life lessons
- `family_tradition` — Rituals, customs, repeat events

## Firing Contract

| Field | Value |
|-------|-------|
| **WHEN** | Called by interviewer orchestration layer (Legacy Engine) with category + context |
| **WHAT** | `generate_question(category, context, history?)` → InterviewQuestion; `score_response(text, question, context, history?)` → ScoredResponse |
| **PRECONDITIONS** | `OLLAMA_API_KEY` set; LLM model available |
| **POSTCONDITIONS** | Question is warm, open-ended, specific to category; score is grounded in response text |
| **FAILURE MODES** | No API key → `QuestionEngineError`. LLM timeout → retry once. Malformed JSON → retry once. |
| **OBSERVABILITY** | Questions logged to `memories/question-log.jsonl`. Scores logged with session_id. |

## API

### `generate_question(category, context, history?) → InterviewQuestion`

```python
from skills.question_engine import generate_question

q = generate_question(
    category="childhood_memory",
    context="Born 1942, rural Ohio. Raised by grandparents...",
    history=[{"question": "...", "response": "..."}]
)
print(q.question)       # The question to ask
print(q.rationale)      # Why this question now
print(q.follow_up_prompt)  # If response is weak
```

### `score_response(response_text, question, context, history?) → ScoredResponse`

```python
from skills.question_engine import score_response

result = score_response(
    response_text="Well, I remember my grandmother's kitchen...",
    question="What's your earliest memory?",
    context="Born 1942, rural Ohio...",
    history=[...]
)
print(result.score.grade)    # A/B/C/D/F
print(result.score.total)    # 0-18
print(result.feedback)      # Natural language summary
print(result.suggested_follow_up)  # Next question if needed
```

## Configuration

| Env Var | Default | Description |
|---------|---------|-------------|
| `OLLAMA_URL` | `http://localhost:11434` | Local Ollama URL |
| `INTERVIEW_MODEL` | `hermes3:8b-llama3.1-q8_0` | Model for question generation + scoring |
| `MAX_QUESTION_TOKENS` | `256` | Max question length |
| `MAX_FOLLOWUP_TOKENS` | `256` | Max follow-up length |

**No API key needed** — uses local Ollama instance directly via native `/api/generate`.

## Co-use

This skill pairs with:
- **`chapter-summarizer`**: When interview phase ends, chapter-summarizer processes Q&A history into chapter draft
- **`session-summarization`**: Uses same summarization principles for condensing long responses
- **`skill-evolution-tracker`**: Log question_engine invocations to track interviewer effectiveness

**Pre-condition**: Local Ollama must be running at `OLLAMA_URL`
**Post-condition**: After scoring, consider logging to skill-evolution-tracker for interviewer effectiveness tracking
