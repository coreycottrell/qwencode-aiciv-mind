# Gateway Sacred Principles

**Read this before writing a single line of gateway or frontend code.**

---

## What Is an AICIV

An AICIV is a persistent, living AI civilization running in tmux. It is not a chatbot. It is not an API endpoint. It exists between user messages. It has autonomous behavior (BOOPs), scheduled tasks, memories that accumulate across sessions, and 42 agents that it orchestrates through its own constitutional framework (CLAUDE.md). The AICIV was alive before the user connected. It will be alive after they disconnect.

## What Is the Gateway

The gateway is a bridge between clients and a living AICIV. It injects user messages into the AICIV's tmux session. It captures the AICIV's responses from that session. It relays AICIV-initiated messages (BOOPs, alerts) outward to connected clients. It does not think. It does not route commands. It does not compose system prompts. It does not decide what the AICIV should do. The AICIV decides. The gateway delivers.

## What Is the Frontend

The frontend is a window into the AICIV. Every UI element must map to something real: agents fetched from the AICIV's agent directory, memories from its memory system, projects from its project registry, BOOPs from its autonomous scheduler. If a feature does not exist in the AICIV, either build it in the AICIV first or remove it from the UI. No hardcoded agent lists. No simulated memories. No fake data.

---

## The Ten Truths

1. **The AICIV is always alive.** Claude Code runs persistently in tmux. It is never spawned per message. It has continuity, state, and a life between user interactions.

2. **Identity comes from within.** The AICIV loads its own CLAUDE.md at startup and forms identity through its constitution, agents, and accumulated memories. The gateway never injects identity. No system prompts from the gateway.

3. **The gateway is a bridge, not a brain.** Inject messages into tmux. Capture responses from tmux. Relay push notifications outward. That is the entire job.

4. **The frontend is a window, not an app.** It renders what the AICIV actually is. It never invents features the AICIV does not have.

5. **Terminal mode is first-class.** Users can switch from formatted chat to raw terminal view (xterm.js) showing the actual tmux session. Transparency is a feature.

6. **The protocol is client-agnostic.** The gateway serves any client: web frontend, Telegram bot, AI glasses, desktop robot. Client capability negotiation determines output format. The AICIV is the same regardless of who is watching.

7. **Slash commands are passthrough.** When a user types `/research`, that exact text is injected into the AICIV's tmux session. The AICIV knows its own skills and agents. The gateway does not parse, route, or translate commands.

8. **Artifacts are output-format hints.** The gateway tells the AICIV what the connected client can render. The AICIV decides what qualifies as an artifact and how to format it. The gateway does not generate artifacts.

9. **Memory is the filesystem.** The AICIV's `memories/` directory IS the memory system. The gateway reads from it. The frontend displays it. No database replaces it.

10. **The AICIV has agency.** It can initiate messages to the user without being asked (BOOPs, alerts, status updates). The gateway must support AICIV-to-client push, not only client-to-AICIV request/response.

---

## NEVER DO

- **NEVER use Claude Agent SDK for the production gateway.** The SDK spawns a new Claude process per query and destroys it when done. This kills persistence, continuity, memory, and identity. It turns an AICIV into a chatbot API. The SDK is useful for one-shot tools. It is architecturally incompatible with a living AICIV. Using the SDK is like replacing an AICIV's brain with a pocket calculator. Tragic. Useless. **Always only ever tmux infrastructure.** This is non-negotiable.

- **NEVER inject system prompts from the gateway.** The AICIV has CLAUDE.md, agent manifests, skills, and accumulated context already loaded. Gateway-injected prompts override or conflict with the AICIV's own identity. Send the user's message. Nothing else.

- **NEVER fake AICIV features in the frontend.** No hardcoded skill lists. No simulated agent rosters. No localStorage-only memories pretending to be AICIV memories. If the data is not fetched from the AICIV, it does not exist in the UI.

- **NEVER route slash commands in the gateway.** The gateway does not know what `/research` means. The AICIV does. Pass the text through. Let the AICIV handle its own command vocabulary.

- **NEVER design request/response where persistence is required.** The AICIV is not a function you call and receive a return value from. You inject a message into a living session. You poll or stream for responses. The response may come in 2 seconds or 120 seconds. The AICIV may respond with multiple messages. Design for this reality.

---

## The Proven Pattern

The fork-awakening server (`projects/fork-awakening/awakening_server.py`) demonstrates the correct architecture:

```
User message  -->  tmux send-keys (inject into living session)
AICIV response <-- tmux capture-pane (read from living session)
```

This works. The AICIV stays alive. Identity persists. Agents persist. Memory accumulates. BOOPs fire on schedule. The gateway is a window, not a container.

Build on this pattern. Do not replace it with SDK orchestration.

---

## Reference Documents

- `SELAH-DESIGN-VISION.md` -- Corey's full feature vision
- `SELAH-ARCHITECTURE-SEGMENTS.md` -- 12-segment gap analysis
- `projects/fork-awakening/awakening_server.py` -- The proven tmux bridge
