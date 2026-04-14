# Netlify CLI Skill

**Purpose**: Deploy sites with full function support via Netlify CLI
**When to use**: Deploying sageandweaver-network or any site with Netlify Functions

---

## Critical: Auth Requires Human

The Netlify CLI opens a browser for OAuth authorization. **${HUMAN_NAME} must complete auth manually.**

When you see:
```
Opening https://app.netlify.com/authorize?...
Waiting for authorization...
```

**Tell ${HUMAN_NAME} to complete auth in his browser.**

---

## Why CLI vs API Deploy Script

| Method | Static Files | Functions | Auth |
|--------|--------------|-----------|------|
| `netlify_api_deploy.py` | ✅ | ❌ NO | Token (automatic) |
| `netlify deploy --prod` | ✅ | ✅ YES | Browser (manual) |

**Use CLI when deploying functions.** The API script only handles static files.

---

## Commands

### Deploy Production (with functions)
```bash
cd ${ACG_ROOT}/sageandweaver-network
/usr/local/bin/netlify deploy --prod
```

### Deploy Preview
```bash
/usr/local/bin/netlify deploy
```

### Check Status
```bash
/usr/local/bin/netlify status
```

### View Function Logs
```bash
/usr/local/bin/netlify functions:log
```

---

## Site Linking

Sites need a `.netlify/state.json` file:
```json
{"siteId":"7e89a1b0-172a-4d48-b191-c7d9dcc452f2"}
```

Or link interactively:
```bash
/usr/local/bin/netlify link
```

---

## Troubleshooting

### "UNC paths are not supported"
Using Windows CLI instead of Linux. Use `/usr/local/bin/netlify` explicitly.

### Functions returning 404
Functions weren't deployed. Use CLI deploy, not API script.

### Auth timeout
${HUMAN_NAME} needs to complete browser auth within ~60 seconds.

### Node version warnings
Current system Node (v18) is below required (v20). Functions still work but may see warnings.

---

## Learned: 2026-01-08

- API deploy script (`netlify_api_deploy.py`) doesn't deploy functions
- Comments system broke because functions weren't being redeployed
- Always use CLI for sites with Netlify Functions
- Linux CLI installed at `/usr/local/bin/netlify` (via `sudo npm install -g netlify-cli`)

---

**Tags**: netlify, deployment, functions, cli
