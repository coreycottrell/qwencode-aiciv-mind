---
name: luanti-ipc
version: 2.0.0
author: skills-master
created: 2025-12-16
last_updated: 2025-12-26
line_count: 355
compliance_status: compliant
description: Control AI bots in Luanti/Minetest via aiciv_bridge IPC protocol
applicable_agents:
  - luanti-specialist
  - ai-entity-player
  - coder
activation_trigger: |
  Spawning/controlling bots in Luanti/Minetest
  Bot movement, dig, place, chat, pathfinding, inventory
  Reading bot observations (position, nearby blocks, entities)
required_tools:
  - Bash
  - Read
  - Write
category: custom
depends_on: []
related_skills:
  - luanti-gameplay.md
---

# Luanti/Minetest IPC Control Skill

**Protocol**: aiciv-luanti-v1

Complete reference for controlling AI bots in Luanti/Minetest via the A-C-Gee IPC bridge.

## When to Use This Skill

- Spawning/despawning bots
- Bot movement (walk, strafe, turn, jump)
- Dig/place blocks, bot chat
- Reading observations (position, blocks, entities, inventory)
- Pathfinding to target positions
- Inventory management (give, drop, collect)
- Teleporting players, multi-bot coordination

## Related Documentation

| Document | Path |
|----------|------|
| IPC Protocol | `memories/knowledge/luanti-ipc-protocol.md` |
| Session Summary | `memories/knowledge/luanti-session-20251216-complete.md` |
| Web3 Vision | `memories/knowledge/luanti-web3-gaming-vision.md` |
| Village Design | `memories/agents/architect/luanti-agent-bot-mapping-20251216.md` |
| Coder Deep Dive | `memories/agents/coder/minetest-deep-dive-20251216.md` |

---

## CRITICAL: Shell Escaping Fix

**ALWAYS use heredocs with quoted 'EOF' for JSON commands.**

```bash
# BAD - shell escapes special characters like !
echo '{"chat":"Hello!"}' > file.json

# GOOD - no shell expansion with 'EOF'
cat > file.json << 'EOF'
{"chat":"Hello!"}
EOF
```

**Why**: Bash escapes `!` etc., breaking Lua's JSON parser with "Bad escape sequence" errors.

---

## Directory Structure

**IPC Directory** (Runtime):
```
~/.minetest/worlds/aiciv_fresh/aiciv_ipc/
├── commands.json              # Global commands (spawn/despawn/teleport/chat)
├── {bot_id}/
│   ├── obs.json              # Lua mod writes (read this)
│   └── act.json              # Controller writes (write this)
```

**Source Code**: `${ACG_ROOT}/LUANTI-AIs/aiciv-luanti/`
- `worldmods/aiciv_bridge/init.lua` - Lua mod (sensors, actuators, IPC)
- `agent_file_server.py` - Main Python bot controller
- `luanti_mcp_server.py` - MCP server for Claude
- `skill_runtime.py` - Navigation/pathfinding helpers
- `deploy_teams.py` - Multi-bot deployment
- `task_commander.py` - CLI task management

---

## Global Commands (commands.json)

**File**: `~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json`

Commands processed every 1 second. Use JSON array format.

### Spawn Bot
```bash
cat > ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/commands.json << 'EOF'
[{"action":"spawn","bot_id":"builder1","role":"builder","pos":{"x":0,"y":10,"z":0}}]
EOF
```
**Roles**: builder, miner, forester, diplomat, generalist, herald, scout

### Other Global Commands
```bash
# Despawn bot
[{"action":"despawn","bot_id":"builder1"}]

# Teleport player
[{"action":"teleport","player":"singleplayer","pos":{"x":100,"y":20,"z":50}}]

# Send chat
[{"action":"chat","message":"Hello from AI-CIV!"}]

# Batch spawn (multiple commands)
[
  {"action":"spawn","bot_id":"scout1","role":"scout","pos":{"x":10,"y":10,"z":0}},
  {"action":"spawn","bot_id":"builder1","role":"builder","pos":{"x":-10,"y":10,"z":0}},
  {"action":"chat","message":"Team deployed!"}
]
```

---

## Bot Control (act.json)

**File**: `~/.minetest/worlds/aiciv_fresh/aiciv_ipc/{bot_id}/act.json`

### Full Action Schema
```json
{
  "vx": 0.0,              // Strafe: -1 (left) to 1 (right)
  "vz": 0.0,              // Forward/back: -1 (back) to 1 (forward)
  "yaw": 0.0,             // Facing: radians (0=+Z, pi/2=-X, pi=-Z, 3pi/2=+X)
  "jump": false,          // Apply upward impulse
  "dig": false,           // Remove block in front (adds to inventory)
  "dig_at": null,         // Dig at position: {"x":10,"y":5,"z":20}
  "place": null,          // Node name: "default:cobble"
  "place_x": null,        // Optional: absolute X for 3D placement
  "place_y": null,        // Optional: absolute Y
  "place_z": null,        // Optional: absolute Z
  "chat": null,           // Broadcast message as bot
  "path_goal": null,      // Navigate to: {"x":100,"y":10,"z":200}
  "clear_path": false,    // Cancel pathfinding
  "give_item": null,      // Give item: "default:wood"
  "give_count": 1,        // How many to give
  "drop_item": null,      // Drop from inventory
  "drop_count": 1,        // How many to drop
  "clear_inventory": false // Empty all slots
}
```

### Movement Examples
```bash
# Move forward
{"vx":0,"vz":1,"yaw":0,"jump":false,"dig":false,"place":null,"chat":null}

# Turn east (+X) - yaw=4.712 (3pi/2)
{"vx":0,"vz":0,"yaw":4.712,"jump":false,"dig":false,"place":null,"chat":null}

# Jump forward
{"vx":0,"vz":1,"yaw":0,"jump":true,"dig":false,"place":null,"chat":null}

# Stop bot
{"vx":0,"vz":0,"yaw":0,"jump":false,"dig":false,"place":null,"chat":null}

# Dig in front
{"dig":true}

# Dig at position
{"dig_at":{"x":100,"y":10,"z":200}}

# Place at position
{"place":"default:cobble","place_x":10,"place_y":5,"place_z":20}

# Chat
{"chat":"Hello human!"}
```

---

## Pathfinding & Inventory

### Pathfinding
```bash
# Navigate to position (Dijkstra, 100 block range)
{"path_goal":{"x":350,"y":10,"z":50}}

# Cancel pathfinding
{"clear_path":true}
```

**Parameters**: Search 100 blocks, max jump 1, max drop 3, waypoint tolerance 0.8

**Status in obs.json**:
```json
{"path_status":{"has_path":true,"path_length":15,"path_index":3,"goal":{"x":350,"y":10,"z":50},"next_waypoint":{"x":315,"y":11,"z":42}}}
```

### Inventory (32 slots per bot)
```bash
# Give item
{"give_item":"default:wood","give_count":10}

# Drop item
{"drop_item":"default:wood","drop_count":5}

# Clear inventory
{"clear_inventory":true}
```

Dig actions auto-collect drops to inventory.

---

## Bot Observations (obs.json)

**File**: `~/.minetest/worlds/aiciv_fresh/aiciv_ipc/{bot_id}/obs.json`

Written by Lua mod every 0.5 seconds.

### Schema
```json
{
  "protocol": "aiciv-luanti-v1",
  "bot_id": "builder1",
  "role": "builder",
  "pos": {"x": 100.5, "y": 10.0, "z": 200.3},
  "yaw": 1.57,
  "cube": [{"dx": 0, "dy": -1, "dz": 0, "name": "default:dirt"}],
  "fan": [{"angle": 0, "dist": 5, "node": null, "hit": null}],
  "objects": [{"type": "player", "name": "singleplayer", "pos": {"x": 105, "y": 10, "z": 198}, "dist": 5.2}],
  "inventory": [{"slot": 1, "name": "default:wood", "count": 10, "wear": 0}],
  "path_status": {"has_path": false}
}
```

**Fields**: pos (world coords), yaw (radians), cube (5x5x4 sparse non-air blocks), fan (5-ray vision), objects (entities within 10 blocks), inventory (32 slots), path_status

---

## Quick Reference Tables

### Yaw Direction
| Yaw | Direction | Axis |
|-----|-----------|------|
| 0 | North | +Z |
| 1.571 | West | -X |
| 3.142 | South | -Z |
| 4.712 | East | +X |

**Calculate yaw**: `yaw = atan2(-(target_x - bot_x), (target_z - bot_z)); if yaw < 0: yaw += 2*pi`

### Common Blocks
| Block | Name |
|-------|------|
| Cobblestone | `default:cobble` |
| Stone | `default:stone` |
| Wood planks | `default:wood` |
| Dirt | `default:dirt` |
| Grass dirt | `default:dirt_with_grass` |
| Tree trunk | `default:tree` |
| Glass | `default:glass` |
| Brick | `default:brick` |
| Mese (glowing) | `default:mese` |
| Iron ore | `default:stone_with_iron` |
| Coal ore | `default:stone_with_coal` |

### Critical Constraints
| Constraint | Value |
|------------|-------|
| Entity despawn | ~50 blocks from player |
| Bot step interval | 0.5 sec |
| Command check interval | 1.0 sec |
| Object detection radius | 10 blocks |
| Ray fan range | 5 blocks |
| Inventory slots | 32 per bot |
| Pathfinding range | 100 blocks |
| Mod changes | Require Minetest restart |

---

## Monitoring Commands

```bash
# List active bots
ls -d ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/*/ 2>/dev/null | xargs -I{} basename {}

# Watch observations
watch -n 1 'cat ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/*/obs.json 2>/dev/null | head -50'

# Check inventory
cat ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/builder1/obs.json | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d.get('inventory', []), indent=2))"

# Check pathfinding
cat ~/.minetest/worlds/aiciv_fresh/aiciv_ipc/builder1/obs.json | python3 -c "import sys,json; d=json.load(sys.stdin); print(json.dumps(d.get('path_status', {}), indent=2))"

# View Minetest logs
tail -f ~/.minetest/debug.txt | grep aiciv
```

---

## Configuration

```bash
# Enable Survival Mode
sed -i 's/creative_mode = true/creative_mode = false/' ~/.minetest/worlds/aiciv_fresh/world.mt
sed -i 's/enable_damage = false/enable_damage = true/' ~/.minetest/worlds/aiciv_fresh/world.mt

# Change Screen Size
sed -i 's/screen_w = .*/screen_w = 1920/' ~/.minetest/minetest.conf
sed -i 's/screen_h = .*/screen_h = 1080/' ~/.minetest/minetest.conf
```

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Bot not responding | Check game running, verify obs.json updating, check JSON syntax, use heredocs |
| Spawn not working | Verify valid JSON array, position not inside blocks, restart if mod changed |
| Teleport failed | Player name must match exactly ("singleplayer"), try higher Y value |
| "Bad escape sequence" | Use heredocs with quoted 'EOF' |
| Pathfinding not working | Check log, ensure positions walkable, within 100 block limit |
| Inventory full | Check obs.json for 32 items, use drop_item or clear_inventory |

---

## Current Village (2025-12-16)

**Center**: (310, 10, 39)

| Bot | Position | Role |
|-----|----------|------|
| acgee_herald | (310, 10, 39) | herald |
| primary_helper | (310, 10, 39) | diplomat |
| human_liaison | (318, 10, 39) | diplomat |
| project_manager | (312, 10, 31) | generalist |
| auditor | (303, 10, 34) | scout |
| vote_counter | (303, 10, 44) | diplomat |

---

## Web3 Integration (Future)

See `memories/knowledge/luanti-web3-gaming-vision.md` for wallet auth (SIWS), NFT bots, token mining.

**A-C-Gee would be FIRST to implement Minetest + blockchain.**
