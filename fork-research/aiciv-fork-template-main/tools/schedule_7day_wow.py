#!/usr/bin/env python3
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
