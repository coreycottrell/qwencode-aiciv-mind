# AiCIV Gateway Environment Variables

All gateway configuration derives from a single input: **AICIV_NAME**.

## Required Environment Variables

These must be set in the gateway's systemd service file (`/etc/systemd/system/aiciv-gateway.service`):

| Variable | Example | Description |
|----------|---------|-------------|
| `AICIV_CIV_NAME` | `selah` | Display name for the civilization |
| `AICIV_CIV_ROOT` | `/home/selah/civ` | Filesystem root of the civilization |
| `AICIV_TMUX_SESSION` | `selah-primary` | Exact tmux session name (MUST match) |
| `AICIV_TMUX_USER` | `selah` | OS user owning the tmux session |
| `AICIV_PORT` | `8100` | Gateway HTTP listen port |
| `AICIV_HOST` | `127.0.0.1` | Listen address (127.0.0.1 for reverse proxy, 0.0.0.0 for direct) |
| `AICIV_AUTH_TOKEN` | `own_abc123...` | Bearer auth token for API access |
| `AICIV_FRONTEND_HTML` | `selah.html` | Frontend HTML filename |

## Derivation Pattern

Everything derives from the civilization name:

```
AICIV_NAME = "selah"
  -> user:          selah
  -> home:          /home/selah
  -> civ root:      /home/selah/civ
  -> tmux session:  selah-primary
  -> systemd unit:  aiciv-gateway-selah.service
  -> frontend:      selah.html
  -> subdomain:     selah.yourdomain.com
```

## systemd Service Template

```ini
[Unit]
Description=AiCIV Gateway - ${CIV_NAME}
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/opt/aiciv-gateway
Environment=AICIV_CIV_NAME=${CIV_NAME}
Environment=AICIV_CIV_ROOT=${CIV_ROOT}
Environment=AICIV_TMUX_SESSION=${CIV_NAME_LOWER}-primary
Environment=AICIV_TMUX_USER=${CIV_NAME_LOWER}
Environment=AICIV_PORT=8100
Environment=AICIV_HOST=127.0.0.1
Environment=AICIV_AUTH_TOKEN=<generated-token>
Environment=AICIV_FRONTEND_HTML=${CIV_NAME_LOWER}.html
ExecStart=/usr/bin/python3 aiciv_gateway.py
Restart=always
RestartSec=5

[Install]
WantedBy=multi-user.target
```

## Auth Token Dual-Location (CRITICAL GOTCHA)

The auth token must exist in TWO places that MUST match:

1. **systemd environment**: `AICIV_AUTH_TOKEN=<token>` in the service file
   - The gateway reads this to validate incoming API requests

2. **aiciv-config.json**: `{"authToken": "<token>"}` in the frontend config
   - The frontend reads this to include the token in API requests

**If these don't match, every API call returns 401 Unauthorized.**

### Token Rotation Procedure

When rotating tokens:
1. Generate new token
2. Update systemd service: `Environment=AICIV_AUTH_TOKEN=<new-token>`
3. Update config: `echo '{"authToken":"<new-token>"}' > /path/to/aiciv-config.json`
4. Reload systemd: `systemctl daemon-reload && systemctl restart aiciv-gateway-${CIV_NAME}`
5. Verify: `curl -H "Authorization: Bearer <new-token>" http://localhost:${PORT}/health`

### Three-Tier Token Prefixes

| Prefix | Tier | Access Level |
|--------|------|-------------|
| `own_` | Owner | Full admin (chat, terminal, config, shutdown) |
| `usr_` | User | Standard (chat, artifacts, read-only teams) |
| `vwr_` | Viewer | Read-only (dashboard, status) |

## Common Mistakes

1. **tmux session name mismatch**: Gateway crashed because `AICIV_TMUX_SESSION` didn't match the actual tmux session name. Always use `{name}-primary` convention.
2. **Auth token mismatch**: Token in systemd env didn't match token in aiciv-config.json -> 401 on every request.
3. **Running as root**: Claude Code blocks `--dangerously-skip-permissions` as root. The AICIV process must run as a non-root user.
4. **Wrong AICIV_HOST**: Using `0.0.0.0` exposes the gateway directly. Use `127.0.0.1` when behind a reverse proxy.
