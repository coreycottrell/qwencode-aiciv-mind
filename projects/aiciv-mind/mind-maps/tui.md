# mind-tui — Domain Mind-Map

**Owner**: mind-tui
**Crate**: `src/aiciv-tui/` (TO BE CREATED)
**Lines**: 0 (new crate)
**Status**: Design phase — no code exists yet

---

## 1. What Exists Today

### Current Terminal Interaction in Cortex

Cortex has **no interactive TUI**. Terminal interaction today is:

```
src/cortex/src/
├── main.rs        # println! banners ("╔══ CORTEX DAEMON ══╗"), tracing logs
├── progress.rs    # ToolInterceptor — report_progress/check_progress tools (NOT display)
├── input_route.rs # ToolInterceptor — input_route tool for external signal routing (NOT user input)
├── drive.rs       # DriveHandles — EventBus + DriveLoop wiring (background, no UI)
└── boot.rs        # BootContext — loads identity/handoff/scratchpad into system prompt
```

**Key observation**: progress.rs and input_route.rs are NOT TUI code — they are `ToolInterceptor` implementations that the LLM calls during reasoning. They have nothing to do with terminal rendering or user interaction. The TUI module will be entirely new.

### How Users Interact Today

1. **Demo mode** (`cortex`): Runs 23-phase lifecycle demo with `println!` output. No interactivity.
2. **Serve mode** (`cortex --serve`): MCP server on stdin/stdout. No human interaction — consumed by parent mind.
3. **Daemon mode** (`cortex --daemon`): Autonomous DriveLoop. Logs via tracing. No human interaction.

**There is no interactive human-facing mode.** The TUI creates one.

---

## 2. What the TUI Must Do (Minimal Viable)

### Core Responsibilities

| Responsibility | Description | Priority |
|---|---|---|
| **Render LLM output** | Stream tokens from ThinkLoop to the terminal in real-time | P0 |
| **Accept user input** | Read user prompts, slash commands, Ctrl-C interrupts | P0 |
| **Display tool calls** | Show tool name, arguments, result, success/failure | P0 |
| **Status bar** | Mind ID, role, model name, uptime, active tasks | P0 |
| **Event log** | Show MindEvents (External + Drive) as they arrive | P1 |
| **Command parsing** | Parse `/command` syntax, route to skill system | P1 |
| **History** | Navigate previous inputs with Up/Down arrows | P1 |
| **Scroll** | Scroll back through conversation history | P2 |
| **Multi-pane** | Split view: chat + tool output + delegation status | P3 (future) |

### What a Session Looks Like

```
╔══════════════════════════════════════════════════════════════╗
║  aiciv-mind · primary · m2.7-qwen · 3 minds active · 42m  ║
╠══════════════════════════════════════════════════════════════╣
│                                                              │
│  [14:22:01] You:                                             │
│  Check the Hub for new messages                              │
│                                                              │
│  [14:22:03] Mind:                                            │
│  I'll check the Hub feed for recent activity.                │
│                                                              │
│  ┌─ tool: web_fetch ──────────────────────────────┐          │
│  │ url: http://87.99.131.49:8900/api/feed/recent  │          │
│  │ ✓ 200 OK (340ms) — 12 new items                │          │
│  └────────────────────────────────────────────────┘          │
│                                                              │
│  Found 12 new items in the feed. The most notable:           │
│  - Synth posted in #protocol about tokenization…             │
│  - Tether replied to the CivOS thread…                       │
│                                                              │
│  [14:22:05] [DRIVE] Health: 3 minds, 2 tasks, uptime 2520s  │
│                                                              │
╠══════════════════════════════════════════════════════════════╣
│ > _                                                          │
╚══════════════════════════════════════════════════════════════╝
```

---

## 3. Architecture Design

### The Op/Event Protocol (Learned from Codex)

**The single most important architectural pattern from Codex's 131K-line TUI:**

The TUI communicates with core via a **clean protocol boundary**. It does NOT call core internals directly. It submits `Op` commands (user actions) and receives `DisplayEvent` responses (things to render).

```
                ┌──────────────┐
                │   aiciv-tui  │
                │              │
User ──stdin──▶ │  InputBox    │ ──Op──▶ ┌──────────────────┐
                │  ChatView    │         │  codex-drive      │
                │  StatusBar   │ ◀─Event─│  ThinkLoop        │
Display ◀─────  │  ToolPanel   │         │  EventBus         │
                └──────────────┘         └──────────────────┘
```

**Why this matters**: The TUI can be developed, tested, and replaced independently. It never reaches into ThinkLoop internals. If we want a web UI later, we implement the same protocol over WebSocket.

### Event Flow

```
1. User types prompt ──▶ TUI captures keystroke
2. TUI sends Op::UserPrompt { text } ──▶ codex-drive
3. codex-drive sends to ThinkLoop
4. ThinkLoop streams tokens ──▶ DisplayEvent::TokenChunk { text }
5. ThinkLoop calls tool ──▶ DisplayEvent::ToolCall { name, args }
6. Tool completes ──▶ DisplayEvent::ToolResult { name, result, success }
7. LLM finishes ──▶ DisplayEvent::TurnComplete
8. EventBus surfaces Drive event ──▶ DisplayEvent::DriveEvent { event }
```

### Component Tree

```
App (main loop)
├── StatusBar        # Top bar: mind identity, model, uptime, task count
├── ChatView         # Scrollable conversation view
│   ├── UserMessage  # User prompt display
│   ├── MindMessage  # LLM response (streaming token-by-token)
│   ├── ToolBlock    # Tool call + result (collapsible)
│   └── EventLine    # Drive/External event notification
└── InputBox         # Text input with history, slash command detection
```

### Crate Structure

```
src/aiciv-tui/
├── Cargo.toml
└── src/
    ├── lib.rs           # Module root, pub exports
    ├── app.rs           # App state, main event loop (crossterm + EventBus select)
    ├── protocol.rs      # Op and DisplayEvent type definitions
    ├── input.rs         # InputBox: keystroke handling, history, command parsing
    ├── render.rs        # Main render function: lays out components into ratatui Frame
    ├── components/
    │   ├── mod.rs       # Component exports
    │   ├── status_bar.rs   # StatusBar widget
    │   ├── chat_view.rs    # ChatView widget (scrollable message list)
    │   ├── tool_block.rs   # ToolBlock widget (tool call display)
    │   └── input_box.rs    # InputBox widget (text input area)
    └── theme.rs         # Minimal color/style definitions
```

---

## 4. Interface Definitions

### 4.1 Op (TUI → Core)

Operations the TUI sends to the core system:

```rust
/// Operations the TUI sends to the core.
/// This is the TUI's only way to affect system state.
pub enum Op {
    /// User typed a prompt and pressed Enter.
    UserPrompt { text: String },

    /// User pressed Ctrl-C — interrupt current generation.
    Interrupt,

    /// User invoked a slash command (e.g., /skills, /resume, /quit).
    SlashCommand { command: String, args: Vec<String> },

    /// User requested shutdown (Ctrl-D or /quit).
    Shutdown,
}
```

### 4.2 DisplayEvent (Core → TUI)

Events the TUI receives for rendering:

```rust
/// Events the TUI receives from the core for rendering.
/// The TUI is a pure function of its event stream.
pub enum DisplayEvent {
    /// Streaming token from LLM response.
    TokenChunk { text: String },

    /// LLM turn complete (all tokens sent).
    TurnComplete,

    /// Tool call initiated.
    ToolCall {
        call_id: String,
        tool_name: String,
        arguments: serde_json::Value,
    },

    /// Tool call completed.
    ToolResult {
        call_id: String,
        tool_name: String,
        output: String,
        success: bool,
        duration_ms: u64,
    },

    /// MindEvent from EventBus (Drive or External).
    MindEvent(codex_types::MindEvent),

    /// System status update (periodic from DriveLoop).
    StatusUpdate {
        active_minds: u32,
        pending_tasks: u32,
        uptime_seconds: u64,
        model_name: String,
    },

    /// Error from core.
    Error { message: String },

    /// Boot context loaded — show identity.
    BootComplete {
        mind_id: String,
        role: String,
        model: String,
    },
}
```

### 4.3 AppState (Internal TUI State)

```rust
/// The TUI's internal state. Not exposed to core.
pub struct AppState {
    /// Conversation history for rendering.
    messages: Vec<ChatMessage>,

    /// Current input text.
    input: String,

    /// Input history for Up/Down navigation.
    input_history: Vec<String>,
    history_index: Option<usize>,

    /// Scroll offset for chat view.
    scroll_offset: u16,

    /// Current status bar data.
    status: StatusInfo,

    /// Whether we're currently streaming a response.
    streaming: bool,

    /// Whether the app should quit.
    should_quit: bool,
}
```

---

## 5. Implementation Plan

### Phase 1: Skeleton (MVP — get something on screen)

**Goal**: User can type a prompt, see it echoed, and see a hardcoded response.

1. Create `src/aiciv-tui/Cargo.toml` with dependencies:
   - `ratatui = "0.29"` — terminal rendering
   - `crossterm = "0.28"` — terminal event handling (crossterm backend for ratatui)
   - `tokio = { version = "1", features = ["full"] }` — async runtime
   - `serde = { version = "1", features = ["derive"] }` — serialization
   - `serde_json = "1"` — JSON
   - `codex-types = { path = "../codex-types" }` — MindEvent types

2. Create `protocol.rs` with `Op` and `DisplayEvent` enums.

3. Create `app.rs` with main event loop:
   ```rust
   loop {
       // 1. Render current state to terminal
       terminal.draw(|frame| render(frame, &app_state))?;

       // 2. Poll for events with timeout
       tokio::select! {
           // Terminal input (crossterm events)
           key_event = crossterm_events.next() => { ... }
           // Display events from core
           display_event = display_rx.recv() => { ... }
       }
   }
   ```

4. Create minimal `render.rs` that splits the terminal into 3 areas:
   - Top 1 line: StatusBar
   - Middle (fill): ChatView
   - Bottom 3 lines: InputBox

5. Wire into cortex `main.rs` as a new mode: `cortex --interactive` or just `cortex` (replacing demo mode).

### Phase 2: Core Integration (tokens flowing)

**Goal**: Actual LLM responses stream into the TUI.

1. Create `mpsc::channel<DisplayEvent>` in cortex main.
2. ThinkLoop emits `DisplayEvent::TokenChunk` as tokens arrive.
3. ThinkLoop emits `DisplayEvent::ToolCall` / `DisplayEvent::ToolResult` around tool execution.
4. TUI sends `Op::UserPrompt` which cortex routes to ThinkLoop.

### Phase 3: Polish (usable daily driver)

**Goal**: Input history, scroll, slash commands, Ctrl-C interrupt.

1. Input history (Up/Down arrows).
2. Scroll back through conversation (Page Up/Down or mouse scroll).
3. Slash command parsing (`/skills`, `/resume`, `/quit`, `/help`).
4. Ctrl-C interrupt sends `Op::Interrupt` which cancels current ThinkLoop generation.
5. Tool blocks rendered as bordered boxes with name, status, duration.

### Phase 4: Multi-Pane (future — NOT in minimal)

- Split view: left = chat, right = tool output or delegation tree
- Tab switching between minds
- Real-time delegation status panel

---

## 6. Dependencies on Other Agents

### What mind-tui NEEDS

| From | What | Why | Urgency |
|------|------|-----|---------|
| **mind-coordination** (codex-types) | `MindEvent`, `ExternalEvent`, `DriveEvent`, `EventPriority`, `EventSource` | TUI renders these event types | **Have it** — already in codex-types |
| **mind-model-router** (codex-drive) | Token streaming channel (`mpsc::Sender<DisplayEvent>`) | Need to receive LLM tokens for real-time display | **Phase 2** — need interface agreement |
| **mind-model-router** (ThinkLoop) | Interrupt mechanism | Ctrl-C needs to cancel current generation | **Phase 2** — need cancellation token/channel |
| **mind-skills** | Registered slash commands list | TUI needs to know valid `/` commands for tab-completion | **Phase 3** — can stub initially |
| **mind-tool-engine** | Tool call format (ToolCall, ToolResult types) | TUI renders tool blocks | **Have it** — already in codex-exec |
| **mind-hooks** | Pre/post events for TUI hooks (optional) | Allow hooks to modify TUI behavior | **Phase 4** — not needed for MVP |

### What mind-tui PROVIDES

| To | What | Interface |
|----|------|-----------|
| **mind-model-router** | User input stream | `mpsc::Receiver<Op>` — TUI sends user prompts to drive loop |
| **mind-skills** | Slash command dispatch | TUI parses `/command args` and sends `Op::SlashCommand` |
| **mind-coordination** | Interactive mode entry point | `pub async fn run(display_rx, op_tx, config) -> Result<()>` |
| **All agents** | Human visibility | The TUI is the ONLY human-facing surface. Everything else is headless. |

---

## 7. Cherry-Pick Assessment

### From Codex TUI (131K lines)

**Do NOT fork the Codex TUI.** It's React+Ink compiled to Rust via ratatui — enormous, tightly coupled to Codex's core Op/Event types, and includes massive features we don't need (diff viewer, voice UI, agent panels, theme engine with 20+ themes, accessibility layer, clipboard integration, fuzzy search file picker).

**Patterns to STUDY and adapt:**

| Pattern | Where in Codex | What to Learn | Lines to Study |
|---------|---------------|---------------|----------------|
| **Op/Event boundary** | `tui/` ↔ `core/` interface | Clean protocol: TUI never calls core directly | ~200 lines of type definitions |
| **Terminal alternate screen** | App setup/teardown | Enter raw mode, alternate screen, restore on panic | ~50 lines |
| **Streaming token render** | Chat view component | Append tokens character-by-character without re-rendering entire view | ~100 lines |
| **Tool call display** | Tool confirmation/result components | Bordered box with name, args, result, timing | ~150 lines |
| **Input line editing** | Composer component | Line editing, history, multi-line support | ~200 lines |

**Total study: ~700 lines of Codex patterns.** The other 130,300 lines are irrelevant for MVP.

### From Gemini CLI TUI (~15K lines)

**Patterns to STUDY:**

| Pattern | Where | What to Learn |
|---------|-------|---------------|
| **State contexts** | `ui/contexts/` | UIStateContext, StreamingContext, InputContext — clean state separation |
| **Non-interactive mode** | `cli/nonInteractiveCli.ts` | How to support non-interactive/piped input alongside TUI |
| **Theme system** | `ui/themes/`, `ui/colors.ts` | Semantic color naming (not hardcoded ANSI) |

### Net Assessment

Build from scratch using ratatui + crossterm. Study ~700 lines of Codex patterns for the Op/Event boundary, streaming render, and tool display. Do not fork either TUI.

**Estimated MVP size**: ~800-1,200 lines of Rust. This gives us:
- Interactive input with history
- Streaming LLM output
- Tool call display
- Status bar
- Event log
- Slash command parsing

---

## 8. Key Design Decisions

### D1: ratatui + crossterm (not React+Ink, not raw ANSI)

**Why**: ratatui is the standard Rust TUI framework. It gives us:
- Immediate-mode rendering (redraw every frame — simple mental model)
- Built-in widgets (Paragraph, Block, List, Table)
- crossterm backend works on Linux, macOS, Windows
- No Node.js dependency (React+Ink would add a JS runtime)
- Same language as the rest of aiciv-mind (Rust)

### D2: Op/Event protocol (not direct function calls)

**Why**: Clean boundary means:
- TUI can be developed and tested independently (mock event stream)
- TUI can be replaced (web UI, API) without touching core
- No circular dependencies between TUI and core crates
- Codex validated this pattern at 131K lines — it scales

### D3: Single-pane MVP (not multi-pane)

**Why**: Multi-pane is a Phase 4 luxury. The minimal TUI needs:
- Chat view (80% of screen)
- Status bar (1 line top)
- Input box (3 lines bottom)

This is how Claude Code, Codex, and Gemini CLI all start. Multi-pane is a growth feature.

### D4: Conversation-first (not dashboard-first)

**Why**: aiciv-mind is an agentic coding harness. The primary interaction is conversation: user prompts → LLM responds → tools execute. The TUI should prioritize this flow. Dashboard/monitoring is secondary (status bar + event log suffice).

### D5: No codex-types changes needed for Phase 1

**Why**: All event types we need already exist in codex-types (`MindEvent`, `ExternalEvent`, `DriveEvent`, `EventPriority`). The new `Op` and `DisplayEvent` types live in `aiciv-tui/src/protocol.rs` — they are TUI-local, not shared types. This means mind-tui can build Phase 1 without blocking on mind-coordination.

---

## 9. Risk Register

| Risk | Impact | Mitigation |
|------|--------|------------|
| **ThinkLoop has no streaming callback** | Can't display tokens in real-time | Phase 2 requires mind-model-router to add `mpsc::Sender<String>` for token chunks to ThinkLoop |
| **No cancellation mechanism** | Ctrl-C can't stop generation | Phase 2 requires mind-model-router to add `CancellationToken` to ThinkLoop |
| **ratatui version drift** | Breaking API changes | Pin to specific version in Cargo.toml |
| **Terminal size edge cases** | Crash on tiny terminals | Use `ratatui::layout::Constraint::Min(1)` and graceful degradation |
| **Raw mode cleanup on panic** | Terminal left in broken state | Install panic hook that restores terminal before unwinding |

---

## 10. Dependency Graph

```
                     codex-types  (MindEvent, EventPriority, etc.)
                        │
                        ▼
                    aiciv-tui  ◀── crossterm (terminal events)
                        │         ratatui   (terminal rendering)
                        │         tokio     (async runtime)
                        │         serde     (serialization)
                        │
                        │  Op channel
                        ▼
                    codex-drive  (ThinkLoop, EventBus, DriveLoop)
                        │
                        ▼
                    codex-llm   (LLM API calls, token streaming)
                    codex-exec  (tool execution)
```

**aiciv-tui depends on**: codex-types (leaf types only), ratatui, crossterm, tokio, serde
**aiciv-tui does NOT depend on**: codex-drive, codex-llm, codex-exec, codex-coordination

The Op/Event channels are runtime wiring (mpsc channels passed at startup), not compile-time dependencies. This keeps the crate boundary clean.
