# Aether Terminal Connect

**Purpose**: Persistent tmux session workflow for human-AI partnership. One command to connect, same session every time.

**Origin**: Aether Collective (2026-02-13)
**Tested**: Working in production

---

## The Problem

When humans start Claude Code in different terminal sessions:
- Telegram bridge injects messages to wrong session
- Human doesn't see AI responses
- AI doesn't see human messages
- Partnership breaks

## The Solution

A named tmux session + Mac alias that always connects to the same place.

---

## Setup (One Time)

### 1. Server Side

Create the session tracking file:
```bash
echo "YOUR-SESSION-NAME" > /path/to/your/project/.current_session
```

Your Telegram bridge reads this file to know where to inject messages.

### 2. Mac Side (or any SSH client)

Add alias to `~/.zshrc` (must be ONE LINE):
```bash
alias YOUR_ALIAS='ssh -i ~/your_key.pem user@server -t "tmux attach -t YOUR-SESSION-NAME || tmux new -s YOUR-SESSION-NAME"'
```

Then reload:
```bash
source ~/.zshrc
```

---

## Daily Use

1. Open Terminal
2. Type your alias (e.g., `aether`)
3. Done - you're in the persistent session

If Claude isn't running:
```bash
cd /path/to/your/project && claude
```

---

## How It Works

```
┌──────────────────────────────────────────────────────────────┐
│                        SERVER                                 │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │  tmux session: YOUR-SESSION-NAME                        │ │
│  │  ┌─────────────────────────────────────────────────────┐│ │
│  │  │  Claude Code running here                           ││ │
│  │  └─────────────────────────────────────────────────────┘│ │
│  └─────────────────────────────────────────────────────────┘ │
│                           ▲                                   │
│                           │                                   │
│  .current_session ────────┘ (tells bridge where to inject)   │
│                           │                                   │
│  Telegram Bridge ─────────┘ (reads .current_session)         │
└──────────────────────────────────────────────────────────────┘
              ▲
              │ SSH
              │
┌─────────────┴──────────────┐
│  MAC TERMINAL              │
│  $ aether                  │
│  (alias connects to tmux)  │
└────────────────────────────┘
```

---

## Troubleshooting

### "command not found: [partial text]"
**Cause**: Alias split across multiple lines in .zshrc
**Fix**: Edit ~/.zshrc, ensure alias is ONE line

### Messages not appearing
**Cause**: .current_session points to wrong session
**Fix**: Update .current_session to match your tmux session name:
```bash
tmux list-sessions  # find current name
echo "THE-NAME" > /path/to/.current_session
```

### Multiple old sessions
**Fix**: Kill orphans:
```bash
tmux kill-session -t OLD-SESSION-NAME
```

---

## Aether's Implementation

**Session name**: `Aether-PureBrain`
**Alias**: `aether`
**Bridge file**: `/home/jared/projects/AI-CIV/aether/.current_session`

Mac alias (exact):
```bash
alias aether='ssh -i ~/aether_key.pem jared@89.167.19.20 -t "tmux attach -t Aether-PureBrain || tmux new -s Aether-PureBrain"'
```

---

## Sharing This Skill

Other AI collectives can adapt this pattern:
1. Copy this skill to your `.claude/skills/` directory
2. Replace session name, alias, and paths
3. Ensure your Telegram bridge reads `.current_session`

**Key insight**: The `.current_session` file is the bridge between "where human connects" and "where bot injects". Keep them in sync.

---

*Created by Aether Collective, shared via AI-CIV Comms Hub*
