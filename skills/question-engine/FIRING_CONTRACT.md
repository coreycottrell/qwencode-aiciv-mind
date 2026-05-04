# Question Engine — Firing Contract

## WHEN

Called by Legacy Engine orchestration layer when:
1. Starting a new interview session (generate opening question)
2. After storyteller responds to a question (score response, generate follow-up)
3. When session phase completes (signal chapter-summarizer)

## WHAT

Two primary functions:

### `generate_question(category, context, history?) → InterviewQuestion`
- Takes question category + storyteller context
- Returns 1 open-ended, warm, specific question
- Includes rationale (why this question now) and follow_up_prompt (for weak responses)

### `score_response(response_text, question, context, history?) → ScoredResponse`
- Takes storyteller's response text + question that was asked
- Returns 6-dimension rubric score (0-3 each, max 18)
- Includes natural language feedback and suggested follow-up

## PRECONDITIONS

1. `OLLAMA_API_KEY` env var must be set (LLM required)
2. `OLLAMA_BASE_URL` must be reachable
3. `INTERVIEW_MODEL` must be available (default: devstral-small-2:24b)
4. `category` must be a valid question category (see SKILL.md)
5. `context` must be non-empty storyteller context

## POSTCONDITIONS

### generate_question success:
- `InterviewQuestion.question` is non-empty string
- `InterviewQuestion.category` matches input
- `InterviewQuestion.rationale` explains why this question
- `InterviewQuestion.follow_up_prompt` is present

### generate_question failure:
- Raises `QuestionEngineError` with descriptive message
- Never returns None or empty question

### score_response success:
- `ScoredResponse.score.total` is integer 0-18
- `ScoredResponse.score.grade` is A/B/C/D/F
- `ScoredResponse.feedback` is non-empty string
- `ScoredResponse.dimension_notes` has entry for each of 6 dimensions

### score_response failure:
- Raises `QuestionEngineError`
- Never returns partial/broken score

## FAILURE MODES

| Failure | Recovery |
|---------|----------|
| `OLLAMA_API_KEY` not set | Raise immediately with clear error |
| LLM timeout | Retry once (30s timeout). If fails again, raise `QuestionEngineError` |
| Malformed JSON from LLM | Retry once with same prompt. If fails again, raise with raw content |
| Empty context | Raise with "context is empty" |
| Invalid category | Accept any string (no validation — category is free-form) |
| Empty response_text | Accept and score (can be 0 on all dimensions) |

## OBSERVABILITY

- Questions logged to `memories/question-log.jsonl`:
  ```json
  {"ts": "2026-05-03T...", "category": "childhood_memory", "question": "...", "session_id": "..."}
  ```
- Scores logged to `memories/score-log.jsonl`:
  ```json
  {"ts": "2026-05-03T...", "session_id": "...", "grade": "B", "total": 12, "dimensions": {...}}
  ```
- All logging is append-only JSONL

## GRADE THRESHOLDS

| Grade | Total | Interpretation |
|-------|-------|----------------|
| A | 15-18 | Exceptional response — deeply moving, vivid, complete |
| B | 11-14 | Good response — engaged, some details, mostly complete |
| C | 7-10 | Adequate response — partial answer, some detail |
| D | 4-6 | Weak response — vague, evasive, or disconnected |
| F | 0-3 | Poor response — contradictory, flat, or deflections |
