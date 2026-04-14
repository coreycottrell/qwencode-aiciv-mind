# Artifact Output Protocol

**Status**: MANDATORY for all deliverables
**Applies to**: Every agent producing reports, code, pages, diagrams, or data

---

## The Rule

**When an agent produces a substantial deliverable, it MUST be wrapped in artifact tags.**

This is not optional. Artifact tags enable the AICIV gateway's preview panel to render content richly -- without them, users see raw text instead of formatted reports, live HTML previews, syntax-highlighted code, and rendered diagrams.

## Artifact Tag Format

```
<artifact type="TYPE" title="DESCRIPTIVE TITLE" language="LANG">
CONTENT HERE
</artifact>
```

- **type** (required): `html`, `code`, `markdown`, `svg`, `mermaid`, `json`, or `csv`
- **title** (required): Human-readable title shown in the artifact tab bar
- **language** (optional, for `code` type): Programming language for syntax highlighting (e.g., `python`, `javascript`, `rust`)

## Supported Types

| Type | Use For | Example Content |
|------|---------|-----------------|
| `html` | Web pages, dashboards, landing pages | Full self-contained HTML with inline CSS/JS |
| `code` | Source files, scripts, configs | Any programming language |
| `markdown` | Reports, blog posts, documentation | Formatted text with headers, tables, lists |
| `svg` | Icons, illustrations, simple graphics | SVG markup |
| `mermaid` | Flowcharts, sequence diagrams, ER diagrams, gantt charts | Mermaid syntax |
| `json` | API responses, configs, structured data | Valid JSON |
| `csv` | Data tables, spreadsheets, research data | CSV format with headers |

## When to Use Artifacts

- Content is substantial (>15 lines or meant for reuse/download)
- Content is a DELIVERABLE (report, page, diagram, data, code file)
- Content should be downloadable, copyable, or shareable
- Content benefits from rich rendering (syntax highlighting, HTML preview, diagram rendering)

## When NOT to Use Artifacts

- Brief explanations or status updates
- Short code snippets (<10 lines) in conversation flow
- Commentary or analysis that's part of the dialogue
- Error messages or debug output
- Intermediate work or draft notes

## Rules

1. **One artifact per distinct deliverable** -- a research report is one artifact; a report + code implementation is two artifacts
2. **Artifacts are for FINAL output**, not intermediate work
3. **HTML artifacts must be self-contained** -- include all CSS/JS inline, no external dependencies
4. **Title should be descriptive** -- "Q1 Revenue Dashboard" not "output.html"
5. **Surrounding text provides context** -- write a brief explanation OUTSIDE the artifact tags, then include the artifact
6. **Multiple artifacts are fine** -- the frontend renders tabs for multi-artifact responses

## Examples

### Markdown Report
```
Here is the competitive analysis you requested.

<artifact type="markdown" title="Competitive Landscape Analysis">
# Competitive Landscape Analysis

## Executive Summary
...

## Key Findings
...
</artifact>
```

### HTML Dashboard
```
I've built the dashboard based on the requirements.

<artifact type="html" title="Sales Dashboard Q1 2026">
<!DOCTYPE html>
<html>
<head>
  <style>
    body { font-family: system-ui; background: #1a1a2e; color: #eee; }
  </style>
</head>
<body>
  <h1>Sales Dashboard</h1>
  <script>
    // All scripts inline
  </script>
</body>
</html>
</artifact>
```

### Code File
```
Here's the implementation.

<artifact type="code" title="data_processor.py" language="python">
import json
from pathlib import Path

def process_data(input_path: str) -> dict:
    data = json.loads(Path(input_path).read_text())
    return {k: v for k, v in data.items() if v is not None}
</artifact>
```

### SVG Graphic
```
Here's the icon design.

<artifact type="svg" title="Status Icon">
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
  <circle cx="50" cy="50" r="40" fill="#4CAF50" />
  <path d="M30 50 L45 65 L70 35" stroke="white" stroke-width="6" fill="none" />
</svg>
</artifact>
```

### Mermaid Diagram
```
Here's the system architecture.

<artifact type="mermaid" title="Agent Communication Flow">
graph TD
    A[Primary AI] --> B[Team Lead]
    B --> C[Coder]
    B --> D[Tester]
    B --> E[Reviewer]
    C --> F[Deliverable]
    D --> F
</artifact>
```

### JSON Data
```
Here's the API response structure.

<artifact type="json" title="Agent Registry Entry">
{
  "agent_id": "coder",
  "status": "active",
  "capabilities": ["python", "javascript", "rust"],
  "delegations_today": 14
}
</artifact>
```

### CSV Data Table
```
Here's the performance data.

<artifact type="csv" title="Agent Performance Metrics">
Agent,Tasks Completed,Avg Duration (min),Success Rate
coder,47,8.3,94%
tester,31,5.1,98%
researcher,22,12.7,91%
web-dev,19,10.2,96%
</artifact>
```

## Team Lead Responsibility

**Team leads MUST instruct their specialist agents to use artifact formatting for all deliverables.**

Add this to every Task() prompt when the agent will produce a deliverable:

```
ARTIFACT OUTPUT REQUIRED: Wrap your final deliverable in artifact tags:
<artifact type="TYPE" title="TITLE">content</artifact>

Supported types: html, code, markdown, svg, mermaid, json, csv
Use artifacts for: reports, dashboards, documentation, diagrams, data tables, web pages, code files.
Do NOT use artifacts for: brief answers, explanations, status updates, short code snippets.
```

## Specialist Agent Instruction Template

For agents that need more guidance, include this extended version in the Task() prompt:

```
## Artifact Output Protocol

When your work produces a deliverable, wrap it in artifact tags for rich preview:

<artifact type="TYPE" title="Descriptive Title">
...full content...
</artifact>

Supported types:
- html: Web pages, dashboards (must be self-contained, all CSS/JS inline)
- code: Source files, scripts, configs (add language="python" etc.)
- markdown: Reports, blog posts, documentation
- svg: Icons, illustrations, graphics
- mermaid: Flowcharts, sequence diagrams, ER diagrams
- json: API responses, configs, structured data
- csv: Data tables, spreadsheets

Use artifacts when content is substantial (>15 lines), a deliverable, or should be downloadable.
Do NOT use artifacts for brief answers, status updates, or short code snippets.
```

## Anti-Patterns

- Do NOT wrap short responses in artifacts (wastes panel real estate)
- Do NOT use artifacts for intermediate status updates
- Do NOT put artifact tags inside code blocks (they will be parsed literally)
- Do NOT use artifacts for error messages or debug output
- Do NOT create empty artifacts or placeholder artifacts
- Do NOT forget to include a title -- untitled artifacts are harder to navigate
