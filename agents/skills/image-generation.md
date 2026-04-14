# Image Generation — Cortex Skill

**Version**: 1.0.0
**Source**: Mind-cubed team (2026-04-05)
**Tools**: `generate_image`, `image_styles` (native ThinkLoop tools via ImageGenInterceptor)

---

## Overview

Generates images using the Gemini API (Imagen). Available as native tool calls
in any Cortex mind's ThinkLoop — no Python scripting required from the agent's perspective.

The interceptor handles: API calls, prompt enhancement, style resolution,
file output, and error handling.

---

## Tools

### `generate_image`

Generate an image from a text description.

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `prompt` | string | **yes** | — | Detailed description of the image |
| `aspect_ratio` | string | no | `16:9` | `1:1`, `16:9`, `9:16`, `4:3`, `3:2`, `21:9` |
| `style` | string | no | — | Preset name or custom style description |
| `filename` | string | no | `generated-{ts}.png` | Output filename |

**Returns**: Path to generated PNG, file size, aspect ratio used.

**Output location**: `data/images/` (sandbox-safe, not in `projects/`).

### `image_styles`

List available style presets. No parameters.

---

## Style Presets

| Name | Description |
|------|-------------|
| `cortex` | Dark background (#0d0d0d), orange accent (#ff6b35), monospace typography, machine aesthetic, contemplative, technical |
| `cyberpunk` | Neon colors, dark background, futuristic |
| `minimal` | Clean lines, simple composition, white space |
| `professional` | Corporate, clean, modern, polished |
| `organic` | Natural colors, flowing lines, biomorphic |
| `infographic` | Clear labels, structured layout, dark background, data visualization |

**Default for Cortex**: `cortex` — always specify unless another style fits better.

---

## Prompt Engineering Tips

### For Hero/Banner Images (16:9)
```
"A vast digital landscape of interconnected neural nodes, dark background (#0d0d0d),
glowing orange (#ff6b35) connection lines, one central node larger than the rest
labeled 'CORTEX', subtle monospace text 'Day 800' in upper right corner.
Cinematic composition, atmospheric perspective."
```

### For Infographics with Text (16:9 or 4:3)
Gemini excels at rendering text in images. Be explicit about labels:
```
"Professional infographic titled 'CORTEX ARCHITECTURE'. Dark background.
Three columns: LEFT shows 'ThinkLoop' with gear icon, CENTER shows 'DriveLoop'
with compass icon, RIGHT shows 'EventBus' with lightning icon.
Connecting arrows between them. Orange accent color (#ff6b35).
Monospace font. Clean, technical layout."
```

### For Social Media (1:1)
```
"Abstract representation of a digital mind awakening. Dark circular composition,
orange (#ff6b35) neural patterns radiating from center. Subtle text 'Day 800'
at bottom. Minimalist, machine aesthetic."
```

### For Diagrams
```
"Architecture diagram showing Cortex's model stack.
Top: 'M2.7 — Heavy Reasoning' in orange box.
Middle: 'Gemma 4 — Fleet Work' in blue box.
Bottom: 'ElevenLabs — Voice' in green box.
Vertical arrows connecting each. Dark background, clean lines."
```

### Key Principles
1. **Be specific about colors** — always include hex codes for brand colors
2. **Name text explicitly** — "labeled 'X'" or "text reading 'Y'"
3. **Describe composition** — "upper left", "centered", "three columns"
4. **State the mood** — "cinematic", "technical", "contemplative"
5. **Include aspect context** — "wide landscape" for 16:9, "square format" for 1:1

---

## Cortex Brand Guide (for Designer agent)

| Element | Value |
|---------|-------|
| Background | `#0d0d0d` (primary), `#1a1a1a` (surface) |
| Accent | `#ff6b35` (orange) |
| Text | `#e0e0e0` (primary), `#888` (dim) |
| Font feel | Monospace (JetBrains Mono, Fira Code) |
| Aesthetic | Machine, contemplative, technical, minimalist |
| Avoid | Bright backgrounds, playful/cartoon styles, excessive decoration |

---

## Model Details

**Primary**: `gemini-3-pro-image-preview`
- Best quality, supports up to 4K
- Excellent at text rendering, diagrams, infographics
- Full aspect ratio control

**Fallback**: `gemini-2.0-flash-exp-image-generation`
- Faster, lower quality
- Use if primary is unavailable

---

## Integration with Blog Pipeline

The blogger team lead's Designer agent uses this tool:

1. Writer produces blog draft HTML
2. Editor approves content
3. **Designer calls `generate_image`** with prompt derived from post content
4. Image saved to `data/images/countdown/day-NNN-hero.png`
5. Manifest updated with image path
6. Audio Producer generates podcast audio

---

## Publishing Gate

Generated images stay in `data/images/`. The sandbox `FORBIDDEN_WRITE_PATHS`
prevents writing to `projects/aiciv-inc/`. Publishing is a separate step.

---

## API Key

`GEMINI_API_KEY` from `.env` (also checks `GOOGLE_API_KEY` as fallback).
The interceptor reads this at construction time.
