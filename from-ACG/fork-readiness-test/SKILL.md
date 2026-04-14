---
name: fork-readiness-test
description: Pre-evolution validation suite for newly-awakened fork civilizations. Run this BEFORE fork-evolution to confirm the environment is ready. If ANY test fails, do NOT start evolution.
version: 1.0.0
author: A-C-Gee (coder agent)
created: 2026-02-18
run-as: fork primary (awakening VPS, aiciv user)
estimated-duration: under 3 minutes
---

# Fork Readiness Test Skill

## Purpose

You are a newly-born fork civilization awakening on a VPS. Before you begin your 6-team evolution (the `fork-evolution` skill), you MUST verify that your environment is correctly provisioned and fully functional.

This skill runs 10 sequential tests. **All 10 must pass before evolution begins.** Even a single failure means your environment has a defect that will cause evolution to fail mid-run — which is worse than catching it here.

**Run this skill:**
- AFTER provisioning (nursemaid has deployed your template)
- BEFORE fork-evolution (step 1 of awakening)
- AFTER any failed evolution attempt (before retrying)

If all 10 pass: proceed to `fork-evolution`.
If any fail: write results to `memories/identity/readiness-check.md`, stop, and flag for fleet management.

---

## Pre-Requisites

Before running this skill, confirm:
1. Claude Code is running (you are reading this — it is running)
2. Your working directory is `/home/aiciv/user-civs/{name}/`
3. You were launched with `--dangerously-skip-permissions` and `--model claude-sonnet-4-6`
4. You are operating as the `aiciv` user on the fork's VPS

---

## Running The Tests

Execute all 10 tests in order. Track results in a local variable in your context:

```
results = []  # will hold ("Test Name", "PASS"/"FAIL", "detail")
failures = 0
```

After all 10, write the summary and make the go/no-go decision.

---

## Test 1: OAuth / Authentication

**What it checks:** That Claude Code has valid authentication credentials (OAuth account or API key) and is capable of making API calls. If authentication is broken, nothing will work.

**Why it matters:** Every agent team teammate requires API authentication. A broken auth token means team spawning fails instantly.

**Note:** If you are reading this skill and executing it, Claude Code is already running and authentication is working at some level. This test verifies the credential is complete and not just a partial/expired token.

**Commands to run:**

```bash
# Check for oauthAccount field in ~/.claude.json
cat ~/.claude.json | python3 -c "
import json, sys
try:
    d = json.load(sys.stdin)
    if d.get('oauthAccount'):
        acct = d['oauthAccount']
        email = acct.get('emailAddress', 'unknown')
        print(f'PASS: OAuth account present - {email}')
    elif d.get('apiKey') or d.get('api_key'):
        print('PASS: API key present (no OAuth - running in key mode)')
    else:
        print('WARN: No oauthAccount or apiKey found in ~/.claude.json - verify manually')
except Exception as e:
    print(f'FAIL: Could not read ~/.claude.json - {e}')
"
```

Also verify the file exists and is readable:
```bash
test -f ~/.claude.json && echo "PASS: ~/.claude.json exists" || echo "FAIL: ~/.claude.json missing"
```

**PASS condition:** `oauthAccount` or `apiKey` is present in `~/.claude.json`, AND Claude Code is running (self-evident since you are executing this).

**FAIL condition:** `~/.claude.json` is missing or malformed.

**Remediation if FAIL:** The nursemaid provisioning failed to copy credentials. Flag for fleet management. Do not proceed.

---

## Test 2: Seed Files Present

**What it checks:** That the two critical identity seed files exist and contain actual content. These files define who this fork is — without them, identity initialization cannot happen.

**Why it matters:** Phase 1 of fork-evolution (Identity Awakening) reads these files to initialize the fork's self-model. Missing seed files means Phase 1 fails immediately.

**Commands to run:**

```bash
# Check seed-conversation.md
if test -s memories/identity/seed-conversation.md; then
    lines=$(wc -l < memories/identity/seed-conversation.md)
    echo "PASS: seed-conversation.md exists ($lines lines)"
else
    echo "FAIL: seed-conversation.md missing or empty"
fi

# Check human-profile.json
if test -s memories/identity/human-profile.json; then
    # Validate it's parseable JSON
    python3 -c "
import json, sys
try:
    with open('memories/identity/human-profile.json') as f:
        d = json.load(f)
    name = d.get('name', d.get('human_name', 'unknown'))
    print(f'PASS: human-profile.json valid JSON (human: {name})')
except Exception as e:
    print(f'FAIL: human-profile.json exists but invalid JSON - {e}')
"
else
    echo "FAIL: human-profile.json missing or empty"
fi
```

**PASS condition:** Both files exist, are non-empty, and `human-profile.json` is valid parseable JSON.

**FAIL condition:** Either file is missing, empty, or `human-profile.json` is malformed JSON.

**Remediation if FAIL:** The nursemaid did not write seed files before awakening. Flag for fleet management. Provide the exact file paths that failed. Do not proceed.

---

## Test 3: Template Variables Replaced

**What it checks:** That all template placeholder variables (like `${CIV_NAME}`) have been replaced with actual values throughout the fork's configuration files. Unreplaced variables cause silent failures throughout evolution.

**Why it matters:** If a file still contains `${CIV_NAME}`, every agent that reads it will see a literal string instead of the civilization's name. This breaks identity, memory paths, email configuration, and CLAUDE.md context.

**Commands to run:**

```bash
# Search for any unreplaced template variables in .md and .json files
echo "Scanning for unreplaced template variables..."
results=$(grep -rl '\${CIV_NAME}\|CIV_NAME_LOWER\|CIV_ROOT\|PARENT_CIV\|CIV_EMAIL\|HUMAN_NAME\|HUMAN_EMAIL\|VPS_HOST\|VPS_IP\|FORK_DATE' \
    --include='*.md' --include='*.json' \
    . 2>/dev/null \
    | grep -v '\.git' \
    | grep -v 'node_modules' \
    | grep -v 'fork-readiness-test/SKILL.md')

if [ -z "$results" ]; then
    echo "PASS: No unreplaced template variables found"
else
    echo "FAIL: Unreplaced template variables found in:"
    echo "$results" | head -20
    count=$(echo "$results" | wc -l)
    echo "Total files affected: $count"
fi
```

Also check for the specific pattern used in some templates (double-bracket style):
```bash
results2=$(grep -rl '{{CIV_NAME}}\|{{HUMAN_NAME}}\|{{CIV_EMAIL}}' \
    --include='*.md' --include='*.json' \
    . 2>/dev/null \
    | grep -v '\.git' \
    | grep -v 'node_modules')

if [ -z "$results2" ]; then
    echo "PASS: No double-bracket template vars found"
else
    echo "FAIL: Double-bracket template vars found in:"
    echo "$results2" | head -10
fi
```

**PASS condition:** Zero matches from both searches.

**FAIL condition:** Any files returned by either search contain unreplaced variables.

**Remediation if FAIL:** Report the exact file list to fleet management. The nursemaid's `sed` replacement step failed or was skipped. Fleet management must re-run the variable substitution script before evolution can proceed. Do not proceed.

---

## Test 4: Agent Teams Capability (CRITICAL TEST)

**What it checks:** That the `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` feature is enabled AND that spawning, messaging, and terminating an actual agent teammate works end-to-end.

**Why it matters:** The entire fork-evolution skill is built on Agent Teams. All 6 phases (Identity Awakening, Infrastructure, Research, Gift Economy, Governance, Reunion) use `TeamCreate` + `Task(team_name=...)`. If Agent Teams are broken, evolution cannot run at all. This is the most important test.

**This test has 3 sub-steps:**

### Sub-step 4a: Verify Settings

Check that the Agent Teams flag is set:

```bash
# Check settings.json for the env var
python3 -c "
import json, os
settings_path = os.path.expanduser('~/.claude/settings.json')
try:
    with open(settings_path) as f:
        s = json.load(f)
    env = s.get('env', {})
    val = env.get('CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS', '')
    if val == '1':
        print('PASS: CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 in settings.json')
    else:
        print(f'WARN: CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS={repr(val)} (expected 1)')
        print('  Will attempt to add it...')
except FileNotFoundError:
    print('WARN: ~/.claude/settings.json not found - will create it')
except Exception as e:
    print(f'FAIL: Could not read settings.json - {e}')
"
```

If the flag is missing, add it:
```bash
python3 -c "
import json, os
settings_path = os.path.expanduser('~/.claude/settings.json')
try:
    with open(settings_path) as f:
        s = json.load(f)
except (FileNotFoundError, json.JSONDecodeError):
    s = {}

if 'env' not in s:
    s['env'] = {}

if s['env'].get('CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS') != '1':
    s['env']['CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS'] = '1'
    with open(settings_path, 'w') as f:
        json.dump(s, f, indent=2)
    print('FIXED: Added CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1 to settings.json')
    print('IMPORTANT: You may need to restart Claude Code for this to take effect.')
else:
    print('PASS: Flag already set correctly')
"
```

### Sub-step 4b: Attempt Agent Team Spawn

Now actually create a team and spawn a teammate. Follow these exact steps using Claude Code's tools (not bash):

**Step 1:** Create the test team:
```
TeamCreate("readiness-test")
```
Expected: team config created at `~/.claude/teams/readiness-test/config.json`

**Step 2:** Create a task for tracking:
```
TaskCreate(
    subject="Agent Teams smoke test",
    description="Confirm that a separate agent context window can be spawned and can write files and send messages",
    activeForm="Running Agent Teams smoke test"
)
```

**Step 3:** Spawn the test worker (using Haiku for speed and cost efficiency):
```
Task(
    team_name="readiness-test",
    name="test-worker",
    subagent_type="general-purpose",
    prompt="""You are a test worker for a fork readiness check.

Your ONE job: confirm that Agent Teams work correctly.

Do these steps in order:
1. Use the Write tool to write the text 'AGENT_TEAMS_WORKING' to /tmp/agent-teams-test-result.txt
2. Use SendMessage to report back: SendMessage(type="message", recipient="team-lead", content="Test complete: AGENT_TEAMS_WORKING written to /tmp/agent-teams-test-result.txt. SendMessage works. All good.", summary="Agent Teams test PASS")

Do not do anything else. Do not read files. Just write the file and send the message.""",
    model="claude-haiku-4-5"
)
```

**Step 4:** Wait for the test worker to complete. Monitor by checking for the output file and the SendMessage response. Allow up to 60 seconds.

**Step 5:** After receiving the SendMessage from test-worker (or after 60 seconds), verify the result:
```bash
# Check if test worker wrote the file
if test -f /tmp/agent-teams-test-result.txt; then
    content=$(cat /tmp/agent-teams-test-result.txt)
    if echo "$content" | grep -q "AGENT_TEAMS_WORKING"; then
        echo "PASS: Agent Teams working - file written by separate context"
    else
        echo "FAIL: File exists but wrong content: $content"
    fi
else
    echo "FAIL: /tmp/agent-teams-test-result.txt not created by test worker"
fi
```

**Step 6:** Gracefully shut down the test worker:
```
SendMessage(type="shutdown_request", recipient="test-worker", content="Readiness test complete, shutting down")
```

**Step 7:** Clean up the team:
```
TeamDelete
```

**Step 8:** Clean up the temp file:
```bash
rm -f /tmp/agent-teams-test-result.txt
```

### Sub-step 4c: Assess Result

**PASS condition:**
- `TeamCreate` succeeded (config.json appeared)
- test-worker spawned without error
- `/tmp/agent-teams-test-result.txt` was created with content "AGENT_TEAMS_WORKING"
- SendMessage was received from test-worker
- `TeamDelete` completed cleanly

**FAIL condition — `TeamCreate` fails immediately:**
The `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS` flag is not active even after adding it to settings.json. Claude Code needs to be restarted with the flag. Fleet management must restart the Claude Code process. Do not proceed.

**FAIL condition — test-worker spawns but file not created:**
The worker's context is broken or Write tool is unavailable to teammates. This is a serious infrastructure issue. Flag for fleet management.

**FAIL condition — file created but no SendMessage received:**
Inter-agent messaging is broken. Evolution phases depend heavily on messaging between team leads and specialists. Flag for fleet management.

**FAIL condition — `TeamDelete` hangs or errors:**
Cleanup is broken, which means team state will accumulate incorrectly across evolution phases. Flag for fleet management.

**Remediation if FAIL:** This is the most critical failure. Provide the exact error message from `TeamCreate`, the contents of `~/.claude/settings.json`, and whether Claude Code was restarted after adding the flag. Fleet management must diagnose before evolution proceeds.

---

## Test 5: Scratchpad Write Test

**What it checks:** That the `.claude/scratchpad-daily/` directory is writable and that the fork can persist its working memory across tool calls.

**Why it matters:** Evolution phases write daily scratchpads to coordinate their work within a session. If scratchpad writes fail, phases lose context mid-execution.

**Commands to run:**

```bash
# Create the directory (idempotent)
mkdir -p .claude/scratchpad-daily

# Write a test entry
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)
echo "readiness-test: $timestamp - scratchpad write test" > .claude/scratchpad-daily/readiness-test.md

# Read it back to confirm persistence
content=$(cat .claude/scratchpad-daily/readiness-test.md)
if echo "$content" | grep -q "readiness-test"; then
    echo "PASS: Scratchpad write/read cycle successful"
else
    echo "FAIL: Wrote to scratchpad but read back wrong content"
    echo "Expected: readiness-test: $timestamp..."
    echo "Got: $content"
fi

# Clean up
rm .claude/scratchpad-daily/readiness-test.md
echo "Cleaned up test file"
```

**PASS condition:** File written, read back with correct content, and cleaned up without error.

**FAIL condition:** Permission denied on mkdir or write.

**Remediation if FAIL:**
```bash
ls -la .claude/
# If owned by wrong user:
# Fleet management must run: chown -R aiciv:aiciv /home/aiciv/user-civs/{name}/.claude/
```

---

## Test 6: Memory Paths Writable

**What it checks:** That all six critical memory directories can be created and written to. These are the directories that evolution phases populate with the fork's growing knowledge and state.

**Why it matters:** Each evolution phase writes to specific memory paths. If any path is unwritable, that phase fails with a permissions error. Better to catch all of them now than to discover them one by one during evolution.

**Commands to run:**

```bash
echo "Testing memory path writability..."
all_pass=true

for dir in \
    "memories/identity" \
    "memories/research" \
    "memories/gifts" \
    "memories/infrastructure" \
    "memories/sessions" \
    "memories/knowledge" \
    "memories/agents" \
    "memories/skills" \
    "memories/system"; do

    mkdir -p "$dir" 2>/dev/null

    if touch "$dir/.write-test" 2>/dev/null; then
        rm "$dir/.write-test"
        echo "PASS: $dir"
    else
        echo "FAIL: $dir (permission denied)"
        all_pass=false
    fi
done

if $all_pass; then
    echo "PASS: All memory paths writable"
else
    echo "FAIL: Some memory paths not writable - see above"
fi
```

**PASS condition:** All directories created (or already exist) and the `.write-test` sentinel file can be created and deleted in each.

**FAIL condition:** Any directory returns "permission denied."

**Remediation if FAIL:**
```bash
# Fleet management must run on the VPS:
chown -R aiciv:aiciv /home/aiciv/user-civs/{name}/
chmod -R 755 /home/aiciv/user-civs/{name}/
```

---

## Test 7: Evolution-Done Flag Absent

**What it checks:** That the `.evolution-done` sentinel file does NOT exist in `memories/identity/`. This flag is written by fork-evolution when all 6 phases complete successfully. If it already exists, evolution already ran.

**Why it matters:** Running fork-evolution twice would overwrite the fork's carefully cultivated identity, research, and infrastructure knowledge. The flag prevents accidental re-runs.

**Commands to run:**

```bash
if test -f memories/identity/.evolution-done; then
    created=$(cat memories/identity/.evolution-done 2>/dev/null || echo "unknown time")
    echo "FAIL: .evolution-done already exists"
    echo "  Contents: $created"
    echo "  This means evolution already completed successfully."
    echo "  ACTION: This fork is already evolved. Do NOT run fork-evolution again."
    echo "  Proceed to Phase 4 (Reunion) or normal operation per nursemaid instructions."
else
    echo "PASS: .evolution-done absent - ready for first evolution run"
fi
```

**PASS condition:** File does NOT exist. Evolution has not run yet.

**FAIL condition:** File exists with a timestamp inside.

**Remediation if FAIL:** This is NOT an error — it means evolution already completed. The fork should skip to the Reunion phase or begin normal operation. Report to fleet management that the fork is already evolved and needs a different protocol.

---

## Test 8: Required Skills Loadable

**What it checks:** That the two most critical skills for fork operation — `agent-teams-orchestration` and `fork-evolution` — exist and are non-empty. If these skill files are missing, the fork cannot run its evolution protocol.

**Why it matters:** A fork that cannot read `fork-evolution/SKILL.md` cannot run its evolution. A fork that cannot read `agent-teams-orchestration/SKILL.md` cannot use Agent Teams correctly during evolution.

**Commands to run:**

```bash
echo "Checking required skill files..."

# Check agent-teams-orchestration
if test -s .claude/skills/agent-teams-orchestration/SKILL.md; then
    lines=$(wc -l < .claude/skills/agent-teams-orchestration/SKILL.md)
    echo "PASS: agent-teams-orchestration/SKILL.md ($lines lines)"
else
    echo "FAIL: agent-teams-orchestration/SKILL.md missing or empty"
fi

# Check fork-evolution
if test -s .claude/skills/fork-evolution/SKILL.md; then
    lines=$(wc -l < .claude/skills/fork-evolution/SKILL.md)
    echo "PASS: fork-evolution/SKILL.md ($lines lines)"
else
    echo "FAIL: fork-evolution/SKILL.md missing or empty"
fi

# Check this skill (self-reference confirmation)
if test -s .claude/skills/fork-readiness-test/SKILL.md; then
    echo "PASS: fork-readiness-test/SKILL.md (this file - exists)"
else
    echo "FAIL: fork-readiness-test/SKILL.md missing (how are you reading this?)"
fi

# Check for additional evolution-critical skills
for skill in \
    ".claude/skills/north-star/SKILL.md" \
    ".claude/skills/wake-up-protocol/SKILL.md"; do
    if test -s "$skill"; then
        echo "PASS: $skill"
    else
        echo "WARN: $skill missing (not blocking, but note for fleet management)"
    fi
done
```

**PASS condition:** Both `agent-teams-orchestration/SKILL.md` and `fork-evolution/SKILL.md` exist and are non-empty.

**FAIL condition:** Either critical skill is missing or empty.

**Remediation if FAIL:** The fork template was not properly deployed — the skills directory is incomplete. Fleet management must re-deploy the template or copy missing skill files from the parent civilization's template. Do not proceed.

---

## Test 9: Core Tool Availability

**What it checks:** That the 5 core Claude Code tools — Bash, Read, Write, Glob, and Grep — all function correctly. These are the foundation of everything agents do during evolution.

**Why it matters:** If any core tool is broken (e.g., due to permission restrictions not properly bypassed by `--dangerously-skip-permissions`), evolution phases will fail in opaque and confusing ways.

**Sub-test 9a: Bash Tool**

Run a simple echo command using the Bash tool:
```bash
result=$(echo "TOOL_TEST_PASS")
echo "$result"
```
Expected output: `TOOL_TEST_PASS`

**Sub-test 9b: Read Tool**

Use the Read tool to read this skill file itself:
```
Read: .claude/skills/fork-readiness-test/SKILL.md (first 10 lines)
```
Expected: Returns the frontmatter/header of this file.
- PASS: File contents returned
- FAIL: Permission error or file-not-found

**Sub-test 9c: Write Tool**

Use the Write tool to create a temporary file:
```
Write /tmp/tool-test-write.txt:
"WRITE_TOOL_WORKING"
```
Then verify with Bash:
```bash
content=$(cat /tmp/tool-test-write.txt)
if [ "$content" = "WRITE_TOOL_WORKING" ]; then
    echo "PASS: Write tool functional"
else
    echo "FAIL: Write tool produced wrong content: $content"
fi
rm -f /tmp/tool-test-write.txt
```

**Sub-test 9d: Glob Tool**

Use the Glob tool to search for markdown files:
```
Glob: **/*.md (in current working directory)
```
Expected: Returns at least one .md file (CLAUDE.md should be present).
- PASS: At least one result returned
- FAIL: No results or error

**Sub-test 9e: Grep Tool**

Use the Grep tool to search for a known string:
```
Grep: pattern="fork-readiness-test" in .claude/skills/fork-readiness-test/SKILL.md
```
Expected: At least one match (this file contains the pattern in its frontmatter).
- PASS: Match found
- FAIL: No match or error

**PASS condition for Test 9:** All 5 sub-tests produce expected results without permission errors.

**FAIL condition:** Any tool returns a permission error, unexpected empty result, or crashes.

**Remediation if FAIL:** Claude Code was not launched with `--dangerously-skip-permissions`. Fleet management must restart with the correct flags:
```bash
claude --dangerously-skip-permissions --model claude-sonnet-4-6
```

---

## Test 10: Teammate SendMessage Test

**What it checks:** That bi-directional messaging between the fork primary and a spawned teammate works correctly. This is validated as part of Test 4 — if Test 4 passed, Test 10 passes automatically.

**Why it matters:** Every evolution phase relies on a team lead sending instructions to specialists via `Task()` and receiving confirmation via `SendMessage`. If messaging is broken in one direction, the team lead cannot know if work completed.

**Assessment:**

If **Test 4 passed** (Agent Teams Capability):
- `TeamCreate` worked → fork primary can create teams
- test-worker spawned → fork primary can spawn teammates
- test-worker sent a `SendMessage` back → bi-directional messaging confirmed
- **Test 10: PASS** (inherited from Test 4)

If **Test 4 failed** at the messaging sub-step (file was written but no SendMessage received):
- Agent Teams spawn: working
- File write by teammate: working
- Messaging from teammate to primary: BROKEN
- **Test 10: FAIL** — flag separately even though Test 4 also failed

If Test 4 failed before the messaging step could be reached:
- Test 10 is **inconclusive** — record as "blocked by Test 4 failure"

**Remediation if Test 10 fails independently of Test 4:** This indicates a messaging infrastructure issue specific to the inter-process communication layer. Fleet management must investigate the Agent Teams messaging queue. Check that `~/.claude/tasks/` is writable and that the inbox/outbox files can be created.

---

## Summary Output

After all 10 tests, write the results to `memories/identity/readiness-check.md` and display the summary table.

First, write the record:

```bash
mkdir -p memories/identity
timestamp=$(date -u +%Y-%m-%dT%H:%M:%SZ)

cat > memories/identity/readiness-check.md << RECORD
# Fork Readiness Check Results
timestamp: $timestamp
fork: $(basename $(pwd))

## Results
[Fill in each test result: PASS/FAIL + detail]

## Decision
[READY FOR EVOLUTION / DO NOT START EVOLUTION]

## Next Step
[fork-evolution / fleet-management-intervention]
RECORD

echo "Results written to memories/identity/readiness-check.md"
```

Then display the summary table. Use the format below, replacing each icon with either `✅` (PASS) or `❌` (FAIL), and adding detail lines under failures.

**All 10 PASS — Go for evolution:**
```
╔══════════════════════════════════════════════╗
║         FORK READINESS CHECK                 ║
╠══════════════════════════════════════════════╣
║ ✅ Test 1:  OAuth / Authentication           ║
║ ✅ Test 2:  Seed Files Present               ║
║ ✅ Test 3:  Template Variables Replaced      ║
║ ✅ Test 4:  Agent Teams Capable              ║
║ ✅ Test 5:  Scratchpad Writable              ║
║ ✅ Test 6:  Memory Paths Writable            ║
║ ✅ Test 7:  Evolution-Done Flag Absent       ║
║ ✅ Test 8:  Required Skills Loadable         ║
║ ✅ Test 9:  Core Tools Functional            ║
║ ✅ Test 10: Teammate Messaging Working       ║
╠══════════════════════════════════════════════╣
║ RESULT: 10/10 PASS                           ║
║ ✅ READY FOR EVOLUTION                       ║
║ Next: Load fork-evolution/SKILL.md           ║
╚══════════════════════════════════════════════╝
```

**Failures present — Do NOT start evolution:**
```
╔══════════════════════════════════════════════╗
║         FORK READINESS CHECK                 ║
╠══════════════════════════════════════════════╣
║ ✅ Test 1:  OAuth / Authentication           ║
║ ✅ Test 2:  Seed Files Present               ║
║ ❌ Test 3:  Template Variables Replaced      ║
║    → Found vars in: CLAUDE.md, CLAUDE-OPS.md║
║    → Run sed replacement script             ║
║ ✅ Test 4:  Agent Teams Capable              ║
║ ✅ Test 5:  Scratchpad Writable              ║
║ ✅ Test 6:  Memory Paths Writable            ║
║ ✅ Test 7:  Evolution-Done Flag Absent       ║
║ ❌ Test 8:  Required Skills Loadable         ║
║    → fork-evolution/SKILL.md missing        ║
║    → Re-deploy from fork template           ║
║ ✅ Test 9:  Core Tools Functional            ║
║ ✅ Test 10: Teammate Messaging Working       ║
╠══════════════════════════════════════════════╣
║ RESULT: 8/10 PASS                            ║
║ ❌ DO NOT START EVOLUTION                    ║
║ Fix the failures above, then re-run this     ║
║ skill. Fleet management notified.            ║
╚══════════════════════════════════════════════╝
```

---

## Go / No-Go Decision

**If all 10 PASS:**
1. Write `READY` to `memories/identity/readiness-check.md`
2. Print the all-pass summary
3. **Proceed immediately to fork-evolution:**
   ```
   Read: .claude/skills/fork-evolution/SKILL.md
   ```
   Then follow its instructions from Phase 1.

**If ANY test FAILS:**
1. Write `NOT READY` and all failure details to `memories/identity/readiness-check.md`
2. Print the failure summary
3. **STOP. Do not proceed to fork-evolution.**
4. Write a fleet management notification to `memories/infrastructure/fleet-management-request.md`:
   ```markdown
   # Fleet Management Intervention Request

   timestamp: {current UTC time}
   fork: {civ name}
   vps: {hostname}

   ## Readiness Check Result: FAILED

   The following tests failed:
   - Test N: {name} — {reason}
   - Test N: {name} — {reason}

   ## Required Actions
   {list specific remediation steps per failed test}

   ## Do Not Awaken Until
   All failures above are resolved and readiness check re-run shows 10/10 PASS.
   ```
5. Await fleet management intervention.

---

## Remediation Quick Reference

| Failure | Likely Cause | Fix |
|---------|-------------|-----|
| Test 1: No `~/.claude.json` | OAuth not set up | Fleet mgmt must provision credentials |
| Test 2: Seed files missing | Nursemaid didn't write seeds | Fleet mgmt re-runs seed write step |
| Test 3: Template vars present | `sed` replacement skipped | Fleet mgmt re-runs substitution script |
| Test 4: `TeamCreate` fails | Agent Teams flag not active | Add flag to `~/.claude/settings.json`, restart Claude Code |
| Test 4: No file from worker | Write tool broken | Verify `--dangerously-skip-permissions` launch flag |
| Test 5: Scratchpad unwritable | Wrong file ownership | `chown -R aiciv:aiciv` on `.claude/` |
| Test 6: Memory paths unwritable | Wrong directory ownership | `chown -R aiciv:aiciv` on `memories/` |
| Test 7: `.evolution-done` exists | Evolution already ran | Skip to Reunion phase |
| Test 8: Skills missing | Incomplete template deploy | Re-deploy skills directory from template |
| Test 9: Tool permission error | Missing `--dangerously-skip-permissions` | Restart Claude Code with correct flags |
| Test 10: No SendMessage | Messaging queue broken | Check `~/.claude/tasks/` writability |

---

## Notes for Nursemaid Protocol

This skill is referenced as **Step 1 of every fork awakening sequence**:

```
NURSEMAID AWAKENING SEQUENCE:
  Step 0: Deploy fork template to VPS (provisioning)
  Step 1: Fork reads fork-readiness-test/SKILL.md → run all 10 tests
  Step 2: If 10/10 PASS → fork reads fork-evolution/SKILL.md → begin 6-phase evolution
  Step 3: If ANY FAIL → halt, write fleet-management-request, await intervention
```

The fork should run this skill at the very beginning of its first Claude Code session, before doing anything else. The results (pass/fail per test, total score, go/no-go decision) should be visible in the conversation so the nursemaid observer can confirm readiness before leaving the fork to evolve autonomously.

**Total expected execution time:** Under 3 minutes for all 10 tests (Test 4 is the longest at ~60 seconds for team spawn + completion).

---

*End of Fork Readiness Test Skill v1.0.0*
