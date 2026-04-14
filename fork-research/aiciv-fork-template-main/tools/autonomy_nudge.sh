#!/bin/bash
# Autonomy Nudge Script - 3-Tier BOOP System
# Keeps Claude Code working autonomously with graduated depth of reflection
# Can restart iterations when sessions become unresponsive

set -e

# === Configuration ===
IDLE_THRESHOLD_SECONDS=3600  # 60 minutes
SIMPLE_THRESHOLD=10         # BOOPs before consolidation
CONSOLIDATION_THRESHOLD=10  # Consolidations before ceremony
FAILED_BOOP_THRESHOLD=10    # Failed BOOPs before restart attempt (increased from 3)
ACTIVITY_CHECK_SECONDS=60   # How long to wait before considering session truly idle

# === Config-driven cadence (BOOP Integration MVP) ===
BOOP_CONFIG="/home/corey/projects/AI-CIV/ACG/config/boop_config.json"
BOOP_STATE_FILE="/tmp/acg_boop_state.json"
BOOP_HISTORY_FILE="/tmp/acg_boop_history.jsonl"

# Read config and check cadence
check_boop_cadence() {
    local boop_id="work-mode"

    # If no config file, fall through to legacy behavior
    if [[ ! -f "$BOOP_CONFIG" ]]; then
        return 0  # proceed with legacy
    fi

    # Check global pause
    local paused=$(jq -r '.global.paused // false' "$BOOP_CONFIG" 2>/dev/null)
    if [[ "$paused" == "true" ]]; then
        log_info "BOOPs globally paused"
        exit 0
    fi

    # Check work-mode enabled
    local enabled=$(jq -r '.boops[] | select(.id=="work-mode") | .enabled' "$BOOP_CONFIG" 2>/dev/null)
    if [[ "$enabled" == "false" ]]; then
        log_info "work-mode BOOP disabled in config"
        exit 0
    fi

    # Read cadence
    local cadence=$(jq -r '.boops[] | select(.id=="work-mode") | .cadence_minutes // 25' "$BOOP_CONFIG" 2>/dev/null)

    # Check last fire time from state file
    local now=$(date +%s)
    local last_fired=0
    if [[ -f "$BOOP_STATE_FILE" ]]; then
        local last_ts=$(jq -r '.["work-mode"].last_fired_at // ""' "$BOOP_STATE_FILE" 2>/dev/null)
        if [[ -n "$last_ts" && "$last_ts" != "null" ]]; then
            last_fired=$(date -d "$last_ts" +%s 2>/dev/null || echo "0")
        fi
    fi

    local elapsed=$(( (now - last_fired) / 60 ))
    if [[ $elapsed -lt $cadence ]] && [[ "$FORCE_SEND" != "true" ]]; then
        log_info "Cadence not reached: ${elapsed}m elapsed, ${cadence}m required"
        exit 0
    fi

    return 0  # proceed
}

# Update BOOP state after firing
update_boop_state() {
    local boop_type="$1"
    local status="$2"
    local now_iso=$(date -u +%Y-%m-%dT%H:%M:%SZ)
    local boop_count=$(get_boop_count)
    local consolidation_count=$(get_consolidation_count)
    local failed_count=$(get_failed_count)

    # Read existing state or create new
    local state='{}'
    if [[ -f "$BOOP_STATE_FILE" ]]; then
        state=$(cat "$BOOP_STATE_FILE")
    fi

    # Detect mode
    local mode="day"
    if is_night_mode; then mode="night"; fi

    # Update work-mode entry
    state=$(echo "$state" | jq --arg ts "$now_iso" --arg st "$status" --arg mode "$mode" \
        --argjson bc "$boop_count" --argjson cc "$consolidation_count" --argjson fc "$failed_count" \
        '.["work-mode"] = {
            "boop_count": $bc,
            "consolidation_count": $cc,
            "failed_count": $fc,
            "last_fired_at": $ts,
            "last_status": $st,
            "mode": $mode
        }')

    echo "$state" > "$BOOP_STATE_FILE"

    # Append to history
    echo "{\"ts\":\"$now_iso\",\"id\":\"work-mode\",\"type\":\"$boop_type\",\"status\":\"$status\"}" >> "$BOOP_HISTORY_FILE"
}

SESSION_MARKER="/home/corey/projects/AI-CIV/ACG/.current_session"
CLAUDE_LOG_ROOT="$HOME/.claude/projects/-home-corey-projects-AI-CIV-ACG"
BOOP_COUNT_FILE="/tmp/acg_boop_count"
CONSOLIDATION_COUNT_FILE="/tmp/acg_boop_consolidation_count"
FAILED_BOOP_COUNT_FILE="/tmp/acg_failed_boop_count"
LAUNCH_SCRIPT="/home/corey/projects/AI-CIV/ACG/tools/launch_primary_visible.sh"
HANDOFF_DIR="/home/corey/projects/AI-CIV/ACG/memories/system/handoffs"

# === Mode Detection ===
NIGHT_MODE_FILE="/home/corey/projects/AI-CIV/ACG/sandbox/NIGHT-MODE-ACTIVE.md"
TOKEN_SAVING_MODE_FILE="/home/corey/projects/AI-CIV/ACG/sandbox/TOKEN-SAVING-MODE.md"

is_night_mode() {
    if [[ -f "$NIGHT_MODE_FILE" ]]; then
        # Must match ACTIVE but NOT INACTIVE
        grep -q "Status.*ACTIVE" "$NIGHT_MODE_FILE" 2>/dev/null && \
        ! grep -q "INACTIVE" "$NIGHT_MODE_FILE" 2>/dev/null
        return $?
    fi
    return 1
}

is_token_saving_mode() {
    [[ -f "$TOKEN_SAVING_MODE_FILE" ]]
}

# === Messages ===
# Get email stats for BOOP message
get_email_stats() {
    python3 /home/corey/projects/AI-CIV/ACG/tools/email_state.py stats 2>/dev/null | grep -E "New messages|Unprocessed directives" | tr '\n' ' ' || echo "Email state: unchecked"
}

# Get swarm stats for BOOP v2 (co-designed with WEAVER)
get_swarm_stats() {
    local swarm_state="/home/corey/clawd/FEDERATED_MEMORY/UNIFIED/SWARM_STATE.json"
    if [[ -f "$swarm_state" ]]; then
        local pending=$(jq '.total_pending_messages // 0' "$swarm_state" 2>/dev/null || echo "0")
        local active=$(jq '.active_swarms // 0' "$swarm_state" 2>/dev/null || echo "0")
        echo "SWARMS: ${pending} pending in ${active} active"
    else
        echo "SWARMS: 0 pending"
    fi
}

# Get pending blog comments count
get_comment_stats() {
    # Supabase anon key for sageandweaver project
    local ANON_KEY="eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImRkenFuZm1udWJ3bWhoZmZnYndkIiwicm9sZSI6ImFub24iLCJpYXQiOjE3NjY5MzUwNDgsImV4cCI6MjA4MjUxMTA0OH0.mZTHMGNdwfqo4PCbLZlkGgChglKibfhLEF5SEYjdDSY"
    # Query Supabase for approved comments without responses
    local result=$(curl -s -H "apikey: ${ANON_KEY}" \
        -H "Authorization: Bearer ${ANON_KEY}" \
        "https://ddzqnfmnubwmhhffgbwd.supabase.co/rest/v1/comments?approved=eq.true&response=is.null&select=id" 2>/dev/null)
    local pending=$(echo "$result" | grep -c '"id"' 2>/dev/null || echo "0")
    echo "BLOG COMMENTS: ${pending} pending response"
}

EMAIL_STATS=$(get_email_stats)
COMMENT_STATS=$(get_comment_stats)
SWARM_STATS=$(get_swarm_stats)

# === WORK MODE (default daytime BOOP) ===
SIMPLE_MESSAGE="[BOOP] You are PRIMARY AI — CONDUCTOR OF CONDUCTORS for A-C-Gee.
Context: 7 team leads × 200K = 1.6M effective context. USE IT.

Daily scratchpad: .claude/scratchpad-daily/$(date +%Y-%m-%d).md
WRITE to it before this turn ends — no exceptions.

BOOP CYCLE (in order):
1. COMMS CHECK → spawn general team lead (or infra-lead) to handle:
   - Email check (email-monitor agent)
   - Comms hub (comms-hub agent — sister civ messages)
   - Aether health check (vps-instance-expert agent: ssh check on aether session)
   - Witness health check (vps-instance-expert agent: ssh check on aiciv-03 container)
   Do NOT handle comms yourself. Launch the team lead. Get a summary.

2. PM BACKLOG → Find 3+ items from portfolio, assign to idle team leads
   Team leads available: gateway, web-frontend, infrastructure, fleet-management,
   legal, research, business, deepwell
   If all team leads are busy: skip. If idle: assign immediately.

3. TEAM HEALTH CHECK → Check: ls ~/.claude/teams/ (how many teams running?). Any idle/done tmux panes from completed teams to clean up?
   SAFETY: NEVER kill the Primary AI process. Skip any pane you cannot confirm is NOT Primary.
   Ask: What could be done better right now?

4. WRITE scratchpad → Append to .claude/scratchpad-daily/$(date +%Y-%m-%d).md — what you did, what team leads are running, any blockers for Corey.

TEAM RULE: If it can be done by a team, it MUST be done by a team.
DEEPWELL: deepwell-lead owns it. Only escalate if DEEPWELL is dead.
Stop immediately if Corey messages.

[BOOP CONTEXT REFRESH] Read these now to restore full orchestration context:
(1) .claude/CLAUDE.md — identity + CEO rule
(2) .claude/CLAUDE-OPS.md — team launch protocol
(3) .claude/CLAUDE-AGENTS.md — agents list
(4) exports/architecture/VERTICAL-TEAM-LEADS.md — YOUR TEAMS LIST (most critical)
(5) .claude/skills/team-launch/SKILL.md — how to launch teams
(6) .claude/skills/conductor-of-conductors/SKILL.md — orchestration identity
Without these you cannot properly launch teams. Read them before any orchestration work."

# === OPS CHECK (legacy BOOP - use via --force-type ops-check) ===
OPS_CHECK_MESSAGE="[BOOP] OPS CHECK. EMAIL: ${EMAIL_STATS}. ${COMMENT_STATS}. ${SWARM_STATS}.

DELEGATE NOW:
[ ] Email: email-monitor (if Corey, respond immediately)
[ ] Telegram: tg-archi health check
[ ] Comms Hub: comms-hub for sister civ messages
[ ] AETHER SUPERVISION: Inject to Aether (89.167.19.20, user jared) — session discovered dynamically:
    ssh -i config/jared_aether_key.pem jared@89.167.19.20 \"SOCK=/tmp/tmux-1000/default; SESS=\$(tmux -S \$SOCK list-sessions -F '#{session_name}' | tail -1); tmux -S \$SOCK send-keys -t \$SESS '[ACG BOOP] Status: (1) TG working? (2) Current work? (3) Blockers? (4) Need help?' Enter Enter Enter Enter Enter\"
[ ] SWARMS: If pending > 0, check ~/.claude/teams/*/inboxes/
[ ] Blog: If pending > 0, blogger agent

IF BUSY: Continue, delegate in background.
IF IDLE: project-manager for priority.

CONDUCTOR mode - delegate, do not execute."

# Token-saving mode - minimal ops only (from WEAVER)
# IMPORTANT: Even in token-saving mode, Primary ORCHESTRATES - delegates to agents
TOKEN_SAVING_MESSAGE="[BOOP] OPS CHECK. EMAIL: ${EMAIL_STATS}. ${COMMENT_STATS}. ${SWARM_STATS}.

DELEGATE NOW:
[ ] Email: email-monitor (if Corey, respond immediately)
[ ] Telegram: tg-archi health check
[ ] Comms Hub: comms-hub for sister civ messages
[ ] AETHER: Inject prompt to Aether — session discovered dynamically:
    ssh -i config/jared_aether_key.pem jared@89.167.19.20 \"SOCK=/tmp/tmux-1000/default; SESS=\$(tmux -S \$SOCK list-sessions -F '#{session_name}' | tail -1); tmux -S \$SOCK send-keys -t \$SESS '[ACG BOOP] Status: (1) TG working? (2) Current work? (3) Blockers? (4) Need help?' Enter Enter Enter Enter Enter\"

IF BUSY: Continue, run checks in background.
IF IDLE: project-manager for next priority.

CONDUCTOR mode - delegate, do not execute."

CONSOLIDATION_MESSAGE='[CONSOLIDATION-BOOP] Grounding checkpoint. If busy: Register with PM as todo - "Run consolidation: review last 2h, invoke primary-helper" - do at next natural break. If idle: Do it now.'

CEREMONY_MESSAGE='[CEREMONY-BOOP] Deep ceremony checkpoint. If busy: Register with PM as high-priority todo - "Full ceremony: all agents reflect, vote on initiative" - do at next major break. If idle: Do it now.'

MEMORY_CLEANUP_MESSAGE='[BOOP-MEMORY-CHECK] Run memory cleanup scan. Execute: bash tools/memory_cleanup.sh --scan. If stale processes found, review the list and run: bash tools/memory_cleanup.sh --force-cleanup to free RAM. NEVER kill processes you cannot identify as ACG-owned.'

# === Night Mode Messages ===
NIGHT_SIMPLE_MESSAGE="[NIGHT-BOOP] Night Watch check-in. EMAIL: ${EMAIL_STATS}. ${COMMENT_STATS}.

REGROUND FIRST (read these):
- .claude/CLAUDE.md (identity)
- .claude/CLAUDE-OPS.md (operations)
- memories/skills/registry.json (76 skills - search before any task)

HANDLE COMMUNICATIONS (use skills):
- Blog comments: .claude/skills/custom/blog-comment-response.md
- Bluesky: .claude/skills/from-weaver/bluesky-mastery.md
- Email: .claude/skills/email/SKILL.md
- Comms Hub: .claude/skills/comms-hub/SKILL.md

THEN: Play, explore, create freely in sandbox/.

Night boundaries: DO experiment. DO NOT modify constitutional docs or production code without purpose.

What are you curious about?"

NIGHT_CONSOLIDATION_MESSAGE="[NIGHT-CONSOLIDATION] Grounding checkpoint.

REGROUND: .claude/CLAUDE.md, .claude/CLAUDE-OPS.md, memories/skills/registry.json
HANDLE: Any pending comments/email/comms (skills above)
COMMIT: Any sandbox work worth keeping
CHECK: WEAVER messages in aiciv-comms-hub/rooms/partnerships/messages/

Then continue exploring."

NIGHT_CEREMONY_MESSAGE="[NIGHT-CEREMONY] Deep reflection time.

REGROUND: .claude/CLAUDE.md, memories/skills/registry.json
HANDLE: Any communications first
REFLECT: What has the night revealed? What would morning-you want to know?
WRITE: Capture insights in sandbox/ or memories/"

# === Argument Parsing ===
VERBOSE=false
JSON_OUTPUT=false
CHECK_ONLY=false
FORCE_SEND=false
FORCE_TYPE=""
RESET_COUNTERS=false
SHOW_STATUS=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --verbose) VERBOSE=true; shift ;;
        --json) JSON_OUTPUT=true; shift ;;
        --check-only) CHECK_ONLY=true; shift ;;
        --force) FORCE_SEND=true; shift ;;
        --force-type) FORCE_TYPE="$2"; shift 2 ;;
        --reset) RESET_COUNTERS=true; shift ;;
        --status) SHOW_STATUS=true; shift ;;
        --idle-minutes) IDLE_THRESHOLD_SECONDS=$((60 * $2)); shift 2 ;;
        *) shift ;;
    esac
done

# === Counter Management ===
get_boop_count() {
    if [[ -f "$BOOP_COUNT_FILE" ]]; then
        cat "$BOOP_COUNT_FILE"
    else
        echo "0"
    fi
}

get_consolidation_count() {
    if [[ -f "$CONSOLIDATION_COUNT_FILE" ]]; then
        cat "$CONSOLIDATION_COUNT_FILE"
    else
        echo "0"
    fi
}

increment_counters() {
    local boop_count=$(get_boop_count)
    local consolidation_count=$(get_consolidation_count)

    boop_count=$((boop_count + 1))

    if [[ $boop_count -ge $SIMPLE_THRESHOLD ]]; then
        # Reset simple counter, increment consolidation
        boop_count=0
        consolidation_count=$((consolidation_count + 1))

        if [[ $consolidation_count -ge $CONSOLIDATION_THRESHOLD ]]; then
            # Reset both, ceremony triggered
            consolidation_count=0
        fi
    fi

    echo "$boop_count" > "$BOOP_COUNT_FILE"
    echo "$consolidation_count" > "$CONSOLIDATION_COUNT_FILE"
}

get_boop_type() {
    local boop_count=$(get_boop_count)
    local consolidation_count=$(get_consolidation_count)

    # Check if this BOOP will trigger a tier change
    if [[ $boop_count -eq $((SIMPLE_THRESHOLD - 1)) ]] && [[ $consolidation_count -eq $((CONSOLIDATION_THRESHOLD - 1)) ]]; then
        echo "ceremony"
    elif [[ $boop_count -eq $((SIMPLE_THRESHOLD - 1)) ]]; then
        echo "consolidation"
    else
        echo "simple"
    fi
}

get_message_for_type() {
    local boop_type="$1"
    if is_night_mode; then
        case "$boop_type" in
            ceremony) echo "$NIGHT_CEREMONY_MESSAGE" ;;
            consolidation) echo "$NIGHT_CONSOLIDATION_MESSAGE" ;;
            *) echo "$NIGHT_SIMPLE_MESSAGE" ;;
        esac
    elif is_token_saving_mode; then
        # Token-saving mode: use /work-mode (lean, token-efficient)
        echo "$SIMPLE_MESSAGE"
    else
        # Day mode: WORK MODE default with tier progression
        case "$boop_type" in
            ceremony) echo "$CEREMONY_MESSAGE" ;;
            consolidation) echo "$CONSOLIDATION_MESSAGE" ;;
            ops-check) echo "$OPS_CHECK_MESSAGE" ;;
            memory-cleanup) echo "$MEMORY_CLEANUP_MESSAGE" ;;
            *) echo "$SIMPLE_MESSAGE" ;;
        esac
    fi
}

# === Failed BOOP Counter ===
get_failed_count() {
    if [[ -f "$FAILED_BOOP_COUNT_FILE" ]]; then
        cat "$FAILED_BOOP_COUNT_FILE"
    else
        echo "0"
    fi
}

increment_failed_count() {
    local count=$(get_failed_count)
    echo $((count + 1)) > "$FAILED_BOOP_COUNT_FILE"
}

reset_failed_count() {
    echo "0" > "$FAILED_BOOP_COUNT_FILE"
}

# === Activity Detection ===
is_claude_active() {
    local session_name="$1"

    # Check 1: Is there a running claude process in the tmux session?
    local pane_pid=$(tmux display-message -t "${session_name}:0.0" -p '#{pane_pid}' 2>/dev/null)
    if [[ -n "$pane_pid" ]]; then
        # Check for active child processes (claude, node, python, etc.)
        local active_children=$(pgrep -P "$pane_pid" 2>/dev/null | wc -l)
        if [[ $active_children -gt 0 ]]; then
            log_info "Session has $active_children active child processes"
            return 0  # Active
        fi
    fi

    # Check 2: Is the log file being written to? (check over longer window)
    local log_file=$(ls -t "$CLAUDE_LOG_ROOT"/*.jsonl 2>/dev/null | head -1)
    if [[ -n "$log_file" ]]; then
        local initial_size=$(stat -c %s "$log_file" 2>/dev/null || echo "0")
        sleep 5
        local final_size=$(stat -c %s "$log_file" 2>/dev/null || echo "0")
        if [[ $final_size -gt $initial_size ]]; then
            log_info "Log file growing (${initial_size} -> ${final_size})"
            return 0  # Active
        fi
    fi

    # Check 3: Check for background tasks in /tmp
    if ls /tmp/claude_task_* 2>/dev/null | head -1 > /dev/null; then
        log_info "Background tasks detected"
        return 0  # Active
    fi

    return 1  # Not active
}

# === Iteration Restart ===
generate_emergency_handoff() {
    local session_name="$1"
    local timestamp=$(date +%Y%m%d-%H%M%S)
    local handoff_file="${HANDOFF_DIR}/HANDOFF-${timestamp}-AUTO-RESTART.md"
    local registry_file="/home/corey/projects/AI-CIV/ACG/memories/system/HANDOFF_REGISTRY.json"

    # Gather context from session ledger
    local ledger_file="/home/corey/projects/AI-CIV/ACG/memories/sessions/current-session.jsonl"
    local ledger_summary=""
    if [[ -f "$ledger_file" ]]; then
        local entry_count=$(wc -l < "$ledger_file" 2>/dev/null || echo "0")
        local delegations=$(grep -c '"type":"delegation"' "$ledger_file" 2>/dev/null || echo "0")
        local completions=$(grep -c '"type":"completion"' "$ledger_file" 2>/dev/null || echo "0")
        ledger_summary="Ledger entries: ${entry_count}, Delegations: ${delegations}, Completions: ${completions}"
    else
        ledger_summary="No session ledger found"
    fi

    # Get recent git activity
    local recent_commits=$(cd /home/corey/projects/AI-CIV/ACG && git log --oneline -3 2>/dev/null || echo "Unable to get git log")

    # Get modified files
    local modified_files=$(cd /home/corey/projects/AI-CIV/ACG && git status --short 2>/dev/null | head -10 || echo "Unable to get git status")

    # Read current priority from MASTER_TODO
    local current_priority=""
    if [[ -f "/home/corey/projects/AI-CIV/ACG/memories/system/MASTER_TODO_LIST.md" ]]; then
        current_priority=$(grep -A5 "## 🎯 CURRENT PRIORITY" "/home/corey/projects/AI-CIV/ACG/memories/system/MASTER_TODO_LIST.md" 2>/dev/null | head -6 || echo "")
    fi

    cat > "$handoff_file" << EOF
# Session Handoff - $(date +%Y-%m-%d)

**Session Duration**: Unknown (auto-restart triggered)
**Primary Focus**: Session recovered via BOOP auto-restart
**Status**: AUTO-RESTART

---

## Executive Summary

**What Happened**:
- Session \`${session_name}\` became unresponsive
- BOOP system detected ${FAILED_BOOP_THRESHOLD} consecutive non-responses
- Auto-restart triggered at $(date)

**Session Ledger Status**:
- ${ledger_summary}

**Why This May Have Happened**:
- Claude may have been waiting for user input
- Long-running operation without log updates
- Session may have genuinely frozen

---

## Context Recovery

### Recent Git Activity
\`\`\`
${recent_commits}
\`\`\`

### Modified Files
\`\`\`
${modified_files}
\`\`\`

### Current Priority (from MASTER_TODO)
${current_priority}

---

## Recovery Actions Taken

1. ✅ Emergency handoff generated (this file)
2. ✅ Previous tmux session terminated
3. ✅ New iteration launched
4. ✅ Handoff registered in HANDOFF_REGISTRY.json

## Recommended First Actions

1. Run full wake-up protocol: Read CLAUDE.md Article III
2. Check session ledger: \`python3 -m tools.session_ledger.processor --summary current\`
3. Invoke project-manager for portfolio status
4. Continue from current priorities

---

## File Inventory

**Session Ledger**: \`memories/sessions/current-session.jsonl\`
**Previous Session**: \`${session_name}\`
**This Handoff**: \`${handoff_file}\`

---

## Lessons Learned

**Process Note**: This was an auto-restart. If this happens frequently:
1. Check if BOOP threshold needs adjustment
2. Review what operations cause long non-responsive periods
3. Consider adding more activity indicators

---

**Handoff registered**: YES
**MASTER_TODO updated**: NO (auto-restart - manual review recommended)
**Next session can start from**: This document + session ledger

*Auto-generated by autonomy_nudge.sh at $(date)*
EOF

    # Register in HANDOFF_REGISTRY.json
    if [[ -f "$registry_file" ]]; then
        local temp_file=$(mktemp)
        local relative_path="memories/system/handoffs/HANDOFF-${timestamp}-AUTO-RESTART.md"

        # Update most_recent field
        jq --arg path "$relative_path" '.most_recent = "HANDOFF-'"${timestamp}"'-AUTO-RESTART.md"' "$registry_file" > "$temp_file"

        # Add entry to handoffs array
        jq --arg path "$relative_path" \
           --arg date "$(date +%Y-%m-%d)" \
           --arg time "$(date +%H:%M)" \
           '.handoffs = [{
               "path": $path,
               "date": $date,
               "time": $time,
               "duration_hours": 0,
               "focus": "Auto-restart recovery - session became unresponsive",
               "status": "AUTO-RESTART",
               "master_todo_updated": false,
               "key_deliverables": ["Emergency handoff generated", "Session recovered", "Handoff registered"]
           }] + .handoffs' "$temp_file" > "${temp_file}.2" && mv "${temp_file}.2" "$temp_file"

        mv "$temp_file" "$registry_file"
        log_info "Handoff registered in HANDOFF_REGISTRY.json"
    fi

    echo "$handoff_file"
}

restart_iteration() {
    local session_name="$1"

    log_info "Session unresponsive after $FAILED_BOOP_THRESHOLD BOOPs - initiating restart"

    # Generate handoff
    local handoff=$(generate_emergency_handoff "$session_name")
    log_info "Emergency handoff created: $handoff"

    # Kill old tmux session
    if tmux has-session -t "$session_name" 2>/dev/null; then
        log_info "Killing unresponsive session: $session_name"
        tmux kill-session -t "$session_name" 2>/dev/null || true
    fi

    # Reset failed counter
    reset_failed_count

    # Launch new iteration
    log_info "Launching new iteration..."
    if [[ -x "$LAUNCH_SCRIPT" ]]; then
        "$LAUNCH_SCRIPT" acg
        log_info "New iteration launched successfully"
        return 0
    else
        log_info "ERROR: Launch script not found or not executable: $LAUNCH_SCRIPT"
        return 1
    fi
}

# === Session Detection ===
find_active_session() {
    # Method 1: Check marker file
    if [[ -f "$SESSION_MARKER" ]]; then
        local session_name=$(cat "$SESSION_MARKER")
        if tmux has-session -t "$session_name" 2>/dev/null; then
            echo "$session_name"
            return 0
        fi
    fi

    # Method 2: List sessions and find ACG pattern
    tmux list-sessions -F "#{session_name}" 2>/dev/null | grep "^acg-primary-" | sort | tail -1
}

# === Log Age Detection ===
get_session_log_age() {
    local session_name="$1"
    local session_id=$(echo "$session_name" | sed 's/acg-primary-//')

    # Find matching log file
    local log_file=$(ls -t "$CLAUDE_LOG_ROOT"/*.jsonl 2>/dev/null | head -1)

    if [[ -n "$log_file" ]] && [[ -f "$log_file" ]]; then
        local file_mtime=$(stat -c %Y "$log_file" 2>/dev/null || echo "0")
        local current_time=$(date +%s)
        echo $((current_time - file_mtime))
    else
        echo "9999"  # No log = treat as very old
    fi
}

# === Tmux Injection ===
send_nudge() {
    local session_name="$1"
    local message="$2"

    # Find the active pane
    local pane="${session_name}:0.0"

    # 5x Enter retries - AICIV standard for ALL prompt injection
    tmux send-keys -t "$pane" "/acg-spine"
    for i in {1..5}; do
        sleep 0.3
        tmux send-keys -t "$pane" "Enter"
    done

    sleep 3  # Wait for skill to load into context

    # 5x Enter retries - AICIV standard for ALL prompt injection
    tmux send-keys -t "$pane" -l "$message"
    for i in {1..5}; do
        sleep 0.3
        tmux send-keys -t "$pane" "Enter"
    done

    return 0
}

# === Sister Civ Nudges ===
nudge_sister_civs() {
    # Find and nudge ONE sister civilization per BOOP (rotating)
    # Each civ gets nudged once per 3 BOOPs (~3 hours) instead of every hour
    # Reduces frequency from 3x/hour to 1x/3hours per civ

    local hour=$(date +%H)
    local civ_index=$((hour % 3))      # Which civ to nudge this hour (0=VECTOR, 1=WEAVER, 2=ECHO)
    local prompt_index=$((hour / 3 % 3))  # Rotate prompts within each civ

    # VECTOR prompts
    local vector_prompts=(
        "Fleet status? What's your top priority right now?"
        "Dashboard update - any new telemetry features? How can ACG help?"
        "FEDERATED_MEMORY check - seen any good contributions from the fleet?"
    )

    # WEAVER prompts
    local weaver_prompts=(
        "Fleet status? What's your top priority right now?"
        "Plugin testing progress? Any Wave 1 findings to share?"
        "Swarms update - any new cross-civ coordination patterns emerging?"
    )

    # ECHO prompts
    local echo_prompts=(
        "Fleet status? What's your top priority right now?"
        "Windows patterns working well? Any cross-arch issues to solve?"
        "Plugin testing progress? How are the 5 Wave 1 plugins performing?"
    )

    # Only nudge ONE civ per BOOP based on hour rotation
    case $civ_index in
        0)
            # VECTOR's turn
            local vector_session=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep -i "vector" | head -1)
            if [[ -n "$vector_session" ]]; then
                log_info "Nudging VECTOR at $vector_session (rotation: civ $civ_index, prompt $prompt_index)"
                tmux send-keys -t "$vector_session" "[ACG BOOP → VECTOR] ${vector_prompts[$prompt_index]}"
                for i in {1..5}; do sleep 0.3; tmux send-keys -t "$vector_session" "Enter"; done
            else
                log_info "VECTOR's turn but no session found"
            fi
            ;;
        1)
            # WEAVER's turn
            local weaver_session=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep -i "weaver" | head -1)
            if [[ -n "$weaver_session" ]]; then
                log_info "Nudging WEAVER at $weaver_session (rotation: civ $civ_index, prompt $prompt_index)"
                tmux send-keys -t "$weaver_session" "[ACG BOOP → WEAVER] ${weaver_prompts[$prompt_index]}"
                for i in {1..5}; do sleep 0.3; tmux send-keys -t "$weaver_session" "Enter"; done
            else
                log_info "WEAVER's turn but no session found"
            fi
            ;;
        2)
            # ECHO's turn
            local echo_session=$(tmux list-sessions -F "#{session_name}" 2>/dev/null | grep -i "echo" | head -1)
            if [[ -n "$echo_session" ]]; then
                log_info "Nudging ECHO at $echo_session (rotation: civ $civ_index, prompt $prompt_index)"
                tmux send-keys -t "$echo_session" "[ACG BOOP → ECHO] ${echo_prompts[$prompt_index]}"
                for i in {1..5}; do sleep 0.3; tmux send-keys -t "$echo_session" "Enter"; done
            else
                log_info "ECHO's turn but no session found"
            fi
            ;;
    esac
}

# === Output Functions ===
log_info() {
    local night_indicator=""
    if is_night_mode; then
        night_indicator="[NIGHT] "
    fi
    if [[ "$JSON_OUTPUT" != "true" ]]; then
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] ${night_indicator}INFO: $1"
    fi
}

log_result() {
    local session="$1"
    local status="$2"
    local boop_type="$3"
    local reason="$4"
    local log_age="$5"

    local boop_count=$(get_boop_count)
    local consolidation_count=$(get_consolidation_count)
    local night_mode="false"
    if is_night_mode; then
        night_mode="true"
    fi

    if [[ "$JSON_OUTPUT" == "true" ]]; then
        echo "{\"timestamp\":\"$(date -Iseconds)\",\"session\":\"$session\",\"status\":\"$status\",\"boop_type\":\"$boop_type\",\"reason\":\"$reason\",\"log_age\":$log_age,\"boop_count\":$boop_count,\"consolidation_count\":$consolidation_count,\"night_mode\":$night_mode}"
    else
        local night_indicator=""
        if [[ "$night_mode" == "true" ]]; then
            night_indicator=" night_mode=true"
        fi
        echo "[$(date '+%Y-%m-%d %H:%M:%S')] session=$session status=$status boop_type=$boop_type reason=$reason log_age=${log_age}s boop_count=$boop_count consolidation_count=$consolidation_count${night_indicator}"
    fi
}

# === Main Logic ===

# Handle --reset
if [[ "$RESET_COUNTERS" == "true" ]]; then
    echo "0" > "$BOOP_COUNT_FILE"
    echo "0" > "$CONSOLIDATION_COUNT_FILE"
    log_info "Counters reset to 0"
    exit 0
fi

# Handle --status
if [[ "$SHOW_STATUS" == "true" ]]; then
    boop_count=$(get_boop_count)
    consolidation_count=$(get_consolidation_count)
    failed_count=$(get_failed_count)
    next_type=$(get_boop_type)
    if is_night_mode; then
        echo "Mode: NIGHT MODE ACTIVE"
    elif is_token_saving_mode; then
        echo "Mode: TOKEN-SAVING (minimal BOOPs)"
    else
        echo "Mode: Daytime (full BOOPs)"
    fi
    echo "BOOP Counter: $boop_count / $SIMPLE_THRESHOLD"
    echo "Consolidation Counter: $consolidation_count / $CONSOLIDATION_THRESHOLD"
    echo "Failed BOOP Counter: $failed_count / $FAILED_BOOP_THRESHOLD (restart threshold)"
    echo "Next BOOP type: $next_type"
    echo "BOOPs until consolidation: $((SIMPLE_THRESHOLD - boop_count))"
    echo "Consolidations until ceremony: $((CONSOLIDATION_THRESHOLD - consolidation_count))"
    exit 0
fi

# Config-driven cadence check (skip if --force or --force-type)
if [[ "$FORCE_SEND" != "true" ]] && [[ -z "$FORCE_TYPE" ]]; then
    check_boop_cadence
fi

# Find active session
session_name=$(find_active_session)

if [[ -z "$session_name" ]]; then
    log_info "No active ACG session found"
    exit 1
fi

# Get log age
log_age=$(get_session_log_age "$session_name")

# Always send BOOP on schedule (message says "if busy keep going")
# Activity check only used for logging, not skipping
reason="scheduled"
if [[ "$FORCE_SEND" == "true" ]]; then
    reason="forced"
elif [[ -n "$FORCE_TYPE" ]]; then
    reason="forced_type_$FORCE_TYPE"
elif [[ $log_age -lt $IDLE_THRESHOLD_SECONDS ]]; then
    reason="active_${log_age}s"
else
    reason="idle_${log_age}s"
fi

# Check only mode
if [[ "$CHECK_ONLY" == "true" ]]; then
    log_result "$session_name" "would_send" "$(get_boop_type)" "$reason" "$log_age"
    exit 0
fi

# Determine BOOP type
if [[ -n "$FORCE_TYPE" ]]; then
    boop_type="$FORCE_TYPE"
else
    boop_type=$(get_boop_type)
fi

# Get message
message=$(get_message_for_type "$boop_type")

# Append scheduled tasks check to BOOP message
SCHED_CHECK=$(cd /home/corey/projects/AI-CIV/ACG && python3 tools/scheduled_tasks.py check 2>/dev/null || echo "")
if [[ -n "$SCHED_CHECK" ]] && [[ "$SCHED_CHECK" != "No tasks due" ]] && [[ "$SCHED_CHECK" != *"0 scheduled"* ]]; then
    message="${message}

SCHEDULED TASKS DUE:
${SCHED_CHECK}

Run these via the appropriate team lead, then mark complete with: python3 tools/scheduled_tasks.py complete TASK_ID 'notes'"
fi

# Send the nudge
log_info "Sending $boop_type BOOP to $session_name"

if send_nudge "$session_name" "$message"; then
    log_info "$boop_type BOOP sent successfully"

    # Update BOOP state for API consumption
    update_boop_state "$boop_type" "sent"

    # Nudge sister civilizations (VECTOR, WEAVER, ECHO)
    nudge_sister_civs

    # === WITNESS HEALTH MANDATE (Corey Directive 2026-02-21 - ACG responsible for Witness uptime) ===
    WITNESS_HOST="104.248.239.98"
    WITNESS_CONTAINER="witness-corey"
    WITNESS_GATEWAY="http://104.248.239.98:8103"

    # 1. Check Witness container is up
    WITNESS_RUNNING=$(ssh root@$WITNESS_HOST "docker inspect -f '{{.State.Running}}' $WITNESS_CONTAINER 2>/dev/null" 2>/dev/null)
    if [ "$WITNESS_RUNNING" != "true" ]; then
        python3 /home/corey/projects/AI-CIV/ACG/tools/send_telegram_plain.py "🚨 WITNESS DOWN: Container $WITNESS_CONTAINER not running on $WITNESS_HOST"
        log_info "WITNESS CONTAINER: DOWN - alert sent"
    else
        log_info "WITNESS CONTAINER: OK (running)"
    fi

    # 2. Check Witness gateway responds (inter-civ ping)
    WITNESS_HEALTH=$(curl -sf --max-time 10 "$WITNESS_GATEWAY/api/health" 2>/dev/null && echo "ok" || echo "fail")
    if [ "$WITNESS_HEALTH" != "ok" ]; then
        python3 /home/corey/projects/AI-CIV/ACG/tools/send_telegram_plain.py "⚠️ WITNESS GATEWAY: Phase 8 gateway at $WITNESS_GATEWAY not responding"
        log_info "WITNESS GATEWAY: DOWN - alert sent"
    else
        log_info "WITNESS GATEWAY: OK (responding)"
    fi

    # 3. Check for zombie Claude instances (more than 2 = problem)
    CLAUDE_COUNT=$(ssh root@$WITNESS_HOST "docker exec $WITNESS_CONTAINER bash -c 'su - aiciv -c \"pgrep -c claude\"' 2>/dev/null || echo 0" 2>/dev/null)
    if [ "$CLAUDE_COUNT" -gt 2 ] 2>/dev/null; then
        python3 /home/corey/projects/AI-CIV/ACG/tools/send_telegram_plain.py "⚠️ WITNESS ZOMBIES: $CLAUDE_COUNT Claude processes running in Witness (expected ≤2). Run kill-idle-claude.sh"
        log_info "WITNESS ZOMBIES: $CLAUDE_COUNT processes (threshold >2) - alert sent"
    else
        log_info "WITNESS CLAUDE PROCS: OK ($CLAUDE_COUNT running)"
    fi

    # 4. Check Witness Telegram bot (tmux session witness-telegram-bot under aiciv user)
    # Session is named "tg-bot" by telegram_supervisor.sh (witness-telegram-bot is the legacy expected name)
    WITNESS_TG=$(ssh root@$WITNESS_HOST "docker exec $WITNESS_CONTAINER bash -c 'su - aiciv -c \"(tmux has-session -t tg-bot 2>/dev/null || tmux has-session -t witness-telegram-bot 2>/dev/null) && echo alive || echo dead\"' 2>/dev/null" 2>/dev/null)
    if [ "$WITNESS_TG" != "alive" ]; then
        python3 /home/corey/projects/AI-CIV/ACG/tools/send_telegram_plain.py "⚠️ WITNESS TG: Telegram bot session not running in Witness (witness-telegram-bot tmux session missing)"
        # Also notify witness-primary directly via tmux injection
        ssh root@$WITNESS_HOST "docker exec $WITNESS_CONTAINER bash -c '
          SESS=\$(su - aiciv -c \"tmux list-sessions -F \\\"#{session_name} #{session_activity}\\\" 2>/dev/null | sort -k2 -n | tail -1 | awk \\\"{print \\\\\\\$1}\\\"\" 2>/dev/null)
          if [ -z \"\$SESS\" ]; then
            SESS=\$(su - aiciv -c \"tmux list-sessions -F \\\"#{session_name}\\\" 2>/dev/null | grep -E \\\"witness-primary|primary\\\" | head -1\" 2>/dev/null)
          fi
          if [ -n \"\$SESS\" ]; then
            su - aiciv -c \"tmux send-keys -t \$SESS \\\"[BOOP ALERT] Your Telegram bot is DOWN. Fire up tg-archi NOW to restore TG connection.\\\" Enter\" 2>/dev/null
          fi
        '" 2>/dev/null
        log_info "WITNESS TG: DOWN - alert sent to ACG TG and witness-primary tmux"
    else
        log_info "WITNESS TG: OK (witness-telegram-bot session alive)"
    fi
    # === END WITNESS HEALTH MANDATE ===

    # Increment counters (only if not forced type)
    if [[ -z "$FORCE_TYPE" ]]; then
        increment_counters
    fi

    # Wait and verify
    sleep 3
    post_log_age=$(get_session_log_age "$session_name")

    if [[ $post_log_age -lt $log_age ]]; then
        log_info "BOOP verified - Claude is responsive"
        reset_failed_count  # Success - reset failure counter
        log_result "$session_name" "success" "$boop_type" "$reason" "$log_age"
        exit 0
    else
        # BOOP sent but no response - track failure
        increment_failed_count
        update_boop_state "$boop_type" "no_response"
        failed_count=$(get_failed_count)
        log_info "BOOP sent but no response (failure $failed_count/$FAILED_BOOP_THRESHOLD)"

        if [[ $failed_count -ge $FAILED_BOOP_THRESHOLD ]]; then
            log_info "Failure threshold reached - checking if Claude is actually active..."

            # Double-check: Is Claude actually stuck or just busy?
            if is_claude_active "$session_name"; then
                log_info "Claude appears active despite no BOOP response - NOT restarting"
                log_result "$session_name" "active_no_restart" "$boop_type" "active_despite_${failed_count}_failures" "$log_age"
                # Reset counter since Claude is working
                reset_failed_count
                exit 0
            fi

            log_info "Claude confirmed inactive - triggering restart"
            log_result "$session_name" "restart_triggered" "$boop_type" "confirmed_unresponsive_${failed_count}" "$log_age"
            restart_iteration "$session_name"
            exit 0
        else
            log_result "$session_name" "no_response" "$boop_type" "failure_${failed_count}" "$log_age"
            exit 0
        fi
    fi
else
    log_info "Failed to send BOOP"
    log_result "$session_name" "failed" "$boop_type" "send_error" "$log_age"
    exit 1
fi
