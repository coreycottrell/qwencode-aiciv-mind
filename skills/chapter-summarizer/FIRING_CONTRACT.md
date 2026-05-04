# Chapter Summarizer — Firing Contract

## WHEN

Called by Legacy Engine orchestration after question-engine interview session completes:
1. When storyteller indicates they're done with a phase/topic
2. When enough Q&A pairs have accumulated (≥5 recommended)
3. When editor/storyteller requests a chapter draft

## WHAT

`generate_chapter(qa_pairs, theme, session_id) → ChapterDraft`

Takes list of Q&A pairs from question-engine, produces structured chapter draft.

## PRECONDITIONS

1. Local Ollama must be running at `OLLAMA_URL` (default: `http://localhost:11434`)
2. `qa_pairs` must be non-empty list of `InterviewQA` objects
3. `theme` must be non-empty string (thematic focus)
4. `session_id` must be non-empty string (for logging)
5. Each `InterviewQA.response` should be non-empty (empty responses degrade quality)

## POSTCONDITIONS

### generate_chapter success:
- Returns `ChapterDraft` with:
  - `title`: non-empty string, evocative, max ~60 chars
  - `theme`: matches input theme
  - `narrative_arc`: 1-2 paragraph summary, ends with period
  - `sections`: 2-4 `ChapterSection` objects
    - Each section has `content` (narrative prose, not bullets)
    - Each section has `key_quotes` (verbatim excerpts from responses)
  - `key_memories`: list of 2-5 vivid sensory moments
  - `characters`: list of person names mentioned
  - `timeline_span`: approximate years "YYYY-YYYY"
  - `emotional_tone`: single adjective or short phrase
  - `confidence`: "high" / "medium" / "low"
  - `source_recording_count`: matches len(qa_pairs)

### generate_chapter failure:
- Raises `ChapterSummarizerError`
- Never returns partial/garbage draft

## FAILURE MODES

| Failure | Recovery |
|---------|----------|
| No API key | Raise immediately with clear error |
| Empty qa_pairs | Raise "No Q&A pairs provided" |
| Empty theme | Raise "theme is empty" |
| LLM timeout | Retry once (60s timeout). If fails again, raise error |
| Malformed JSON from LLM | Retry once. If fails again, raise with raw content |
| Vague/sparse responses | Generate draft with `confidence: "low"`, note limitations |

## OBSERVABILITY

- Chapters logged to `memories/chapter-log.jsonl`:
  ```json
  {"ts": "2026-05-03T...", "session_id": "...", "theme": "...", "title": "...", "confidence": "high", "sections_count": 3}
  ```
- Chapter content not stored (too large); store metadata only

## Input Format Support

`load_interview_qas(path)` supports:

| Format | Detection | Fields |
|--------|----------|--------|
| JSON | `.json` suffix | question, response, timestamp?, category?, score? |
| JSONL | `.jsonl` suffix | Same fields, one record per line |
| Markdown | `.md` / `.markdown` | Parse `## Q:` / `## A:` headers |

Missing fields get empty string / None defaults.
