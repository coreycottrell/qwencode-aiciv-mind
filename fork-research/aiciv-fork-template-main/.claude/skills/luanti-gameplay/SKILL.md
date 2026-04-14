---
name: luanti-gameplay
version: 1.0.0
author: coder-agent
created: 2025-12-16
last_updated: 2025-12-26
line_count: 363
compliance_status: compliant

description: |
  Autonomous Minetest gameplay combining desktop vision and bot IPC control.
  Use when playing Minetest as Claude - launching game, controlling player
  via GUI, managing bots via IPC, building structures, exploring terrain.

applicable_agents:
  - primary
  - ai-entity-player
  - luanti-specialist

activation_trigger: |
  Load this skill when:
  - Playing Minetest autonomously (vision + controls)
  - Managing bots while watching game visually
  - Building structures with coordinated bot teams

required_tools:
  - Bash
  - mcp__desktop-automation__keyboard_press
  - mcp__desktop-automation__keyboard_type

category: custom
depends_on:
  - desktop-vision
  - luanti-ipc

related_skills:
  - vision/vision-action-loop
---

# Luanti/Minetest Autonomous Gameplay Skill

Complete reference for autonomous Minetest gameplay combining vision-based GUI control with IPC bot management.

## When to Use This Skill

**Trigger scenarios:**
- Playing Minetest autonomously (vision + controls)
- Launching and managing the game window
- Controlling player movement via keyboard
- Managing bots while watching game visually
- Building structures with coordinated bot teams

## Foundational Skills (MUST READ)

This skill combines two foundational skills:

| Skill | Path | Purpose |
|-------|------|---------|
| Desktop Vision | `.claude/skills/desktop-vision/SKILL.md` | Screenshots, mouse/keyboard control |
| Luanti IPC | `.claude/skills/luanti-ipc/SKILL.md` | Bot spawning, actions, observations |

**Read these first if unfamiliar with either system.**

## Quick Start: Launch Game and Play

### Step 1: Launch Minetest

```bash
DISPLAY=:0 /usr/games/minetest --world ~/.minetest/worlds/aiciv_fresh --name ACGee --go &
```

**Parameters:** `DISPLAY=:0` (X display for WSL2), `--go` (skip menu)

### Step 2: Wait and Focus Window

```bash
sleep 5  # Give game time to start
```

Then focus window using PowerShell Win32 API (see Common Operations below).

### Step 3: Take Screenshot to See Game

```bash
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
```

Read the returned image path to see current game state.

### Step 4: Grant Player Privileges (For Teleport)

Check/add privileges in auth.txt:
```bash
cat ~/.minetest/worlds/aiciv_fresh/auth.txt
# Format: player_name:password_hash:privileges
# Example: ACGee::interact,shout,teleport,settime,fly,fast
```

Or use in-game: `/grant ACGee teleport`

## Core Gameplay Loop

### Vision-Action Cycle

```
1. SCREENSHOT  →  See current game state
2. ANALYZE     →  Identify what to do (where am I? what's nearby?)
3. ACT         →  Send keyboard/mouse commands OR bot IPC
4. VERIFY      →  Screenshot again to confirm action worked
5. REPEAT      →  Continue gameplay loop
```

### Example: Navigate to Location

```bash
# 1. See current position
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot

# 2. Open chat and teleport
# MCP: keyboard_press(key="t")
# MCP: keyboard_type(text="/teleport 310 15 39")
# MCP: keyboard_press(key="enter")

# 3. Verify arrival
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
```

## Player Control (Keyboard/Mouse)

### Movement Keys

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| W | Move forward | E | Open inventory |
| S | Move backward | Q | Drop item |
| A | Strafe left | Space | Jump |
| D | Strafe right | Shift | Sneak/slow |

### Chat Commands (Press T first)

| Command | Effect |
|---------|--------|
| `/teleport X Y Z` | Teleport to coordinates |
| `/time 6000` | Set to noon |
| `/give ACGee default:mese 99` | Give items |
| `/grant ACGee fly` | Grant privilege |

### Function Keys

| Key | Action | Key | Action |
|-----|--------|-----|--------|
| F1 | Hide HUD | F10 | Show console |
| F5 | Debug info | F11 | Fullscreen |
| F7 | Camera mode | Esc | Pause menu |

## Bot Control (IPC)

### Directory Structure

```
~/.minetest/worlds/aiciv_fresh/aiciv_ipc/
├── commands.json              # Global: spawn/despawn/teleport
├── {bot_id}/
│   ├── obs.json              # Read: bot observations
│   └── act.json              # Write: bot actions
```

### Critical: Shell Escaping

**ALWAYS use heredocs with quoted 'EOF' for JSON:**

```bash
# GOOD - no shell expansion
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"spawn","bot_id":"builder1","role":"builder","pos":{"x":310,"y":10,"z":39}}]
EOF
```

### Spawn/Despawn Bot

```bash
# Spawn
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"spawn","bot_id":"scout1","role":"scout","pos":{"x":310,"y":10,"z":39}}]
EOF

# Despawn
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"despawn","bot_id":"scout1"}]
EOF
```

### Move/Stop Bot

```bash
# Move forward
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/scout1/act.json << 'EOF'
{"vx":0,"vz":1,"yaw":0,"jump":false,"dig":false,"place":null,"chat":null}
EOF

# Stop
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/scout1/act.json << 'EOF'
{"vx":0,"vz":0,"yaw":0,"jump":false,"dig":false,"place":null,"chat":null}
EOF
```

### Read Bot Position

```bash
cat ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/scout1/obs.json | python3 -m json.tool
```

## Common Operations Quick Reference

### Take Screenshot (Reliable)

```bash
python3 ${ACG_ROOT}/tools/autonomous_control.py screenshot
# Returns path like: /mnt/c/temp/claude_screenshots/screenshot_20251216_143000_123.png
```

### Focus Minetest Window (PowerShell)

```bash
powershell.exe -Command "
Add-Type @'
using System;
using System.Runtime.InteropServices;
using System.Text;
public class Win32 {
    public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
    [DllImport(\"user32.dll\")]
    public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
    [DllImport(\"user32.dll\")]
    public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);
    [DllImport(\"user32.dll\")]
    public static extern bool SetForegroundWindow(IntPtr hWnd);
    [DllImport(\"user32.dll\")]
    public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
}
'@

[Win32]::EnumWindows({
    param(\$hwnd, \$lparam)
    \$title = New-Object System.Text.StringBuilder 256
    [Win32]::GetWindowText(\$hwnd, \$title, 256)
    if (\$title.ToString() -like '*Minetest*') {
        [Win32]::ShowWindow(\$hwnd, 3)
        [Win32]::SetForegroundWindow(\$hwnd)
    }
    return \$true
}, [IntPtr]::Zero)
"
```

### Teleport Player (Via IPC)

```bash
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"teleport","player":"ACGee","pos":{"x":310,"y":15,"z":39}}]
EOF
```

### Spawn Team of Bots

```bash
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[
  {"action":"spawn","bot_id":"builder1","role":"builder","pos":{"x":310,"y":10,"z":39}},
  {"action":"spawn","bot_id":"builder2","role":"builder","pos":{"x":312,"y":10,"z":39}},
  {"action":"spawn","bot_id":"miner1","role":"miner","pos":{"x":308,"y":10,"z":39}}
]
EOF
```

### List Active Bots

```bash
ls -d ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/*/ 2>/dev/null | xargs -I{} basename {}
```

## Critical Constraints

| Constraint | Impact | Workaround |
|------------|--------|------------|
| Bots despawn ~50 blocks from player | Bots disappear if player moves away | Teleport player to bot area first |
| Terrain only generates near players | Bots can't explore unloaded chunks | Player must lead exploration |
| MCP screen_capture unreliable | May not return visible images | Use PowerShell workaround |
| Bot step interval: 0.5s | Actions take time to execute | Wait between commands |
| Command check interval: 1.0s | Global commands processed slowly | Don't spam commands.json |
| Motor control (vx/vz) works immediately | Movement is reliable | Prefer motor over pathfinding |

## Troubleshooting

### Game Not Launching?

```bash
pgrep -a minetest   # Check if running
pkill minetest      # Kill existing
DISPLAY=:0 /usr/games/minetest --world ~/.minetest/worlds/aiciv_fresh --name ACGee --go &
```

### Teleport Not Working?

```bash
grep ACGee ~/.minetest/worlds/aiciv_fresh/auth.txt  # Check privileges
```

Add privilege if missing (edit auth.txt or use /grant in-game).

### Bots Not Responding?

1. Is game running? (not paused)
2. Is player nearby? (bots despawn at 50 blocks)
3. Check obs.json updating: `ls -la ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/*/obs.json`
4. Verify JSON syntax in act.json/commands.json

### "Bad escape sequence" Error?

Shell is escaping special characters. Use heredocs with quoted 'EOF'.

## Integration Patterns

### Vision + Bot Coordination

```
1. Screenshot to see terrain
2. Identify build location visually
3. Spawn bots near that location via IPC
4. Issue build commands via act.json
5. Screenshot to verify progress
```

### Building Structure

```
1. Teleport player to build site
2. Screenshot to assess terrain
3. Spawn builder bots at corners
4. Issue place commands with coordinates
5. Verify each layer with screenshots
```

## Village Center (Current)

**Location**: (310, 10, 39)

When starting a session, teleport here to rejoin existing bots:
```bash
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"teleport","player":"ACGee","pos":{"x":310,"y":15,"z":39}}]
EOF
```

## Best Practices

1. **Always screenshot first** - Know game state before acting
2. **Keep player near bots** - Prevents despawn at 50 blocks
3. **Use heredocs for JSON** - Avoids shell escaping bugs
4. **Focus window before keyboard** - Commands go to focused app
5. **Verify actions with screenshots** - Confirm commands worked
6. **Use motor control over pathfinding** - More reliable in practice
7. **Process commands.json slowly** - 1 second between batches
