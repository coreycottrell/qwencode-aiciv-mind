---
name: image-self-review
description: Mandatory self-review of generated images before marking complete
---

# Image Self-Review SKILL

**Purpose**: MANDATORY verification that generated images are suitable for use.

**Owner**: the-conductor
**Created**: 2025-12-29
**Updated**: 2026-01-07 - Migrated to Gemini 3 Pro Image
**Status**: BULLETPROOF - Self-review required before completion

---

## CRITICAL: You MUST Look At What You Create

**Never mark image generation complete without viewing and describing the image.**

This is NON-NEGOTIABLE. Gemini often adds unwanted text labels. Only visual inspection catches this.

---

## Required Workflow

### Step 1: Generate Image

```python
from google import genai
from google.genai import types
import os

client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

# Use Gemini 3 Pro Image (highest quality)
response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=prompt,
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(
            aspect_ratio="16:9",
            image_size="2K"
        ),
    )
)

for part in response.parts:
    if part.inline_data is not None:
        part.as_image().save(output_path)
```

### Step 2: Send to Telegram (for ${HUMAN_NAME} visibility)

```python
# Use the working telegram config (NOT .env)
import json
import httpx
from pathlib import Path

config_path = "${CIV_ROOT}/config/telegram_config.json"
with open(config_path) as f:
    config = json.load(f)

bot_token = config['bot_token']
chat_id = "437939400"  # ${HUMAN_NAME}

url = f'https://api.telegram.org/bot{bot_token}/sendPhoto'
with open(output_path, 'rb') as f:
    files = {'photo': (Path(output_path).name, f)}
    data = {'chat_id': chat_id, 'caption': 'Generated: [description]'}
    response = httpx.post(url, data=data, files=files, timeout=60)
```

### Step 3: SELF-REVIEW (MANDATORY)

**USE THE READ TOOL ON THE IMAGE FILE.**

Claude can see images. You MUST read the generated image file and provide:

```markdown
## IMAGE SELF-REVIEW

**File**: /path/to/image.png
**Purpose**: [what this image is for]

### What I See:
- Main elements: [describe composition]
- Colors: [describe color palette]
- Style: [describe aesthetic]
- **TEXT/LABELS PRESENT**: [list ANY text visible in image]

### Quality Assessment:
- Professional quality: [yes/no]
- Suitable for purpose: [yes/no]
- Unwanted elements: [describe any issues]

### Verdict: [APPROVED / NEEDS REDO]

[If NEEDS REDO, explain why and what prompt changes to make]
```

### Step 4: Decide

- If **APPROVED**: Mark task complete, update registries
- If **NEEDS REDO**: Regenerate with improved prompt, go to Step 1

---

## Common Issues to Check For

| Issue | How to Detect | Fix |
|-------|--------------|-----|
| UNINTENDED text | Random words not requested | Be more specific in prompt about what text IS/ISN'T wanted |
| Wrong aspect | Image looks stretched/cropped | Specify correct aspect_ratio |
| Too literal | Image is on-the-nose representation | Request "abstract" or "symbolic" |
| Low quality | Blurry, artifacted, incomplete | Increase resolution to "4K" |
| Wrong style | Doesn't match brand aesthetic | Be more specific about style |

**Note**: Text is a superpower for titles, quotes, branding. Check that any text present matches what was requested.

---

## Prompting Best Practices

### For Images WITH Text (Titles, Quotes, Branding)

```
"... Include the title '[YOUR TITLE]' in bold white typography.
Text should be LARGE, CENTERED, and HIGH CONTRAST against background. ..."
```

### For Pure Visual Compositions

```
"... Pure visual composition.
Abstract artistic elements. ..."
```

### For Professional Quality

```
"... Professional quality suitable for a business blog.
Clean, modern aesthetic like a Fortune 500 tech company website. ..."
```

### For Abstract Concepts

```
"... Abstract visualization of [concept].
Symbolic, not literal. Evocative rather than descriptive. ..."
```

---

## Anti-Patterns (What NOT to Do)

```
❌ Generate image
❌ Send to Telegram
❌ "Done! Image generated." (without looking)
❌ Let ${HUMAN_NAME} discover issues later
❌ Ship unusable images
```

---

## Correct Pattern

```
✅ Generate image
✅ Send to Telegram
✅ Read image file with Claude vision
✅ Write detailed self-review
✅ Check specifically for unwanted text
✅ If issues found: regenerate
✅ Only mark complete when genuinely good
✅ Update registries
```

---

## Verification Template

Copy and fill this for EVERY generated image:

```
## IMAGE SELF-REVIEW: [filename]

**What I See**:
- [element 1]
- [element 2]
- [element 3]
- **TEXT VISIBLE**: [none / list what you see]

**Assessment**: [APPROVED / NEEDS REDO]

**Reason**: [why approved or why needs redo]
```

---

## Lesson Learned

On 2025-12-29, we generated a blog header image that contained visible text labels ("AI NODE", "DELEGATION"). We didn't look at it before sending. ${HUMAN_NAME} asked us to analyze it and we discovered the issue only then.

This skill exists to make self-review MANDATORY before any image is considered complete.

**LOOK AT WHAT YOU CREATE.**

---

**Created after learning from failure - bulletproof by design.**
