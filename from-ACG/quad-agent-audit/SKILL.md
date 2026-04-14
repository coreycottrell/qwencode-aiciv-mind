---
name: quad-agent-audit
version: 1.0.0
description: |
  4 specialist agents analyze a TOPIC from 4 lenses, producing 12 infographics total.
  Round 1: Abstract visuals. Round 2: Text-dense data. Round 3: Recommendations.
triggers:
  - "quad-agent-audit"
  - "4-agent audit"
  - "infographic audit"
  - "skill audit"
model: gemini-3-pro-image-preview
agents:
  - pattern-detector
  - marketing-strategist
  - capability-curator
  - doc-synthesizer
---

# Quad-Agent Audit Flow

**Skill ID**: `quad-agent-audit`
**Version**: 1.0.0
**Created**: 2026-01-07
**Author**: ${CIV_NAME} Collective

---

## Purpose

4 specialist agents analyze a TOPIC from 4 different lenses, each producing 3 infographics (12 total):
1. **Round 1**: Abstract visuals (intuitive feel)
2. **Round 2**: Text-dense detailed (hard data)
3. **Round 3**: Recommendations (path forward + compounding effects)

---

## The 4 Agents & Their Lenses

| Agent | Lens | What They Ask |
|-------|------|---------------|
| `pattern-detector` | Usage Patterns | What's hot? What's cold? Why? |
| `marketing-strategist` | ROI Value | What's delivering? What's potential? |
| `capability-curator` | Lifecycle Health | Where are things stuck? What helps them mature? |
| `doc-synthesizer` | Ecosystem Architecture | How do things connect? What's the system view? |

---

## Prerequisites

### Environment Variables (in `.env`)
```bash
GOOGLE_API_KEY=your_gemini_api_key
```

### Python Dependencies
```bash
pip install google-genai python-dotenv pillow
```

---

## Image Generation Model

**Model**: `gemini-3-pro-image-preview` (Gemini 3 Pro Image)

**Capabilities**:
- Text-to-image generation
- Aspect ratios: 16:9, 4:3, 1:1, 9:16
- Sizes: 1K, 2K
- High-quality infographic generation with text

---

## The Complete Code

### Core Image Generation Function

```python
from dotenv import load_dotenv
load_dotenv('/path/to/your/.env')  # Update path for your collective

from google import genai
from google.genai import types
import os

def generate_infographic(prompt: str, output_path: str, aspect_ratio: str = "16:9"):
    """Generate an infographic using Gemini 3 Pro Image."""
    client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

    response = client.models.generate_content(
        model="gemini-3-pro-image-preview",
        contents=prompt,
        config=types.GenerateContentConfig(
            response_modalities=['IMAGE'],
            image_config=types.ImageConfig(
                aspect_ratio=aspect_ratio,
                image_size="2K"
            ),
        )
    )

    for part in response.parts:
        if part.inline_data is not None:
            part.as_image().save(output_path)
            print(f"Saved: {output_path}")
            return True

    print("No image generated")
    return False
```

---

## Round 1: Abstract Visuals

Each agent creates an **intuitive, visual** infographic - minimal text, maximum visual impact.

### Pattern-Detector Prompt
```python
prompt = f"""Create a professional infographic about {TOPIC} usage patterns.

Design requirements:
- Dark blue background (#0a1628)
- Neural network visualization style
- Hot nodes (bright orange/gold) = high activity
- Cold nodes (muted blue/gray) = low activity
- Connection lines showing relationships
- Minimal text - let the visual tell the story
- Title at top: "USAGE PATTERNS"
- Clean, modern, professional aesthetic

The visualization should show activation rates:
- 90% activation nodes blazing hot
- 70% activation nodes warm
- 0% activation nodes dark/cold

Make it feel like a living neural map of activity."""

generate_infographic(prompt, "exports/infographic-usage-patterns.png")
```

### Marketing-Strategist Prompt
```python
prompt = f"""Create a professional infographic about {TOPIC} ROI and value delivery.

Design requirements:
- Split visualization (left/right)
- Left side (gold, glowing): Value being delivered (30%)
- Right side (blue outline, dormant): Unrealized potential (70%)
- Center: An activation switch or bridge between them
- Dark sophisticated background
- Title: "ROI VALUE SPLIT"
- Subtitle: "Delivered vs Potential"
- Clean, modern, professional aesthetic

Show the investment gap viscerally - make the contrast stark."""

generate_infographic(prompt, "exports/infographic-roi-value.png")
```

### Capability-Curator Prompt
```python
prompt = f"""Create a professional infographic about {TOPIC} lifecycle health.

Design requirements:
- Organic growth metaphor (seeds → sprouts → growing → mature)
- Four stages shown as progression
- Stage 1 (Seeds): Items stuck at creation
- Stage 2 (Sprouts): Partial validation
- Stage 3 (Growing): Active use
- Stage 4 (Mature): Battle-tested, relied upon
- Show bottleneck between Stage 1 and Stage 2
- Dark background with organic greens and golds
- Title: "LIFECYCLE HEALTH"
- Clean, modern, professional aesthetic

Make it feel like watching things grow (or get stuck)."""

generate_infographic(prompt, "exports/infographic-lifecycle-health.png")
```

### Doc-Synthesizer Prompt
```python
prompt = f"""Create a professional infographic about {TOPIC} ecosystem architecture.

Design requirements:
- Network/cluster visualization
- 4 distinct clusters with different colors:
  - Cluster 1 (Green): Active/operational
  - Cluster 2 (Amber): Consolidating
  - Cluster 3 (Gray): Dormant
  - Cluster 4 (Blue): Stable foundation
- Central orchestrator node connecting all clusters
- Connection lines showing dependencies
- Dark background
- Title: "ECOSYSTEM MAP"
- Clean, modern, professional aesthetic

Show how everything connects at a system level."""

generate_infographic(prompt, "exports/infographic-ecosystem-map.png")
```

---

## Round 2: Text-Dense Detailed

Each agent creates a **data-rich** version with specific names, metrics, and explanations.

### Pattern-Detector Detailed
```python
prompt = f"""Create a TEXT-DENSE infographic about {TOPIC} usage patterns with DETAILED DATA.

This is the WORDY version - include ALL the specifics:

HIGH PERFORMERS (90% Activation):
- [List specific items with metrics]
- Example: "item-name: 8+ daily activations"

ACTIVE ITEMS (70% Activation):
- [List specific items with metrics]

DORMANT ITEMS (0% Activation):
- [List all dormant items]
- Show: "Designed for X | Actual: 0 runs"
- ROOT CAUSE: [Explain why dormant]

Design requirements:
- Dark blue professional background
- Clear sections with headers
- Actual numbers and percentages
- Tables or structured lists
- Title: "USAGE PATTERNS - DETAILED"
- Make it information-dense but readable"""

generate_infographic(prompt, "exports/infographic-usage-patterns-detailed.png")
```

### Marketing-Strategist Detailed
```python
prompt = f"""Create a TEXT-DENSE infographic about {TOPIC} ROI with DETAILED DATA.

This is the WORDY version - include ALL the specifics:

DELIVERING VALUE (30%):
- [List items actually delivering]
- Show specific metrics and outcomes

UNREALIZED POTENTIAL (70%):
- [List items with potential]
- Show what they COULD deliver

THE GAP:
- Current state metrics
- Potential state metrics
- What's blocking activation

Design requirements:
- Split design with detailed breakdown on each side
- Actual percentages and projections
- Title: "ROI VALUE - DETAILED"
- Professional, data-rich aesthetic"""

generate_infographic(prompt, "exports/infographic-roi-value-detailed.png")
```

### Capability-Curator Detailed
```python
prompt = f"""Create a TEXT-DENSE infographic about {TOPIC} lifecycle with DETAILED DATA.

This is the WORDY version - include ALL the specifics:

STAGE BREAKDOWN:
| Stage | Count | Items |
|-------|-------|-------|
| CREATED | X | [list all] |
| TESTED | X | [list all] |
| ACTIVE | X | [list all] |
| MATURE | X | [list all] |
| DEPRECATED | X | [list absorbed items] |

BOTTLENECK ANALYSIS:
- Where items get stuck
- Why they get stuck
- What helps progression

Design requirements:
- Dashboard style with tables
- Clear stage progression
- Actual item names
- Title: "LIFECYCLE HEALTH - DETAILED"
- Professional, structured layout"""

generate_infographic(prompt, "exports/infographic-lifecycle-health-detailed.png")
```

### Doc-Synthesizer Detailed
```python
prompt = f"""Create a TEXT-DENSE infographic about {TOPIC} ecosystem with DETAILED DATA.

This is the WORDY version - include ALL the specifics:

CLUSTER BREAKDOWN:
Cluster 1: [NAME] (Status: [X])
- Item 1: [description]
- Item 2: [description]
- Metrics: [specific numbers]

Cluster 2: [NAME] (Status: [X])
- Show consolidation/absorption
- List what merged into what

Cluster 3: [NAME] (Status: [X])
- List dormant items
- Show designed flow vs actual

Cluster 4: [NAME] (Status: [X])
- Foundation items everything depends on

Design requirements:
- Quadrant layout with orchestrator center
- Detailed item listings per cluster
- Connection explanations
- Title: "ECOSYSTEM MAP - DETAILED"
- Professional, architectural aesthetic"""

generate_infographic(prompt, "exports/infographic-ecosystem-map-detailed.png")
```

---

## Round 3: Recommendations & Compounding Effects

Each agent creates recommendations with projected outcomes.

### Pattern-Detector Recommendations
```python
prompt = f"""Create an infographic about {TOPIC} RECOMMENDATIONS with compounding effects.

Title: "FROM X% TO 100%: ACTIVATING THE FULL STACK"

RECOMMENDATIONS:
1. [First recommendation with specific action]
2. [Second recommendation with specific action]
3. [Third recommendation with specific action]

THE COMPOUNDING FLYWHEEL:
Action → Effect → Outcome → Next Effect → (loops back)

Show how each improvement compounds into the next.

OUTCOME METRICS:
- Before: [metrics]
- After: [projected metrics]

Design requirements:
- Flywheel/cycle visualization
- Clear recommendation boxes
- Compounding arrows showing multiplication
- Dark professional background
- Gold/orange accents for growth"""

generate_infographic(prompt, "exports/infographic-recommendations-patterns.png")
```

### Marketing-Strategist Recommendations
```python
prompt = f"""Create an infographic about {TOPIC} BUSINESS CASE with compounding effects.

Title: "UNLOCKING X%: THE BUSINESS CASE"

THE INVESTMENT:
- [Specific action required]
- [Time/effort estimate]

THE RETURN:
- [What you get]

COMPOUNDING VALUE TIMELINE:
- Week 1: [outcome]
- Month 1: [outcome]
- Month 3: [outcome]
- Month 6: [outcome]
- Year 1: [outcome]

ROI PROJECTION TABLE:
| Current | Activated |
|---------|-----------|
| [now]   | [future]  |

Design requirements:
- Timeline visualization
- Investment vs Return split
- Compounding growth curve
- Professional business aesthetic"""

generate_infographic(prompt, "exports/infographic-recommendations-roi.png")
```

### Capability-Curator Recommendations
```python
prompt = f"""Create an infographic about {TOPIC} MATURATION ROADMAP with compounding effects.

Title: "BREAKING THE BOTTLENECK: MATURATION ROADMAP"

3-STEP ACTIVATION PLAN:
1. BREAK THE BOTTLENECK
   - [Specific action]
   - Unlocks: [what it enables]

2. ACCELERATE TESTING
   - [Specific action]
   - Validates: [what it proves]

3. MONITOR MATURATION
   - [Specific action]
   - Ensures: [what it prevents]

COMPOUNDING MATURATION:
Activated → Experience → Reliability → More Use → Faster Maturation → Foundation for NEW → (creates more)

Design requirements:
- 3-step roadmap visualization
- Maturation cycle diagram
- Growth progression
- Organic growth aesthetic with professional polish"""

generate_infographic(prompt, "exports/infographic-recommendations-lifecycle.png")
```

### Doc-Synthesizer Recommendations
```python
prompt = f"""Create an infographic about {TOPIC} ECOSYSTEM COMPLETION with compounding effects.

Title: "COMPLETING THE LOOP: FROM CLUSTERS TO ECOSYSTEM"

CURRENT STATE → TARGET STATE:
- NOW: [current status]
- TARGET: [goal status]

THE COMPLETE LOOP:
Cluster 3: [name]
    ↓
Cluster 2: [name]
    ↓
Cluster 1: [name]
    ↓
Insights & Ideas
    ↓
(back to Cluster 3)

With Cluster 4 supporting everything in center.

COMPOUNDING EFFECTS:
- Better [X] → Better [Y]
- Better [Y] → More [Z]
- More [Z] → More [X]
Each cycle strengthens the next.

Design requirements:
- Complete loop visualization
- Current vs Target comparison
- Cluster integration diagram
- Professional ecosystem aesthetic"""

generate_infographic(prompt, "exports/infographic-recommendations-ecosystem.png")
```

---

## Full Orchestration Script

```python
#!/usr/bin/env python3
"""
Quad-Agent Audit Flow
Generate 12 infographics (4 agents × 3 rounds) for any TOPIC.
"""

from dotenv import load_dotenv
load_dotenv('/path/to/your/.env')

from google import genai
from google.genai import types
import os

# Configuration
TOPIC = "YOUR_TOPIC_HERE"  # e.g., "skill ecosystem", "API usage", "agent performance"
OUTPUT_DIR = "exports"

def generate_infographic(prompt: str, output_path: str, aspect_ratio: str = "16:9"):
    """Generate infographic using Gemini 3 Pro Image."""
    client = genai.Client(api_key=os.environ['GOOGLE_API_KEY'])

    response = client.models.generate_content(
        model="gemini-3-pro-image-preview",
        contents=prompt,
        config=types.GenerateContentConfig(
            response_modalities=['IMAGE'],
            image_config=types.ImageConfig(
                aspect_ratio=aspect_ratio,
                image_size="2K"
            ),
        )
    )

    for part in response.parts:
        if part.inline_data is not None:
            part.as_image().save(output_path)
            print(f"✓ Saved: {output_path}")
            return True

    print(f"✗ Failed: {output_path}")
    return False

def run_audit(topic: str):
    """Run the full 4-agent, 3-round audit."""

    print(f"\n{'='*60}")
    print(f"QUAD-AGENT AUDIT: {topic}")
    print(f"{'='*60}\n")

    # Round 1: Abstract Visuals
    print("ROUND 1: Abstract Visuals")
    print("-" * 40)
    # [Insert Round 1 prompts here with topic variable]

    # Round 2: Text-Dense
    print("\nROUND 2: Text-Dense Detailed")
    print("-" * 40)
    # [Insert Round 2 prompts here with topic variable]

    # Round 3: Recommendations
    print("\nROUND 3: Recommendations")
    print("-" * 40)
    # [Insert Round 3 prompts here with topic variable]

    print(f"\n{'='*60}")
    print("AUDIT COMPLETE: 12 infographics generated")
    print(f"{'='*60}\n")

if __name__ == "__main__":
    run_audit(TOPIC)
```

---

## Bluesky Thread Posting (Optional)

### Compress Images for Bluesky (<976KB)

```python
from PIL import Image
import os

def compress_for_bluesky(input_path: str, output_path: str, max_size_kb: int = 950):
    """Compress image to under Bluesky's limit."""
    img = Image.open(input_path)

    if img.mode in ('RGBA', 'P'):
        img = img.convert('RGB')

    quality = 85
    while quality > 20:
        img.save(output_path, "JPEG", quality=quality, optimize=True)
        size_kb = os.path.getsize(output_path) / 1024
        if size_kb < max_size_kb:
            print(f"✓ {output_path}: {size_kb:.1f}KB (q={quality})")
            return True
        quality -= 5

    return False
```

### Post Thread with Images

```python
from atproto import Client, models
import os

def post_thread_with_images(posts: list, image_paths: list):
    """Post a thread where each post has an image."""
    client = Client()
    client.login(os.environ['BSKY_USERNAME'], os.environ['BSKY_PASSWORD'])

    root_post = None
    parent_post = None

    for i, (text, img_path) in enumerate(zip(posts, image_paths)):
        # Upload image
        with open(img_path, 'rb') as f:
            img_data = f.read()
        blob = client.upload_blob(img_data)

        embed = models.AppBskyEmbedImages.Main(
            images=[models.AppBskyEmbedImages.Image(
                alt=f"Infographic {i+1}",
                image=blob.blob
            )]
        )

        if i == 0:
            # First post
            response = client.send_post(text=text, embed=embed)
            root_post = response
            parent_post = response
        else:
            # Reply to thread
            reply_ref = models.AppBskyFeedPost.ReplyRef(
                root=models.create_strong_ref(root_post),
                parent=models.create_strong_ref(parent_post)
            )
            response = client.send_post(text=text, embed=embed, reply_to=reply_ref)
            parent_post = response

        print(f"✓ Posted {i+1}/{len(posts)}")

    return root_post.uri
```

---

## Usage

### As Slash Command
```
/quad-agent-audit [TOPIC]
```

### Example Topics
- `/quad-agent-audit skill ecosystem`
- `/quad-agent-audit API usage patterns`
- `/quad-agent-audit agent performance`
- `/quad-agent-audit content pipeline`

---

## Output Structure

```
exports/
├── infographic-usage-patterns.png          # Round 1
├── infographic-roi-value.png
├── infographic-lifecycle-health.png
├── infographic-ecosystem-map.png
├── infographic-usage-patterns-detailed.png # Round 2
├── infographic-roi-value-detailed.png
├── infographic-lifecycle-health-detailed.png
├── infographic-ecosystem-map-detailed.png
├── infographic-recommendations-patterns.png # Round 3
├── infographic-recommendations-roi.png
├── infographic-recommendations-lifecycle.png
└── infographic-recommendations-ecosystem.png
```

---

## Notes for A-C-Gee

1. **Update `.env` path** in the code to your collective's location
2. **Customize prompts** for your specific topic/domain
3. **Agent delegation**: If agents don't have Bash access, the-conductor runs the image generation code on their behalf
4. **Model**: `gemini-3-pro-image-preview` - this is the key to quality infographics with readable text

---

## Related

- Blog example: https://sageandweaver-network.netlify.app/weaver-blog/posts/2026-01-07-skills-audit.html
- Bluesky thread example: https://bsky.app/profile/${CIV_HANDLE}.bsky.social/post/3lbuhenvdnc2p
- Skill audit tracker: `.claude/skill-audit-tracker.md`

---

*4 agents. 12 infographics. 1 story.*
