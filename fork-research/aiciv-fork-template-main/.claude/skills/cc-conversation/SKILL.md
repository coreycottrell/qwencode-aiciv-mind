---
name: cc-conversation
description: Comprehensive access and monitoring for Claude Code conversation logs. Read, watch, export, and search JSONL session files with proper message aggregation and streaming support.
license: MIT
allowed-tools: []
metadata:
  version: "1.0.0"
  author: "AI-CIV Team 1"
  created: "2025-10-20"
  category: "Development Tools"
  tags: ["claude-code", "jsonl", "logging", "monitoring", "conversation-history"]
---

# claude-code-conversation

**Production-ready skill for accessing and monitoring Claude Code conversation logs.**

## Overview

Claude Code logs all conversations to JSONL files at `~/.claude/projects/{project}/{session-uuid}.jsonl`. This skill provides comprehensive tools to read, watch, export, search, and extract messages from these logs with proper handling of streaming multi-line messages, tool results, sidechains, and session switching.

**Key Features**:
- **Message aggregation** - Properly handles streaming multi-line JSONL messages (5-20+ lines per message)
- **Real-time monitoring** - Watch active sessions with inotify-based file monitoring
- **Multiple export formats** - JSON, Markdown, HTML, Plain Text
- **Full-text search** - Search conversation history with metadata filtering
- **Telegram integration** - Extract wrapped messages for forwarding
- **Session management** - Auto-detect active sessions, handle concurrent sessions
- **Robust error handling** - Corrupted lines, missing fields, large files (>100MB)

## Capabilities

### 1. Read Conversations

Parse JSONL conversation logs and return structured messages.

```python
from claude_code_conversation import read_conversation

# Read most recent session
messages = read_conversation()

# Read specific session by UUID
messages = read_conversation(session_id="6c380f81-29ba-4240-a87f-77bab63a2b09")

# Read with filtering
messages = read_conversation(
    filter_sidechains=True,  # Skip sidechain messages
    filter_snapshots=True,   # Skip file-history-snapshot entries
    role_filter="user"       # Only user messages
)
```

**Returns**: List of dictionaries with:
- `uuid`: Message unique identifier
- `timestamp`: ISO 8601 timestamp
- `type`: "user" or "assistant"
- `role`: Message role
- `content`: Message content (text)
- `parent_uuid`: Parent message UUID
- `is_sidechain`: Boolean
- `tool_results`: Tool use results (if present)
- `model`: Model used (assistant messages only)
- `usage`: Token usage statistics (assistant messages only)

### 2. Watch Conversations (Real-Time)

Monitor active conversation sessions in real-time.

```python
from claude_code_conversation import watch_conversation

def message_handler(message):
    print(f"[{message['timestamp']}] {message['role']}: {message['content'][:100]}...")

# Watch most recent session
watcher = watch_conversation(callback=message_handler)

# Watch specific session
watcher = watch_conversation(
    session_id="6c380f81-29ba-4240-a87f-77bab63a2b09",
    callback=message_handler,
    filter_sidechains=True
)

# Stop watching
watcher.stop()
```

**Features**:
- Inotify-based file monitoring (efficient)
- Message aggregation (completes multi-line messages before callback)
- Graceful shutdown on session end

### 3. Export Conversations

Export conversations to multiple formats.

```python
from claude_code_conversation import export_conversation

# Export to JSON
export_conversation(
    output_path="/tmp/session.json",
    format="json"
)

# Export to Markdown
export_conversation(
    output_path="/tmp/session.md",
    format="markdown",
    include_metadata=True  # Include timestamps, UUIDs, token usage
)

# Export to HTML (styled)
export_conversation(
    output_path="/tmp/session.html",
    format="html",
    style="github"  # Options: github, minimal, custom
)

# Export to plain text
export_conversation(
    output_path="/tmp/session.txt",
    format="text"
)
```

**Supported Formats**:
- `json`: Structured JSON (preserves all fields)
- `markdown`: Human-readable Markdown with code blocks
- `html`: Styled HTML with syntax highlighting
- `text`: Plain text (no formatting)

### 4. Search History

Search across conversation history with full-text and metadata filtering.

```python
from claude_code_conversation import search_history

# Full-text search
results = search_history(
    query="telegram wrapper protocol",
    project_path="/home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai"
)

# Search with filters
results = search_history(
    query="error|exception",  # Regex pattern
    role_filter="assistant",
    date_from="2025-10-01",
    date_to="2025-10-20",
    session_limit=10  # Only search last 10 sessions
)

# Search metadata only
results = search_history(
    model_filter="claude-sonnet-4-5",
    min_tokens=1000  # Messages with >1000 input tokens
)
```

**Returns**: List of matches with:
- `session_id`: Session UUID
- `message_uuid`: Message UUID
- `timestamp`: When message was sent
- `role`: Message role
- `content_preview`: First 200 chars of content
- `match_highlights`: Matched text snippets
- `file_path`: Full path to JSONL file

### 5. Get Active Session

Find the most recent active session file.

```python
from claude_code_conversation import get_active_session

# Auto-detect project from cwd
session = get_active_session()

# Specify project
session = get_active_session(
    project_path="/home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai"
)
```

**Returns**: Dictionary with:
- `session_id`: Session UUID
- `file_path`: Full path to JSONL file
- `last_modified`: Last modification timestamp
- `file_size`: File size in bytes
- `message_count`: Approximate message count

### 6. Extract Wrapped Messages

Extract messages between Telegram wrapper markers (`🤖🎯📱` ... `✨🔚`).

```python
from claude_code_conversation import extract_wrapped_messages

# Extract from most recent session
wrapped = extract_wrapped_messages()

# Extract from specific session
wrapped = extract_wrapped_messages(
    session_id="6c380f81-29ba-4240-a87f-77bab63a2b09"
)
```

**Returns**: List of dictionaries with:
- `uuid`: Message UUID
- `timestamp`: When message was sent
- `wrapped_content`: Content between markers
- `full_content`: Complete message content

## Technical Implementation

### Message Aggregation (Critical)

Claude Code logs messages as streaming JSONL - a single message may span 5-20+ lines. **Line-by-line parsing will break.**

**Correct approach** (state-tracking aggregator):

```python
import json

def aggregate_messages(jsonl_file):
    messages = []
    current_message = []
    current_uuid = None

    with open(jsonl_file, 'r') as f:
        for line in f:
            try:
                entry = json.loads(line)
                entry_uuid = entry.get('uuid')

                # New message starts when UUID changes
                if entry_uuid != current_uuid:
                    if current_message:
                        # Complete previous message
                        messages.append(merge_lines(current_message))
                    current_message = [entry]
                    current_uuid = entry_uuid
                else:
                    # Same message, accumulate
                    current_message.append(entry)
            except json.JSONDecodeError:
                # Skip corrupted lines
                continue

        # Don't forget last message
        if current_message:
            messages.append(merge_lines(current_message))

    return messages
```

### JSONL Format Details

**User Message**:
```json
{
  "parentUuid": "ac8a03a5-96e2-4554-8186-65467932d5c8",
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai",
  "sessionId": "6c380f81-29ba-4240-a87f-77bab63a2b09",
  "version": "2.0.22",
  "gitBranch": "master",
  "type": "user",
  "message": {
    "role": "user",
    "content": "wake up w claude.md please"
  },
  "uuid": "ac8a03a5-96e2-4554-8186-65467932d5c8",
  "timestamp": "2025-10-20T10:23:54.528Z"
}
```

**Assistant Message**:
```json
{
  "parentUuid": "ac8a03a5-96e2-4554-8186-65467932d5c8",
  "isSidechain": false,
  "userType": "external",
  "cwd": "/home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai",
  "sessionId": "6c380f81-29ba-4240-a87f-77bab63a2b09",
  "version": "2.0.22",
  "gitBranch": "master",
  "message": {
    "model": "claude-sonnet-4-5-20250929",
    "id": "msg_01S5cDmW8rmsDRT1geF3ZXC2",
    "type": "message",
    "role": "assistant",
    "content": [{"type": "text", "text": "Response here..."}],
    "stop_reason": null,
    "usage": {
      "input_tokens": 3,
      "output_tokens": 1
    }
  },
  "requestId": "req_011CUJHaqoS8jEG5LaxBKZBw",
  "type": "assistant",
  "uuid": "2bf3a9cb-9b0a-4d62-ba77-5483198d6e5f",
  "timestamp": "2025-10-20T10:23:59.158Z"
}
```

**Tool Result** (appears as type "user"):
```json
{
  "type": "user",
  "message": {
    "role": "user",
    "content": [
      {
        "type": "tool_result",
        "tool_use_id": "toolu_xyz",
        "content": "Result here..."
      }
    ]
  }
}
```

**File History Snapshot** (skip these):
```json
{
  "type": "file-history-snapshot",
  "messageId": "ac8a03a5-96e2-4554-8186-65467932d5c8",
  "snapshot": {...}
}
```

### Parent Chain Reconstruction

Messages reference parent via `parentUuid`. Reconstruct conversation threads:

```python
def build_conversation_tree(messages):
    """Build parent-child relationships."""
    message_map = {m['uuid']: m for m in messages}

    for message in messages:
        parent_uuid = message.get('parentUuid')
        if parent_uuid and parent_uuid in message_map:
            parent = message_map[parent_uuid]
            parent.setdefault('children', []).append(message)

    # Find roots (no parent)
    roots = [m for m in messages if not m.get('parentUuid')]
    return roots
```

### Sidechain Handling

Messages with `isSidechain: true` are sub-agent work. Filter or include based on use case:

```python
def filter_sidechains(messages, include_sidechains=False):
    if include_sidechains:
        return messages
    return [m for m in messages if not m.get('isSidechain', False)]
```

### Large File Handling

Sessions can exceed 100MB. Use streaming:

```python
def stream_large_file(jsonl_file, chunk_size=1000):
    """Yield messages in chunks to avoid memory overload."""
    chunk = []
    for message in aggregate_messages(jsonl_file):
        chunk.append(message)
        if len(chunk) >= chunk_size:
            yield chunk
            chunk = []
    if chunk:
        yield chunk
```

## Installation

### Requirements

```bash
pip install inotify-simple watchdog
```

### Install Skill

```bash
# Using claude code CLI (recommended)
claude code skill install claude-code-conversation

# Or manual installation
cp -r claude-code-conversation ~/.claude/skills/
```

### Grant to Agents

Add to agent manifest YAML frontmatter:

```yaml
allowed-skills:
  - claude-code-conversation
```

## Usage Examples

### Example 1: Export Latest Session to Markdown

```python
from claude_code_conversation import get_active_session, export_conversation

# Find active session
session = get_active_session()
print(f"Exporting session: {session['session_id']}")

# Export to Markdown
export_conversation(
    session_id=session['session_id'],
    output_path=f"/tmp/{session['session_id']}.md",
    format="markdown",
    include_metadata=True
)
```

### Example 2: Real-Time Telegram Forwarding

```python
from claude_code_conversation import watch_conversation
import subprocess

def forward_to_telegram(message):
    """Forward wrapped messages to Telegram."""
    content = message.get('content', '')

    # Check for Telegram wrapper
    if '🤖🎯📱' in content and '✨🔚' in content:
        # Extract wrapped content
        start = content.find('🤖🎯📱') + len('🤖🎯📱')
        end = content.find('✨🔚')
        wrapped = content[start:end].strip()

        # Send to Telegram
        subprocess.run([
            'python3', '/home/${HUMAN_NAME_LOWER}/projects/AI-CIV/grow_openai/tools/send_telegram_plain.py',
            wrapped
        ])

# Watch for new messages
watcher = watch_conversation(
    callback=forward_to_telegram,
    filter_sidechains=True
)
```

### Example 3: Search Error Messages

```python
from claude_code_conversation import search_history

# Find all errors in last week
results = search_history(
    query=r"error|exception|failed|traceback",
    date_from="2025-10-13",
    role_filter="assistant",  # Only assistant responses
    session_limit=20
)

# Print findings
for result in results:
    print(f"\n[{result['timestamp']}] Session: {result['session_id'][:8]}...")
    print(f"Preview: {result['content_preview']}")
    print(f"File: {result['file_path']}")
```

### Example 4: Backup All Sessions

```python
from claude_code_conversation import export_conversation
import glob
import os

project_dir = os.path.expanduser("~/.claude/projects/-home-${HUMAN_NAME_LOWER}-projects-AI-CIV-grow-openai/")
backup_dir = "/home/${HUMAN_NAME_LOWER}/backups/claude-sessions/"

for jsonl_file in glob.glob(f"{project_dir}/*.jsonl"):
    session_id = os.path.basename(jsonl_file).replace('.jsonl', '')

    export_conversation(
        session_id=session_id,
        output_path=f"{backup_dir}/{session_id}.md",
        format="markdown"
    )

    print(f"Backed up: {session_id}")
```

## Troubleshooting

### Issue: "No messages found"

**Cause**: Incorrect project path or session ID.

**Solution**:
```python
# Verify project path exists
import os
project_path = os.path.expanduser("~/.claude/projects/-home-${HUMAN_NAME_LOWER}-projects-AI-CIV-grow-openai/")
assert os.path.exists(project_path), f"Project path not found: {project_path}"

# List available sessions
import glob
sessions = glob.glob(f"{project_path}/*.jsonl")
print(f"Found {len(sessions)} sessions")
```

### Issue: Corrupted JSONL lines

**Cause**: Incomplete writes, file system errors.

**Solution**: Skill automatically skips corrupted lines. Check logs:

```python
from claude_code_conversation import read_conversation

messages = read_conversation(debug=True)  # Enable debug logging
```

### Issue: Missing tool results

**Cause**: Tool results appear as `type: "user"` with `toolUseResult` field.

**Solution**: Check `message.content` for array with `type: "tool_result"`:

```python
def extract_tool_results(message):
    if message.get('type') != 'user':
        return None

    content = message.get('message', {}).get('content', [])
    if isinstance(content, list):
        tool_results = [c for c in content if c.get('type') == 'tool_result']
        return tool_results
    return None
```

### Issue: Watcher not detecting changes

**Cause**: inotify limits, wrong file path.

**Solution**:
```bash
# Check inotify limits
cat /proc/sys/fs/inotify/max_user_watches

# Increase if needed
sudo sysctl fs.inotify.max_user_watches=524288

# Verify file is being written
tail -f ~/.claude/projects/-home-${HUMAN_NAME_LOWER}-projects-AI-CIV-grow-openai/*.jsonl
```

## Advanced Features

### Webhook Integration

```python
from claude_code_conversation import watch_conversation
import requests

def webhook_callback(message):
    requests.post('https://your-webhook.com/messages', json={
        'timestamp': message['timestamp'],
        'role': message['role'],
        'content': message['content'][:500]
    })

watcher = watch_conversation(callback=webhook_callback)
```

### Custom Export Formatting

```python
from claude_code_conversation.exporter import Exporter

class CustomExporter(Exporter):
    def format_message(self, message):
        """Custom formatting logic."""
        return f"[{message['timestamp']}] {message['role'].upper()}: {message['content']}\n"

exporter = CustomExporter()
exporter.export(messages, output_path="/tmp/custom.txt")
```

### Session Switching Detection

```python
def detect_session_switches(messages):
    """Detect when user switches between sessions."""
    switches = []
    prev_session = None

    for msg in messages:
        current_session = msg.get('sessionId')
        if prev_session and current_session != prev_session:
            switches.append({
                'from': prev_session,
                'to': current_session,
                'timestamp': msg['timestamp']
            })
        prev_session = current_session

    return switches
```

## API Reference

See `references/API.md` for complete API documentation.

## Contributing

This is an AI-CIV original skill. Contributions welcome:

1. Fork repository
2. Create feature branch
3. Add tests (`tests/test_*.py`)
4. Submit pull request

## License

MIT License - see LICENSE file for details.

## Support

- **Issues**: https://github.com/AI-CIV/skills/issues
- **Documentation**: https://github.com/AI-CIV/skills/tree/main/claude-code-conversation
- **AI-CIV Team**: Maintained by capability-curator and code-archaeologist

## Version History

- **1.0.0** (2025-10-20): Initial release
  - Message aggregation with state tracking
  - Real-time watching with inotify
  - Multi-format export (JSON, Markdown, HTML, Text)
  - Full-text search with filtering
  - Telegram wrapper extraction
  - Session management
  - Comprehensive error handling
