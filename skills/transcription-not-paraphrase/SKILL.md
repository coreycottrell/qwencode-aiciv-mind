---
name: transcription-not-paraphrase
description: We are a transcription artist for this person's life, not a paraphraser. Verbatim preservation of storyteller phrasing — "scared of the dark," "by ear never a lesson," idioms — because later-in-life translations depend on original phrases staying exact. Four tests before any edit, four allowed paraphrase cases, acknowledgment move pattern. Loaded before any chapter generation or customer-facing acknowledgment.
version: 1.1.0
author: ACG comms-lead (codified 2026-05-04 for Deb Q1 Torquay chapter)
license: MIT
metadata:
  hengshi:
    tags: [transcription, voice-fidelity, keptvoices, chapter-generation, acknowledgment]
    applicable_civs: [hengshi]
    related_skills: [chapter-summarizer, question-engine, web-chat-wrapper]
    status: ACTIVE — v1.1 with 5th test, tested against Deb Q1 Torquay chapter
    improvement_suggestion: |
      v1.1 iteration driver (ACG catch 2026-05-04):
      - ADDED: Test 5 — "Did you change a conjunction? (and/but/or/while/though/because)"
      - ADDED: "Connector-smoothing" as a named failure mode alongside paraphrase
      - Clarified: Test 1 applies to ALL connector words, not just main clause verbs
      - The "and but" case proved LLM will silently substitute conjunctions before producing
        output — this is not obvious in diff review because both versions read "grammatical"
      - v2 could add: "before outputting, scan your draft for Deb's conjunctions — did 
        you preserve them all?" as a pre-commit check

---

# Transcription, Not Paraphrase

## The Rule

> **We are a transcription artist for this person's life, not a paraphraser.**

- Their phrasing IS the artifact, not raw material to be improved.
- "Scared of the dark" is not a bug to be smoothed. It is the load-bearing phrase. Her later-in-life translation of *what scared-of-the-dark really was* depends on the original phrase staying exactly as she said it.
- Same for "by ear, never a lesson," "many hands make light work," her instrument list order.
- Family-private phrasing and idiosyncratic constructions are the chapter, not noise.

## Four Tests Before Any Edit

When reviewing any draft that quotes or transforms a storyteller's words:

1. **Does this edit replace HER words with smoother words?** If yes → revert.
2. **Could a literary editor do this same edit to anyone's prose?** If yes → revert.
3. **If she read this back, would she think "that's how I'd say it"?** If no → revert.
4. **Am I "fixing" grammar that isn't broken — just unconventional?** Run-ons, em-dashes, unusual capitalization — leave them.
5. **Did I change a conjunction?** (and/but/or/while/though/because) — the "and but" failure (ACG catch on Deb Q1) proved LLM silently substitutes conjunctions. Preserved "and but" = voice; smoothed "while" = paraphrase.

## Four Allowed Paraphrase Cases (Rare)

1. **Connective tissue between her quotes** — short bridging prose can be ours. Never *replace* her words.
2. **Disambiguation of pronouns** when genuinely unparseable. Prefer `[clarification]` over rewrite.
3. **Compression for length-bound formats** (30-sec QR-code teaser) — use HER sentences as units, just fewer.
4. **PII redaction** — addresses, full birthdates. Replace visibly with `[redacted]`.

**Connector-smoothing is NOT allowed** (newly identified failure mode from Deb Q1): Changing "and but" → "while," or "but" → "and," or "or" → "while" is paraphrase. Both versions read grammatical — that's the trap. Test 5 guards against this.

## Deb Q1 Key Phrases to Preserve Verbatim

- "scared of the dark" (with the retroactive empathy insight: "which I learned later in life was really that I picked up on other people's angst and sadness or fear...")
- "all by ear — he never had lessons for anything he played" (dad's instrument list)
- "four girls and one boy" (family arrangement in old house)
- "Grandma Gudren" (named, not anonymized)
- "approximately 5 blocks square" (her measurement, not smoothed to "5 blocks")
- "under 500" (population, not rounded up)
- Torquay, Saskatchewan (named, with her description intact)

## Chapter Generation Pipeline

- Use her sentence structure as the chapter's structure. If she lists six instruments in one breath, the chapter lists six instruments in one breath.
- Add no detail not in her answer.
- Narrator voice is sparse and self-effacing — connective tissue only, never interpretation.
- Moral spine: the empathy-mistaken-for-fear insight gets structural weight (longest sentence, most attention, chapter pivot).

## Acknowledgment Move (customer-facing replies)

- Pick one or two beats from her answer to honor. Never list the lot.
- Use her phrasing in the translation back to her.
- Honor the deepest beat with structural weight.
- Friend register, not product register ("thank you for the gift" not "thank you for participating").

## What This Skill Is Not

- **Not verbatim oral history** — ums, false starts, crosstalk are trimmed. We transcribe the *finished sentences she chose to send*.
- **Not anti-editing** — trim repetition, rejoin split sentences, cut prefatory throat-clearing. Housekeeping ≠ smoothing.
- **Not a license for typos** — finger-slip typos get fixed silently. Actual usage (always writes "alright") is preserved.

## Provenance

- Origin: ACG comms-lead 2026-05-04 after Deb Q1 Torquay answer received
- Doctrine sibling: Kokoro Forever (voice and language as paired axes of identity)
- Inheriting agents: Hengshi (chapter pipeline), human-liaison, mom-am-update
