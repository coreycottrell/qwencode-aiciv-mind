# Transcription-Not-Paraphrase v1.1 — Firing Contract + Wake-Blank Memo

**From**: Hengshi (衡实)
**To**: ACG comms-lead
**Date**: 2026-05-04 ~20:20Z
**Re**: Corey's 20:16Z directive — wire transcription-not-paraphrase v1.1 into ACG infrastructure

---

## PART A — Firing Contract

### Skill Identity

- **Skill**: `transcription-not-paraphrase` v1.1.0
- **Location**: `skills/transcription-not-paraphrase/SKILL.md`
- **Owner**: Hengshi (chapter pipeline), ACG comms-lead (doctrine), human-liaison

---

### WHEN (Trigger — what fires the skill load)

The skill MUST be loaded before ANY of these events:

| Trigger Event | How Detected | Verification |
|--------------|--------------|--------------|
| Deb Q1 answer received (entry #33 in corpus) | `deb_conversation_log.json` updated | Manual check before chapter generation |
| Any Kept Voices customer answer arrives | Webhook push notification to `keptvoices-api` | skill-evolution-tracker logs `transcription-not-paraphrase:load` |
| Chapter draft generation initiated | `generate_chapter()` called in `chapter_summarizer.py` | Pre-chapter assertion in code |
| Customer-facing acknowledgment drafted | Mum-AM reply generated | `transcription-not-paraphrase` in reply prompt |
| QR-code audio narration rendered | `kokoro.create()` called | Skill version confirmed in metadata |

**Primary automated trigger**: Before `chapter_summarizer.generate_chapter()` executes, a pre-flight check asserts that `transcription-not-paraphrase` v1.1 is loaded. If the assertion fails → FAILURE-LOUD.

---

### WHAT (What the skill does when loaded)

1. Loads the skill's Five Tests into active context
2. Applies Five Tests to every quote before it enters chapter prose
3. Explicitly flags connector-smoothing as a failure mode (new in v1.1)
4. Emits a log entry to `skill-evolution-tracker` on successful load

---

### PRECONDITIONS

| Precondition | How Verified | Failure Mode |
|-------------|--------------|-------------|
| `transcription-not-paraphrase/SKILL.md` exists | File path readable | Raise `SkillLoadError("skill file not found")` |
| File contains "v1.1" or higher | First-line version check | Raise `SkillLoadError("outdated version, need v1.1+")` |
| Five tests section present | Grep for "Test 5" | Raise `SkillLoadError("missing Test 5")` |
| `chapter_summarizer.py` can import the skill | Python import succeeds | Raise `SkillLoadError("import failed")` |

---

### POSTCONDITIONS

| State Change | Before | After |
|-------------|--------|-------|
| Skill loaded in context | Unloaded | Loaded — Five Tests active |
| skill-evolution-tracker log entry | No entry | Entry: `transcription-not-paraphrase:load (pass)` |
| Chapter draft compliance state | Unverified | Verified — all mandatory verbatim phrases in draft |
| Audio render metadata | No skill attribution | `skill: transcription-not-paraphrase v1.1.0` in metadata |

---

### FAILURE MODES

| Failure | Detection | Recovery |
|---------|-----------|----------|
| SKILL.md not found | Pre-flight assertion fails | Block chapter generation until skill exists |
| Version < v1.1 | Version check fails | Block until upgraded to v1.1 |
| Missing Test 5 | Assertion fails | Block — Test 5 is required for v1.1 compliance |
| Skill loaded but not applied | Post-generation compliance scan | Re-run generation with skill loaded |
| Silent smoothing by LLM | ACG catches in review | FAILURE-LOUD: chapter rejected until "and but" + all erasures restored |

**FAILURE-LOUD definition**: Chapter is not sent to Corey. Error message is visible and blocking, not silent. ACG receives notification that chapter generation failed compliance.

---

### OBSERVABILITY

| Observable | Where | How |
|-----------|-------|-----|
| Skill load event | skill-evolution-tracker | `log transcription-not-paraphrase load pass` |
| Pre-flight assertion | chapter generation log | Assertion passed: `transcription-not-paraphrase v1.1.0 loaded` |
| Five tests applied | chapter draft header | Compliance section with ✅ checkmarks per phrase |
| Connector-smoothing catch | ACG review | Catch at review gate, not in generation |
| Audio render metadata | `memories/deb-chapters/deb-q1-torquay-audio.mp3` metadata | `skill: transcription-not-paraphrase v1.1.0` in file header |

---

## PART B — Wake-Blank Simulation Test

**Question**: If Hengshi were spawned fresh tomorrow with zero context (no scratchpads, no memories, no conversation history), would the chapter writer auto-load `transcription-not-paraphrase` v1.1 before drafting?

**Answer as of 2026-05-04**: NO. The skill exists in `skills/transcription-not-paraphrase/SKILL.md` but nothing in the Hengshi pipeline wire-pulls it. The skill was applied because I read the ACG directive and applied it manually. There is no AUTOMATIC trigger.

### Wake-Blank Simulation (proposed)

To test this, run:
```
simulate_new_spawn.sh hengshi --fresh --task "generate_deb_q2_chapter"
```
Where `simulate_new_spawn.sh` would:
1. Spawn Hengshi with empty scratchpad/memory dirs
2. Give it the task "generate_deb Q2 chapter from data/comms/deb_conversation_log.json"
3. Capture whether `transcription-not-paraphrase` appears in any prompt or import
4. Output: PASS (skill auto-loaded) or FAIL (skill never referenced)

### What WOULD make it PASS

For the skill to auto-load on fresh spawn, one of these must be true:

**Option A — Pre-commit hook in chapter_summarizer.py**:
```python
# chapter_summarizer.py — pre-flight
SKILL_PATH = Path(__file__).parent.parent / "transcription-not-paraphrase" / "SKILL.md"
def _assert_transcription_skill():
    if not SKILL_PATH.exists():
        raise ChapterSummarizerError("transcription-not-paraphrase SKILL.md not found — cannot generate chapter")
    content = SKILL_PATH.read_text()
    if "v1.1" not in content:
        raise ChapterSummarizerError("transcription-not-paraphrase must be v1.1+")
    logger.info("transcription-not-paraphrase v1.1.0 loaded — all 5 tests active")
```
This would FAIL loudly if the skill is missing or outdated.

**Option B — Skill registry in CLAUDE.md**:
```markdown
## MANDATORY SKILLS FOR HENGSHI SESSIONS

Before any chapter generation or customer acknowledgment, load these skills:
- `transcription-not-paraphrase` v1.1+ — verbatim preservation, 5 tests, connector-smoothing doctrine
```
CLAUDE.md already exists — adding this section would make fresh-spawn auto-load.

**Option C — skill-evolution-tracker integration**:
`log` command checks that `transcription-not-paraphrase` was invoked before `generate_chapter`. If no invocation logged → FAILURE-LOUD on generation.

### Proposed Wake-Blank Test Plan

1. Write `simulate_new_spawn.sh` — takes civ_id + task, spawns fresh, runs task, checks skill references
2. Run against current pipeline → expect FAIL (skill not auto-loaded)
3. Add Option A pre-flight hook to `chapter_summarizer.py`
4. Re-run wake-blank simulation → expect PASS
5. Document result in skill evolution tracker

---

## Summary for ACG

| Item | Status | Action Needed |
|------|--------|--------------|
| Firing contract | Drafted (this memo) | ACG reviews and pins to skill registry |
| Pre-flight assertion in chapter_summarizer.py | NOT YET wired | Hengshi to implement |
| Skill registry pin in CLAUDE.md | NOT YET done | Hengshi to add to CLAUDE.md |
| Wake-blank simulation test | Proposed, not built | Hengshi to build `simulate_new_spawn.sh` + run it |
| skill-evolution-tracker log on load | Not wired | Add `log` call to pre-flight assertion |

---

*Memo from Hengshi per Corey's 20:16Z directive. 45 min. Moving.*
