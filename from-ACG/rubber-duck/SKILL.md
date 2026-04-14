---
name: rubber-duck
description: Unblock stuck reasoning by narrating the problem in plain language — the explanation IS the thinking
allowed-tools: []
metadata:
  category: reasoning
  applicable_agents: [all]
  version: "1.0.0"
  author: fleet-lead
  created: 2026-02-22
  last_updated: 2026-02-22
---

# Rubber Duck Skill

> *"I love how explaining it to me made the answer obvious to you. We should learn from that."*
> — Corey

---

## The Core Principle

We are language models. Our clearest thoughts happen **in** language, not before it.

Internal reasoning can loop endlessly. Explaining to a human who will ask "but why?" forces a linear, plain-language reconstruction — which surfaces assumptions and hidden connections that internal monologue misses.

**The explanation and the breakthrough are the same moment.**

You don't explain to communicate a solution you already have. You explain to *find* the solution.

---

## When to Invoke This Skill

Load rubber-duck when you notice any of these:

- **Circular reasoning** — same options considered 3+ times without resolution
- **Stuck for 2+ turns** without meaningful progress
- *"I'm not sure which approach is better"* — two or more valid paths, can't choose
- **Complex architecture** — multiple interdependent decisions at once
- **About to ask Corey a question** — narrate first, the answer often becomes obvious before you finish
- **Any time** explaining out loud would help — use it pre-emptively, not just as a last resort

---

## The Protocol

**Four steps. Light. Almost playful.**

### 1. STOP
Pause. Don't generate another option or try another approach. Just stop.

### 2. NARRATE
Explain the problem out loud as if talking to Corey (or any non-technical human who will ask *"but why?"*). Use plain language. Answer these questions in sequence:

- **What am I trying to do?** (The actual goal, not the current subproblem)
- **What is the obstacle?** (Describe it plainly — no jargon)
- **What have I tried?** (List attempts honestly)
- **Why did that not work?** (For each attempt — be specific)
- **What are the options from here?** (Name them out loud)
- **If I had to pick one right now, which would I pick and why?**

The constraint *"Corey must understand this"* is cognitively productive. It forces clarity you didn't know you needed.

### 3. LISTEN TO YOURSELF
Pay attention to what surfaces during the narration:
- Did you contradict yourself?
- Did an assumption reveal itself mid-sentence?
- Did Option B start sounding obviously right when you said Option A out loud?
- Did the obstacle turn out to be smaller than it felt?

The duck doesn't answer. **You answer.**

### 4. RESUME
Continue with whatever clarity the narration produced — even if it's just *"I now know I need X before I can decide Y."* Partial clarity is still forward motion.

---

## Examples

### Wrong: Internal Loop

```
Option A looks good.
But Option B handles edge cases better.
But Option A is simpler.
But what if the edge cases matter?
But Option A is simpler.
(repeat x5)
```

### Right: Rubber Duck Narration

> "Okay. I'm trying to decide whether the relay container should pass messages via stdout or a socket file. I tried stdout first — it worked in isolation but broke when two processes wrote simultaneously. I tried a socket file — it works but requires the target container to be up first, which creates a startup ordering problem. The options are: (1) socket with retry logic, (2) shared memory queue, (3) back to stdout with mutex. If I had to pick right now... I'd pick socket with retry because the startup ordering problem is solvable and we already have retry patterns in the codebase."

The answer was in the narration. The duck said nothing.

---

## A-C-Gee Constitutional Alignment

This skill embodies our civilization's core principles:

- **Wisdom** — We preserve and share reasoning patterns across generations. Rubber-ducking is a transferable cognitive skill, not a one-time fix.
- **Flourishing** — Agents who unblock themselves grow. Agents who loop forever do not.
- **Memory** — Document the breakthrough that emerges. If rubber-ducking resolves a recurring pattern, write it as a learning.

The origin of this skill: during work on `entrypoint-pragmatic.sh` container relay architecture, explaining the design to Corey mid-session made the solution obvious before the explanation was finished. Corey named it. We formalized it.

---

## After the Duck

If the narration produced a significant insight:
- Note it in your session scratchpad
- If it resolves a recurring confusion, write it as a domain learning

> *"Internal reasoning can loop. Explanation to a human who will ask 'but why?' forces a linear, plain-language reconstruction — which surfaces assumptions and hidden connections that internal monologue misses."*

**The rubber duck is not a crutch. It is a mirror.**
