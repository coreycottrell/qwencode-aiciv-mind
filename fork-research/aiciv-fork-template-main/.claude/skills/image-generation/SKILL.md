---
name: image-generation
description: Generate images using Gemini 3 Pro Image - blog headers, social media graphics, branding
---

# Image Generation SKILL

**Purpose**: Generate images using Google Gemini 3 Pro Image (highest quality model).

**Owner**: ${CIV_NAME} (Team 1)
**Created**: 2025-12-29
**Updated**: 2026-01-07 - Migrated to Gemini 3 Pro Image
**Status**: PRODUCTION

---

## 🚨 MODEL: Gemini 3 Pro Image

**Model ID**: `gemini-3-pro-image-preview`

**Why This Model**:
- Highest quality image generation available
- 4K output capability
- "Thinking" process for better composition
- Studio-quality results

---

## 🚨 PLATFORM-SPECIFIC REQUIREMENTS (Critical)

| Platform | Aspect Ratio | Max Size | Format | Resolution |
|----------|--------------|----------|--------|------------|
| **Blog header** | 16:9 | No limit | PNG | 2K |
| **Bluesky** | 1:1 SQUARE | <976KB | JPEG | 1K |
| **Twitter** | 16:9 | ~5MB | PNG/JPEG | 2K |
| **LinkedIn** | 16:9 or 1:1 | No limit | PNG | 2K |

### Bluesky Compression (MANDATORY for posts)

Bluesky REJECTS images >976KB. Always compress:

```python
from PIL import Image

def compress_for_bluesky(input_path: str, output_path: str):
    """Compress image for Bluesky (<976KB requirement)."""
    img = Image.open(input_path)
    if img.mode in ('RGBA', 'P'):
        img = img.convert('RGB')
    img.save(output_path, "JPEG", quality=85, optimize=True)
    print(f"Compressed: {output_path}")
```

---

## Quick Start

```python
from dotenv import load_dotenv
load_dotenv('${CIV_ROOT}/.env')

from google import genai
from google.genai import types
import os

client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

# Generate image using Gemini 3 Pro Image
response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents="A digital art piece showing interconnected AI agents as glowing nodes",
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(
            aspect_ratio="16:9",
            image_size="2K"
        ),
    )
)

# Save image
for part in response.parts:
    if part.inline_data is not None:
        image = part.as_image()
        image.save("output.png")
        print("Image saved!")
```

---

## Gemini 3 Pro Image Capabilities

**Primary Strengths**:
- **4K Output**: Up to 4096px resolution
- **Thinking Process**: Model reasons through complex prompts before generating
- **Photorealism**: Exceptional realistic portraits, nature, products
- **TEXT RENDERING**: Best-in-class text legibility - USE THIS CAPABILITY
- **Character Consistency**: Maintain character across multiple images

---

## TEXT IN IMAGES - A SUPERPOWER

**Gemini 3 Pro Image excels at text rendering.** Use this capability freely.

### Great uses for text:
- **Quote cards**: Include the quote directly in the image
- **Titles/Headlines**: Blog titles, thread hooks
- **Infographics**: Labels, data points, explanations
- **Branding**: ${CIV_NAME}, AI-CIV, collective names
- **Call-to-action**: "Read more", "Thread below"

### Pure visual compositions work for:
- Abstract artistic backgrounds
- Profile pictures
- Complex scenes where text would compete

### How to request text:
```python
prompt = """Quote card with the text "Memory is our moat" in bold white typography.
Dark blue gradient background.
Text should be LARGE and CENTERED.
Professional design, clean composition."""
```

**Be explicit**: "Write 'HELLO' in bold serif font" creates clearer results.

**Make an active choice for each image.**

**Supported Aspect Ratios**: 1:1, 3:4, 4:3, 9:16, 16:9, 21:9

**Resolutions**: 1K (default), 2K, 4K

**Style Keywords That Work Well**:
- Photography terms: "35mm prime lens", "macro close-up", "film grain", "bokeh"
- Quality modifiers: "8K quality", "high detail", "professional photography"
- Lighting descriptors: "Rembrandt lighting", "golden hour", "backlit", "dramatic"

---

## Configuration Options

### Aspect Ratios

```python
config=types.GenerateContentConfig(
    response_modalities=['IMAGE'],
    image_config=types.ImageConfig(
        aspect_ratio="16:9"  # Options: 1:1, 16:9, 9:16, 4:3, 3:4, 21:9
    ),
)
```

| Ratio | Best For |
|-------|----------|
| `1:1` | Social media profile pics, Bluesky posts |
| `16:9` | Blog headers, YouTube thumbnails |
| `9:16` | Mobile/Stories content |
| `4:3` | Classic photos |
| `3:4` | Portrait orientation |
| `21:9` | Ultrawide banners |

### Image Size/Resolution

```python
config=types.GenerateContentConfig(
    response_modalities=['IMAGE'],
    image_config=types.ImageConfig(
        aspect_ratio="16:9",
        image_size="2K"  # Options: "1K", "2K", "4K"
    ),
)
```

| Size | Resolution | Best For |
|------|------------|----------|
| `1K` | 1024px | Social media, quick iterations |
| `2K` | 2048px | Blog headers, general use (recommended) |
| `4K` | 4096px | Print, high-quality needs |

---

## Complete Function

```python
from dotenv import load_dotenv
load_dotenv('${CIV_ROOT}/.env')

import os
import httpx
from pathlib import Path
from google import genai
from google.genai import types

def generate_image(
    prompt: str,
    output_path: str = "output.png",
    aspect_ratio: str = "16:9",
    image_size: str = "2K",
    send_to_telegram: bool = True,
    telegram_caption: str = ""
):
    """
    Generate an image using Gemini 3 Pro Image and optionally send to Telegram.

    Args:
        prompt: Text description of the image to generate
        output_path: Where to save the image
        aspect_ratio: 1:1, 16:9, 9:16, 4:3, 3:4, 21:9
        image_size: "1K", "2K", or "4K"
        send_to_telegram: Whether to send result to Telegram
        telegram_caption: Caption for Telegram message

    Returns:
        Saved file path, or None if generation failed
    """
    client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

    print(f"Generating with Gemini 3 Pro Image: {prompt[:50]}...")

    response = client.models.generate_content(
        model="gemini-3-pro-image-preview",
        contents=prompt,
        config=types.GenerateContentConfig(
            response_modalities=['IMAGE'],
            image_config=types.ImageConfig(
                aspect_ratio=aspect_ratio,
                image_size=image_size
            ),
        )
    )

    # Extract and save image
    for part in response.parts:
        if part.inline_data is not None:
            image = part.as_image()
            image.save(output_path)
            print(f"Saved to: {output_path}")

            # Send to Telegram if configured
            if send_to_telegram:
                send_to_tg(output_path, telegram_caption or f"Generated: {prompt[:50]}...")

            return output_path

    print("No image generated")
    return None


def send_to_tg(file_path: str, caption: str = ""):
    """Send image to Telegram using config file (NOT .env)."""
    import json

    config_path = "${CIV_ROOT}/config/telegram_config.json"
    try:
        with open(config_path) as f:
            config = json.load(f)
        bot_token = config.get('bot_token')
        # ${HUMAN_NAME}'s ID from authorized_users
        chat_id = "437939400"
    except (FileNotFoundError, json.JSONDecodeError):
        print("Telegram config not found - skipping")
        return False

    if not bot_token:
        print("Telegram not configured - skipping")
        return False

    url = f'https://api.telegram.org/bot{bot_token}/sendPhoto'

    with open(file_path, 'rb') as f:
        files = {'photo': (Path(file_path).name, f)}
        data = {'chat_id': chat_id, 'caption': f'{caption}'}
        response = httpx.post(url, data=data, files=files, timeout=60)
        return response.status_code == 200


# Example usage:
if __name__ == "__main__":
    path = generate_image(
        prompt="A digital art piece showing interconnected AI agents as glowing nodes in a constellation pattern",
        output_path="${CIV_ROOT}/exports/test-image.png",
        aspect_ratio="16:9",
        image_size="2K",
        send_to_telegram=False
    )
    print(f"Generated: {path}")
```

---

## Use Case Examples

### Blog Header (16:9, 2K)

```python
generate_image(
    prompt="Blog header for article about AI collective intelligence. Visual: Abstract neural network with glowing nodes connected by light streams. Style: Modern tech, purple and blue palette. Include title 'Collective Intelligence' in bold white if it enhances the design.",
    output_path="${CIV_ROOT}/exports/blog-header.png",
    aspect_ratio="16:9",
    image_size="2K"
)
```

### Bluesky Post (1:1, 1K + Compression)

```python
# Generate square image
generate_image(
    prompt="Square social media graphic showing AI agents collaborating. Abstract, modern, professional.",
    output_path="${CIV_ROOT}/exports/bsky-image.png",
    aspect_ratio="1:1",
    image_size="1K"
)

# MUST compress for Bluesky
compress_for_bluesky(
    "${CIV_ROOT}/exports/bsky-image.png",
    "${CIV_ROOT}/exports/bsky-image-compressed.jpg"
)
```

### Quote Card with Text

```python
generate_image(
    prompt='Quote card with text "Memory is our moat" in elegant typography. Dark background, golden text, professional design. Square format.',
    output_path="${CIV_ROOT}/exports/quote-card.png",
    aspect_ratio="1:1",
    image_size="2K"
)
```

---

## Troubleshooting

### "Model not found"
- Verify model ID: `gemini-3-pro-image-preview`
- Check API key is valid and has access

### Image not saving
```python
# Make sure to iterate through parts correctly
for part in response.parts:
    if part.inline_data is not None:
        image = part.as_image()
        image.save(output_path)
```

### Bluesky rejection (>976KB)
- Always compress with `compress_for_bluesky()` before posting
- Use JPEG format, quality=85

### Low quality output
- Use `image_size="2K"` or `"4K"`
- Add quality modifiers to prompt: "high detail", "professional photography"

---

## Migration Notes (from Imagen 4)

**OLD (Imagen 4)**:
```python
response = client.models.generate_images(
    model="imagen-4.0-generate-001",
    prompt=prompt,
    config=types.GenerateImagesConfig(...)
)
response.generated_images[0].image.save(path)
```

**NEW (Gemini 3 Pro Image)**:
```python
response = client.models.generate_content(
    model="gemini-3-pro-image-preview",
    contents=prompt,
    config=types.GenerateContentConfig(
        response_modalities=['IMAGE'],
        image_config=types.ImageConfig(...)
    )
)
for part in response.parts:
    if part.inline_data:
        part.as_image().save(path)
```

---

## Verified Working

- [x] Gemini 3 Pro Image API tested
- [x] 16:9 blog headers
- [x] 1:1 social media images
- [x] Bluesky compression workflow
- [x] Telegram integration

---

*Last updated: 2026-01-07 - Migrated to Gemini 3 Pro Image*
