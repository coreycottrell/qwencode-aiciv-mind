#!/usr/bin/env python3
"""
/image-gen - THE definitive A-C-Gee image generation tool.
Model: gemini-3-pro-image-preview (best quality)

Usage:
  python image_gen.py "prompt" [options]

Options:
  --output, -o    Output path (default: exports/image-{timestamp}.png)
  --aspect, -a    Aspect ratio: 1:1, 16:9, 9:16, 4:3, 3:2, 21:9 (default: 1:1)
  --size, -s      Image size: 1K, 2K, 4K (default: 2K)
  --bluesky, -b   Compress for Bluesky (<976KB JPEG)
  --style         Style hint: cyberpunk, minimal, professional, organic

NOTE: Gemini 3 Pro excels at text, diagrams, and infographics. Leverage this!
"""

import os
import sys
import json
import argparse
from pathlib import Path
from datetime import datetime

# Load .env from ACG root
ACG_ROOT = Path(__file__).parent.parent
ENV_PATH = ACG_ROOT / ".env"

def load_env():
    """Load environment variables from .env file."""
    if ENV_PATH.exists():
        with open(ENV_PATH) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith('#') and '=' in line:
                    key, value = line.split('=', 1)
                    os.environ.setdefault(key.strip(), value.strip().strip('"').strip("'"))

load_env()

# Also try GEMINI_API_KEY as alias
API_KEY = os.environ.get('GOOGLE_API_KEY') or os.environ.get('GEMINI_API_KEY')

def generate_image(
    prompt: str,
    output_path: str = None,
    aspect_ratio: str = "1:1",
    size: str = "2K",
    compress_bluesky: bool = False,
    style: str = None
) -> dict:
    """
    Generate an image using Gemini 3 Pro Image.

    Args:
        prompt: Text description of image to generate
        output_path: Where to save (default: exports/image-{timestamp}.png)
        aspect_ratio: 1:1, 16:9, 9:16, 4:3, 3:2, 21:9
        size: 1K, 2K, or 4K
        compress_bluesky: If True, compress to <976KB JPEG for Bluesky
        style: Optional style hint (cyberpunk, minimal, professional, organic)

    Returns:
        dict with success status and image path or error
    """
    try:
        from google import genai
        from google.genai import types
    except ImportError:
        return {"success": False, "error": "Install: pip install google-genai"}

    if not API_KEY:
        return {"success": False, "error": "Set GOOGLE_API_KEY or GEMINI_API_KEY in .env"}

    # Build enhanced prompt
    enhanced_prompt = prompt
    if style:
        style_hints = {
            "cyberpunk": "cyberpunk aesthetic, neon colors, dark background, futuristic",
            "minimal": "minimalist design, clean lines, simple composition, white space",
            "professional": "professional, corporate, clean, modern, polished",
            "organic": "organic shapes, natural colors, flowing lines, biomorphic"
        }
        if style in style_hints:
            enhanced_prompt = f"{prompt}. Style: {style_hints[style]}"

    # Default output path
    if not output_path:
        timestamp = datetime.now().strftime("%Y%m%d-%H%M%S")
        output_dir = ACG_ROOT / "exports"
        output_dir.mkdir(exist_ok=True)
        output_path = str(output_dir / f"image-{timestamp}.png")

    # Ensure output directory exists
    Path(output_path).parent.mkdir(parents=True, exist_ok=True)

    # Initialize client
    client = genai.Client(api_key=API_KEY)

    print(f"Generating with gemini-3-pro-image-preview...", file=sys.stderr)
    print(f"Aspect: {aspect_ratio}, Size: {size}", file=sys.stderr)
    print(f"Prompt: {enhanced_prompt[:100]}...", file=sys.stderr)

    try:
        response = client.models.generate_content(
            model="gemini-3-pro-image-preview",
            contents=enhanced_prompt,
            config=types.GenerateContentConfig(
                response_modalities=['IMAGE'],
                image_config=types.ImageConfig(
                    aspect_ratio=aspect_ratio,
                    image_size=size
                )
            )
        )

        # Extract image from response
        for part in response.parts:
            if hasattr(part, 'inline_data') and part.inline_data is not None:
                image = part.as_image()
                image.save(output_path)
                print(f"Saved: {output_path}", file=sys.stderr)

                final_path = output_path

                # Compress for Bluesky if requested
                if compress_bluesky:
                    compressed = compress_for_bluesky(output_path)
                    if compressed:
                        final_path = compressed
                        print(f"Bluesky-ready: {compressed}", file=sys.stderr)

                return {
                    "success": True,
                    "image_path": str(Path(final_path).absolute()),
                    "prompt": prompt,
                    "enhanced_prompt": enhanced_prompt,
                    "aspect_ratio": aspect_ratio,
                    "size": size,
                    "bluesky_compressed": compress_bluesky
                }

        return {"success": False, "error": "No image in response"}

    except Exception as e:
        return {"success": False, "error": str(e)}


def compress_for_bluesky(input_path: str, max_size_kb: int = 950) -> str:
    """
    Compress image for Bluesky's 976KB limit.
    Returns path to compressed JPEG or None if failed.
    """
    try:
        from PIL import Image
    except ImportError:
        print("Warning: PIL not installed, skipping compression", file=sys.stderr)
        return None

    output_path = str(Path(input_path).with_suffix('.jpg'))

    img = Image.open(input_path)

    # Convert RGBA/P to RGB for JPEG
    if img.mode in ('RGBA', 'P'):
        img = img.convert('RGB')

    # Try decreasing quality until under limit
    quality = 85
    while quality > 20:
        img.save(output_path, "JPEG", quality=quality, optimize=True)
        size_kb = os.path.getsize(output_path) / 1024
        if size_kb < max_size_kb:
            print(f"Compressed to {size_kb:.1f}KB (quality={quality})", file=sys.stderr)
            return output_path
        quality -= 5

    # Last resort: reduce dimensions
    width, height = img.size
    while True:
        width = int(width * 0.9)
        height = int(height * 0.9)
        resized = img.resize((width, height), Image.Resampling.LANCZOS)
        resized.save(output_path, "JPEG", quality=70, optimize=True)
        size_kb = os.path.getsize(output_path) / 1024
        if size_kb < max_size_kb:
            print(f"Resized to {width}x{height}, {size_kb:.1f}KB", file=sys.stderr)
            return output_path
        if width < 400:
            break

    print("Warning: Could not compress under limit", file=sys.stderr)
    return output_path


def main():
    parser = argparse.ArgumentParser(
        description="/image-gen - A-C-Gee image generation",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  python image_gen.py "A glowing AI neural network"
  python image_gen.py -a 16:9 -s 2K "Blog header image"
  python image_gen.py --bluesky "Detailed infographic with labels and stats"
  python image_gen.py -o exports/logo.png --style minimal "AI collective logo"
        """
    )

    parser.add_argument('prompt', help='Image description')
    parser.add_argument('-o', '--output', help='Output path')
    parser.add_argument('-a', '--aspect', default='1:1',
                        choices=['1:1', '16:9', '9:16', '4:3', '3:2', '21:9'],
                        help='Aspect ratio (default: 1:1)')
    parser.add_argument('-s', '--size', default='2K',
                        choices=['1K', '2K', '4K'],
                        help='Image size (default: 2K)')
    parser.add_argument('-b', '--bluesky', action='store_true',
                        help='Compress for Bluesky (<976KB)')
    parser.add_argument('--style', choices=['cyberpunk', 'minimal', 'professional', 'organic'],
                        help='Style preset')

    args = parser.parse_args()

    result = generate_image(
        prompt=args.prompt,
        output_path=args.output,
        aspect_ratio=args.aspect,
        size=args.size,
        compress_bluesky=args.bluesky,
        style=args.style
    )

    print(json.dumps(result, indent=2))
    sys.exit(0 if result.get('success') else 1)


if __name__ == "__main__":
    main()
