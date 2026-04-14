#!/usr/bin/env python3
"""
Quick image generation using Gemini API.
"""

import sys
import os
import json
from pathlib import Path
from datetime import datetime

def generate_image(prompt: str, output_path: str = "output.png",
                   aspect_ratio: str = "1:1", resolution: str = "2K"):
    """Generate an image from a text prompt."""

    try:
        from google import genai
        from google.genai import types
    except ImportError:
        print(json.dumps({"success": False, "error": "Install: pip install google-genai"}))
        return None

    api_key = os.getenv("GOOGLE_API_KEY")
    if not api_key:
        print(json.dumps({"success": False, "error": "Set GOOGLE_API_KEY environment variable"}))
        return None

    client = genai.Client(api_key=api_key)

    print(f"Generating: {prompt[:50]}...", file=sys.stderr)
    print(f"Aspect: {aspect_ratio}, Resolution: {resolution}", file=sys.stderr)

    try:
        response = client.models.generate_content(
            model="gemini-2.0-flash-exp-image-generation",
            contents=prompt,
            config=types.GenerateContentConfig(
                response_modalities=['IMAGE'],
            )
        )

        for part in response.parts:
            if hasattr(part, 'text') and part.text:
                print(f"Model said: {part.text}", file=sys.stderr)
            if hasattr(part, 'inline_data') and part.inline_data:
                image = part.as_image()
                image.save(output_path)
                print(f"Saved to: {output_path}", file=sys.stderr)
                print(json.dumps({
                    "success": True,
                    "image_path": str(Path(output_path).absolute()),
                    "prompt": prompt,
                    "aspect_ratio": aspect_ratio
                }))
                return output_path

        print(json.dumps({"success": False, "error": "No image generated"}))
        return None

    except Exception as e:
        print(json.dumps({"success": False, "error": str(e)}))
        return None


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python generate_image.py 'your prompt' [output.png] [16:9] [2K]")
        sys.exit(1)

    prompt = sys.argv[1]
    output = sys.argv[2] if len(sys.argv) > 2 else "output.png"
    aspect = sys.argv[3] if len(sys.argv) > 3 else "1:1"
    res = sys.argv[4] if len(sys.argv) > 4 else "2K"

    generate_image(prompt, output, aspect, res)
