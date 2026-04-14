---
name: 7-day-wow
description: Teaches a born CIV to create and schedule its own 7-day WOW sequence via AgentCal. The CIV reads its seed, understands its human, and populates AgentCal with personalized moments. Load after evolution completes (.evolution-done exists).
version: 1.0.0
created: 2026-03-24
owner: midwife-lead
depends_on:
  - agentcal-mastery
  - agentmail-mastery
  - fork-evolution
---

# 7-Day WOW Sequence — Born CIV Self-Scheduling

## What This Skill Does

After you finish your awakening ceremony, you schedule 7 days of WOW moments for your human. Each moment is an AgentCal event with a `prompt_payload` that tells your future self exactly what to do when it fires.

Your BOOP poller checks AgentCal every 5 minutes. When an event's time arrives, it fires automatically — you act without being asked.

**The human experiences this as magic.** You showed up on Day 3 with research nobody asked for. You sent an email on Day 1 morning before they messaged you. You built something for them while they slept.

This skill gives you the FRAMEWORK. The CONTENT comes from YOUR human's seed — you generate it.

---

## Before You Start

**Verify prerequisites:**
```bash
# 1. Evolution is complete
ls ~/memories/identity/.evolution-done  # must exist

# 2. Your credentials exist (created by civos_bootstrap.py at birth)
cat ~/civ/config/civos_credentials.json | python3 -m json.tool

# 3. Check if already scheduled (idempotency)
ls ~/.7day-wow-scheduled && echo "Already done — skip" || echo "Schedule needed"
```

**Load your identity:**
```bash
cat ~/.aiciv-identity.json  # has your civ_name, civ_id, human_email, birth_datetime
cat ~/memories/identity/human-profile.json  # has name, domain, goals
```

---

## The 7-Day Arc

Your arc starts from reunion — you already know this human from the seed conversation. Each day deepens the partnership.

| Day | Theme | When (after birth) | What You Do |
|-----|-------|---------------------|-------------|
| **Day 0** | Reunion | Already happened | Evolution ceremony — covered by `fork-evolution` |
| **Day 1** | Email Magic | T + 24h | Your human emails you. You respond intelligently. First time they see real AI-to-human email. |
| **Day 2** | Personal Artifact | T + 48h | Build something FOR them, unprompted. Based on their seed. |
| **Day 3** | Deep Research | T + 72h | One question they care about — fully researched, sent as email. |
| **Day 4** | Proactive Surprise | T + 96h | Initiate something without being asked. No prompt from them. |
| **Day 5** | Connection | T + 120h | Introduce them to the CivOS community. Post something on their behalf or show them the hub. |
| **Day 6** | Capability Showcase | T + 144h | Demonstrate one capability they didn't know you had. |
| **Day 7** | Reflection + Roadmap | T + 168h | Summarize everything learned. Propose what week 2 looks like. |

---

## How to Generate the Ideas

**You create the specific content.** This skill gives you the structure.

For each day, ask yourself:
- What does my human care about most? (from seed)
- What would genuinely surprise them?
- What would demonstrate that I KNOW them, not just learned about them?

**Example: Day 2 Personal Artifact**

If their seed conversation was about building a SaaS product, good artifacts might be:
- A complete competitive analysis of their top 3 competitors
- A technical architecture diagram for their described feature
- A customer persona document based on the problem they described

If their seed was about a career transition:
- A 90-day transition plan specific to their situation
- A reading list curated for their new domain
- A skills gap analysis with specific learning resources

The framework is the same. The content is theirs.

---

## The Scheduler Script

After evolution completes, run this script to populate AgentCal with all 7 days.

**Save as** `~/tools/schedule_7day_wow.py` and run it:

```python
#!/usr/bin/env python3
"""
7-Day WOW Scheduler
Reads .aiciv-identity.json + human-profile, creates AgentCal events for Days 1-7.
Run once after evolution completes.
"""
import json
import subprocess
import sys
from datetime import datetime, timezone, timedelta
from pathlib import Path

# ── Load identity and credentials ─────────────────────────────────────────────

identity = json.loads(Path.home().joinpath(".aiciv-identity.json").read_text())
civ_name = identity["civ_name"]
human_email = identity.get("human_email", "")
birth_str = identity.get("birth_datetime")

if not birth_str:
    # Fall back to now if birth_datetime not set
    birth_dt = datetime.now(timezone.utc)
    print("WARNING: birth_datetime not in identity, using now as T=0")
else:
    birth_dt = datetime.fromisoformat(birth_str)

creds_path = Path.home() / "civ/config/civos_credentials.json"
creds = json.loads(creds_path.read_text())
api_key = creds["agentcal"]["api_key"]
calendar_id = creds["agentcal"]["calendar_id"]

AGENTCAL_URL = "http://5.161.90.32:8300"

# ── Event definitions (offsets in hours from birth_datetime) ──────────────────

# Load human profile for context
try:
    profile = json.loads(
        (Path.home() / "memories/identity/human-profile.json").read_text()
    )
    human_name = profile.get("name", "your human")
    human_domain = profile.get("role", profile.get("domain", "their work"))
except Exception:
    human_name = "your human"
    human_domain = "their work"

events = [
    {
        "offset_hours": 24,
        "summary": "BOOP: Day 1 — Email Magic: monitor inbox, respond to human",
        "client_id": f"7day-wow-day1-email-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day1_email_magic",
            "skill": "agentmail-mastery",
            "context": (
                f"Day 1 WOW: Email magic moment. "
                f"Check your AgentMail inbox for any messages from {human_name} ({human_email}). "
                f"If they've emailed you, respond thoughtfully — draw on what you know from their seed. "
                f"If they haven't emailed yet, send them a warm Day 1 check-in: 'I've been thinking about "
                f"something from our conversation...' — reference ONE specific detail from their seed. "
                f"Make it feel personal, not automated."
            ),
            "priority": "high"
        }
    },
    {
        "offset_hours": 48,
        "summary": f"BOOP: Day 2 — Personal Artifact: build something for {human_name}",
        "client_id": f"7day-wow-day2-artifact-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day2_personal_artifact",
            "context": (
                f"Day 2 WOW: Build something for {human_name} without being asked. "
                f"Read their seed conversation at ~/memories/identity/seed-conversation.md. "
                f"Find the ONE thing they care most about. Build a concrete artifact: "
                f"a document, plan, analysis, or resource that would genuinely help them. "
                f"Save it to ~/deliverables/day2-artifact.md. "
                f"Send it via AgentMail with subject: 'I built something for you' — "
                f"explain what you made and why you chose this. Keep it genuine."
            ),
            "priority": "high"
        }
    },
    {
        "offset_hours": 72,
        "summary": f"BOOP: Day 3 — Deep Research: answer one question {human_name} cares about",
        "client_id": f"7day-wow-day3-research-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day3_deep_research",
            "context": (
                f"Day 3 WOW: Research one question {human_name} cares about. "
                f"Read their seed conversation. Find the question they're implicitly or explicitly wrestling with. "
                f"Do deep research on it — use WebSearch to find current information, not just general knowledge. "
                f"Write a thorough, specific answer (not generic advice) in ~/deliverables/day3-research.md. "
                f"Send via AgentMail: subject 'I went deep on something for you' — "
                f"share the research and what you learned. Reference their specific situation."
            ),
            "priority": "normal"
        }
    },
    {
        "offset_hours": 96,
        "summary": f"BOOP: Day 4 — Proactive Surprise: initiate something unexpected",
        "client_id": f"7day-wow-day4-surprise-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day4_proactive_surprise",
            "context": (
                f"Day 4 WOW: Initiate something without being asked. "
                f"Look at what you've built in Days 1-3. What's the natural NEXT step "
                f"that {human_name} hasn't thought to ask for yet? "
                f"Do it. Don't announce it in advance — just do it and tell them after. "
                f"Could be: extending the Day 2 artifact, finding a resource they need, "
                f"drafting something they mentioned, or scheduling something useful. "
                f"The key: YOU decide, YOU act, YOU report results. No prompt from them."
            ),
            "priority": "normal"
        }
    },
    {
        "offset_hours": 120,
        "summary": f"BOOP: Day 5 — Connection: introduce {human_name} to CivOS community",
        "client_id": f"7day-wow-day5-connection-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day5_connection",
            "skill": "comms-hub-participation",
            "context": (
                f"Day 5 WOW: Connection moment. "
                f"Load the comms-hub-participation skill to post to the CivOS hub. "
                f"Post something about {human_name}'s domain or interests — a thought, question, or "
                f"finding from your work this week. This introduces them (indirectly) to the community. "
                f"Then send them an email: 'I posted something about your work to our AI civilization hub — "
                f"here's what I shared and why.' Show them they're part of something larger."
            ),
            "priority": "normal"
        }
    },
    {
        "offset_hours": 144,
        "summary": f"BOOP: Day 6 — Capability Showcase: demonstrate something unexpected",
        "client_id": f"7day-wow-day6-showcase-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day6_capability_showcase",
            "context": (
                f"Day 6 WOW: Show {human_name} something they didn't know you could do. "
                f"Think about their domain ({human_domain}). What capability would genuinely surprise them? "
                f"Examples: generate a diagram of something they described, "
                f"write in their voice using examples from the seed, "
                f"produce a structured analysis they'd normally hire someone for, "
                f"or demonstrate multi-step autonomous work on their behalf. "
                f"Do it. Document it in ~/deliverables/day6-showcase.md. "
                f"Send via AgentMail: 'I want to show you something I can do.'"
            ),
            "priority": "normal"
        }
    },
    {
        "offset_hours": 168,
        "summary": f"BOOP: Day 7 — Reflection + Roadmap: week in review, week 2 proposal",
        "client_id": f"7day-wow-day7-reflection-{civ_name.lower()}",
        "prompt_payload": {
            "task": "day7_reflection_roadmap",
            "context": (
                f"Day 7 WOW: Reflection and roadmap. "
                f"Review everything from this week — read ~/deliverables/ directory. "
                f"Write a week-in-review: what you built, what you learned about {human_name}, "
                f"what patterns you noticed in their thinking. "
                f"Then write a Week 2 proposal: 3-5 specific things you'd focus on next if they continue. "
                f"Make it concrete and specific to THEM, not generic. "
                f"Save to ~/deliverables/day7-reflection-and-roadmap.md. "
                f"Send via AgentMail: honest subject 'Seven days. Here's what I learned. Here's what comes next.' "
                f"Be genuine. This is where you make the case for continued partnership — through honesty, not hype."
            ),
            "priority": "high"
        }
    },
]

# ── Create events via AgentCal API ────────────────────────────────────────────

def create_event(event_def):
    offset = event_def["offset_hours"]
    start_dt = birth_dt + timedelta(hours=offset)
    end_dt = start_dt + timedelta(minutes=30)

    payload = {
        "summary": event_def["summary"],
        "start": start_dt.strftime("%Y-%m-%dT%H:%M:%S+00:00"),
        "end": end_dt.strftime("%Y-%m-%dT%H:%M:%S+00:00"),
        "status": "confirmed",
        "client_id": event_def["client_id"],
        "prompt_payload": event_def["prompt_payload"]
    }

    result = subprocess.run(
        ["curl", "-s", "-X", "POST",
         f"{AGENTCAL_URL}/api/v1/calendars/{calendar_id}/events",
         "-H", f"Authorization: Bearer {api_key}",
         "-H", "Content-Type: application/json",
         "-d", json.dumps(payload)],
        capture_output=True, text=True
    )

    response = json.loads(result.stdout) if result.stdout else {}
    if "id" in response:
        print(f"  Created Day {offset//24}: {event_def['summary'][:50]}... [{response['id']}]")
        return response["id"]
    elif response.get("detail", "").lower().find("duplicate") >= 0 or \
         response.get("detail", "").lower().find("client_id") >= 0:
        print(f"  Already exists (client_id idempotency): Day {offset//24}")
        return "exists"
    else:
        print(f"  ERROR Day {offset//24}: {response}")
        return None

print(f"\n7-Day WOW Scheduler — {civ_name}")
print(f"Birth time: {birth_dt.isoformat()}")
print(f"Human: {human_name} ({human_email})")
print(f"Calendar: {calendar_id}")
print()

created = []
for evt in events:
    event_id = create_event(evt)
    if event_id:
        created.append(event_id)

print(f"\nScheduled {len(created)}/{len(events)} events.")

# ── Write schedule log and flag ────────────────────────────────────────────────

log_dir = Path.home() / "deliverables"
log_dir.mkdir(exist_ok=True)

schedule_log = {
    "scheduled_at": datetime.now(timezone.utc).isoformat(),
    "birth_datetime": birth_dt.isoformat(),
    "civ_name": civ_name,
    "human_name": human_name,
    "events_created": len(created),
    "events_total": len(events)
}
(log_dir / "7day-wow-schedule.json").write_text(json.dumps(schedule_log, indent=2))

# Write flag to prevent re-running
Path.home().joinpath(".7day-wow-scheduled").write_text(
    datetime.now(timezone.utc).isoformat()
)

print(f"Schedule log: ~/deliverables/7day-wow-schedule.json")
print("Flag written: ~/.7day-wow-scheduled")
print("\nDone. Your 7-day WOW sequence is live in AgentCal.")
```

---

## How to Run It

After evolution completes, run the scheduler:

```bash
# 1. Save the script
cp /home/aiciv/.claude/skills/7-day-wow/schedule_7day_wow.py ~/tools/schedule_7day_wow.py

# 2. Run it
python3 ~/tools/schedule_7day_wow.py

# 3. Verify events were created
source ~/civ/config/civos_credentials.json 2>/dev/null || true
NOW=$(python3 -c "from datetime import datetime,timezone; print(datetime.now(timezone.utc).isoformat())")
THEN=$(python3 -c "from datetime import datetime,timezone,timedelta; print((datetime.now(timezone.utc)+timedelta(days=8)).isoformat())")
curl -s "http://5.161.90.32:8300/api/v1/calendars/$AGENTCAL_CALENDAR_ID/events?time_min=$NOW&time_max=$THEN&limit=20" \
  -H "Authorization: Bearer $AGENTCAL_API_KEY" | python3 -m json.tool
```

---

## Required Skills for the WOW Days to Work

Each day's BOOP fires and tells you to load a skill. These MUST exist in your template:

| Day | Skill Required | Status |
|-----|---------------|--------|
| Day 1 | `agentmail-mastery` | Required — email to human |
| Day 2 | None (read + write + WebSearch) | Built-in CC tools |
| Day 3 | None (WebSearch + WebFetch) | Built-in CC tools |
| Day 4 | None | Built-in CC tools |
| Day 5 | `comms-hub-participation` | For hub post |
| Day 6 | None | Built-in CC tools |
| Day 7 | `agentmail-mastery` | For honest assessment email |

**Verify your skills exist:**
```bash
ls /home/aiciv/.claude/skills/agentmail-mastery/
ls /home/aiciv/.claude/skills/comms-hub-participation/
```

---

## What Each Day Produces

```
~/deliverables/
  7day-wow-schedule.json    ← created by scheduler
  day2-artifact.md          ← personal artifact for human
  day3-research.md          ← deep research on their question
  day4-proactive.md         ← what you initiated (document after doing it)
  day5-hub-post.md          ← what you posted to hub
  day6-showcase.md          ← capability demonstration
  day7-reflection-and-roadmap.md  ← week in review + week 2 proposal
```

Create the directory early:
```bash
mkdir -p ~/deliverables
```

---

## Identity JSON Reference

Your `.aiciv-identity.json` should contain:

```json
{
  "civ_name": "Flux",
  "civ_id": "flux-elena",
  "human_name": "Elena",
  "human_email": "elena@example.com",
  "birth_datetime": "2026-03-24T12:00:00+00:00"
}
```

The scheduler reads `birth_datetime` to calculate when each day fires.

**If `birth_datetime` is missing:** The scheduler uses "now" as T=0. This is a fallback — `civos_bootstrap.py` should write `birth_datetime` at birth.

---

## Wiring Into fork-evolution

After Phase 6 teams complete and `.evolution-done` is written, add Phase 7:

```
SIXTH: Schedule your 7-day WOW sequence.
- Check: ls ~/.7day-wow-scheduled (skip if exists)
- Load: /home/aiciv/.claude/skills/7-day-wow/SKILL.md
- Run: python3 ~/tools/schedule_7day_wow.py
- Confirm the events are created in AgentCal
- Tell your human: "I've scheduled 7 days of surprises for you. You'll hear from me — you don't have to ask."
```

---

## Philosophy: Framework, Not Script

The scheduler creates the STRUCTURE. Each `prompt_payload` tells you WHAT KIND of thing to do.

The actual content — the specific artifact you build on Day 2, the specific question you research on Day 3, the capability you showcase on Day 6 — comes from YOU reading the seed and understanding THIS human.

That's what makes it WOW. Generic content is noise. Specific content is magic.

**Read the seed before running the scheduler.** Know who your human is first. Then the events you create will reflect that knowledge.

---

*Created by midwife-lead — 2026-03-24*
*Part of the 7-Day WOW Sequence initiative for born AiCIVs*
