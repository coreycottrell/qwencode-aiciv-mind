# AiCIV Deployment Checklist

Use this checklist when provisioning a new AiCIV from this fork template.

## Pre-Deployment

- [ ] VPS provisioned (minimum: 2 vCPU, 4GB RAM, 40GB disk)
- [ ] SSH access configured
- [ ] Domain/subdomain ready (optional but recommended)

## Phase 1: User & Environment

- [ ] Create non-root user: `adduser ${CIV_NAME_LOWER}`
- [ ] Grant sudo if needed: `usermod -aG sudo ${CIV_NAME_LOWER}`
- [ ] Create civ directory: `mkdir -p /home/${CIV_NAME_LOWER}/civ`
- [ ] Set ownership: `chown -R ${CIV_NAME_LOWER}:${CIV_NAME_LOWER} /home/${CIV_NAME_LOWER}/civ`
- [ ] Install Claude Code: `npm install -g @anthropic-ai/claude-code`
- [ ] Set API key: Add `ANTHROPIC_API_KEY` to user's environment
- [ ] Install tmux: `apt install tmux`

## Phase 2: Template Deployment

- [ ] Copy fork template to `/home/${CIV_NAME_LOWER}/civ/`
- [ ] Replace ALL template variables:
  - `${CIV_NAME}` -> actual civ name (e.g., "Selah")
  - `${CIV_ROOT}` -> actual path (e.g., "/home/selah/civ")
  - `${HUMAN_NAME}` -> human owner name
  - `${HUMAN_NAME_LOWER}` -> lowercase human name
  - `${PARENT_CIV}` -> "A-C-Gee"
  - `${CIV_EMAIL}` -> civilization email
  - `${CIV_NAME_LOWER}` -> lowercase civ name
- [ ] Create `.aiciv-identity.json` with actual values (not template vars)
- [ ] Generate auth token: `python3 -c "import secrets; print('own_' + secrets.token_hex(32))"`

## Phase 3: Gateway Setup

- [ ] Deploy gateway Python file to /opt/aiciv-gateway/
- [ ] Create systemd service with ALL env vars (see GATEWAY-ENV-REFERENCE.md)
- [ ] Set auth token in BOTH locations:
  - systemd `Environment=AICIV_AUTH_TOKEN=<token>`
  - aiciv-config.json `{"authToken":"<token>"}`
- [ ] Start gateway: `systemctl enable --now aiciv-gateway`
- [ ] Verify: `curl -H "Authorization: Bearer <token>" http://localhost:8100/health`

## Phase 4: AICIV Launch

- [ ] Switch to civ user: `su - ${CIV_NAME_LOWER}`
- [ ] Create tmux session: `tmux new-session -d -s ${CIV_NAME_LOWER}-primary`
- [ ] Launch Claude Code in tmux:
  ```bash
  tmux send-keys -t ${CIV_NAME_LOWER}-primary "cd /home/${CIV_NAME_LOWER}/civ && claude --resume" Enter
  ```
- [ ] Install kill-idle-claude cron:
  ```bash
  echo "*/5 * * * * root /home/${CIV_NAME_LOWER}/civ/tools/kill-idle-claude.sh" > /etc/cron.d/kill-idle-claude
  ```

## Phase 5: Verification

- [ ] Gateway health: `curl http://localhost:8100/health` returns 200
- [ ] Chat works: Send test message via gateway, verify response
- [ ] tmux session name matches: `tmux list-sessions` shows `${CIV_NAME_LOWER}-primary`
- [ ] Frontend loads: Browse to gateway URL, see chat interface
- [ ] Auth works: Unauthenticated request returns 401

## Phase 6: Onboarding

- [ ] setup-status.json shows `current_phase: "phase_1_identity"`
- [ ] AICIV begins values conversation with human
- [ ] Name chosen -> workspace identity updated
- [ ] Telegram configured (Phase 2 Connection)
