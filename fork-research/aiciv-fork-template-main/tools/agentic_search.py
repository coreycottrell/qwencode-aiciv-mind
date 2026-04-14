#!/usr/bin/env python3
"""
Agentic File Search - Human-like document exploration using Gemini API.

Three-phase search pattern:
1. PARALLEL SCAN: Preview files, ask Gemini which are relevant
2. DEEP DIVE: Full read of top-ranked files
3. BACKTRACK: Follow cross-references if needed

Usage:
    python3 tools/agentic_search.py "query" ./path/ [--max-files N] [--verbose]

Cost: ~$0.001 per query with gemini-2.0-flash
"""

import argparse
import json
import os
import sys
from pathlib import Path
from typing import NamedTuple


class TokenUsage(NamedTuple):
    """Track token usage across phases."""
    phase: str
    input_tokens: int
    output_tokens: int


def load_env():
    """Load environment variables from .env file."""
    env_path = Path(__file__).parent.parent / ".env"
    if env_path.exists():
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, _, value = line.partition("=")
                    os.environ.setdefault(key.strip(), value.strip())


def get_gemini_client():
    """Initialize Gemini client with API key."""
    # Try GOOGLE_API_KEY first (newer), then GEMINI_API_KEY as fallback
    api_key = os.environ.get("GOOGLE_API_KEY") or os.environ.get("GEMINI_API_KEY")
    if not api_key:
        print("ERROR: Neither GOOGLE_API_KEY nor GEMINI_API_KEY found", file=sys.stderr)
        print("Set one in .env or export GOOGLE_API_KEY=...", file=sys.stderr)
        sys.exit(1)

    try:
        import google.generativeai as genai
        genai.configure(api_key=api_key)
        return genai.GenerativeModel("gemini-2.0-flash")
    except ImportError:
        print("ERROR: google-generativeai not installed", file=sys.stderr)
        print("Run: pip install google-generativeai", file=sys.stderr)
        sys.exit(1)


def discover_files(search_path: Path, max_files: int = 50) -> list[Path]:
    """Discover searchable files (markdown, text, json, python)."""
    extensions = {".md", ".txt", ".json", ".py", ".yaml", ".yml"}
    files = []

    for ext in extensions:
        files.extend(search_path.rglob(f"*{ext}"))

    # Sort by modification time (most recent first)
    files.sort(key=lambda f: f.stat().st_mtime, reverse=True)
    return files[:max_files]


def get_file_preview(file_path: Path, max_chars: int = 500) -> str:
    """Get preview of file content (first N chars)."""
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read(max_chars)
            if len(content) == max_chars:
                content += "..."
            return content
    except Exception as e:
        return f"[Error reading file: {e}]"


def phase1_parallel_scan(
    model, query: str, files: list[Path], verbose: bool = False
) -> tuple[list[Path], TokenUsage]:
    """
    Phase 1: Send file previews to Gemini, get ranked relevance.
    Returns: (ranked_files, token_usage)
    """
    if verbose:
        print(f"\n=== PHASE 1: PARALLEL SCAN ({len(files)} files) ===")

    # Build preview document
    previews = []
    for i, f in enumerate(files):
        preview = get_file_preview(f)
        previews.append(f"[{i}] {f.name}\n{preview}\n")

    prompt = f"""You are analyzing files to answer this query: "{query}"

Here are previews of {len(files)} files. Return a JSON array of file indices
ranked by relevance (most relevant first). Include ONLY files that seem
relevant to the query. Format: {{"relevant": [0, 3, 7], "reasoning": "brief explanation"}}

FILES:
{''.join(previews)}

Return ONLY valid JSON, no markdown formatting."""

    try:
        response = model.generate_content(prompt)

        # Extract token usage
        usage = response.usage_metadata
        tokens = TokenUsage(
            phase="scan",
            input_tokens=usage.prompt_token_count,
            output_tokens=usage.candidates_token_count
        )

        # Parse response
        text = response.text.strip()
        # Handle potential markdown code blocks
        if text.startswith("```"):
            text = text.split("```")[1]
            if text.startswith("json"):
                text = text[4:]

        result = json.loads(text)
        relevant_indices = result.get("relevant", [])

        if verbose:
            print(f"Gemini found {len(relevant_indices)} relevant files")
            print(f"Reasoning: {result.get('reasoning', 'N/A')}")

        ranked_files = [files[i] for i in relevant_indices if i < len(files)]
        return ranked_files, tokens

    except json.JSONDecodeError as e:
        if verbose:
            print(f"Warning: Could not parse Gemini response: {e}")
            print(f"Raw response: {response.text[:200]}")
        # Fall back to returning all files
        return files[:5], TokenUsage("scan", 0, 0)
    except Exception as e:
        print(f"Error in Phase 1: {e}", file=sys.stderr)
        return files[:5], TokenUsage("scan", 0, 0)


def phase2_deep_dive(
    model, query: str, files: list[Path], max_files: int = 5, verbose: bool = False
) -> tuple[str, list[str], TokenUsage]:
    """
    Phase 2: Read full content of top files, extract answer + cross-references.
    Returns: (answer, cross_references, token_usage)
    """
    if verbose:
        print(f"\n=== PHASE 2: DEEP DIVE ({min(len(files), max_files)} files) ===")

    # Read full content of top files
    contents = []
    for f in files[:max_files]:
        try:
            with open(f, "r", encoding="utf-8", errors="ignore") as fp:
                content = fp.read()
                # Truncate very long files
                if len(content) > 8000:
                    content = content[:8000] + "\n...[truncated]..."
                contents.append(f"=== FILE: {f} ===\n{content}\n")
                if verbose:
                    print(f"  Reading: {f.name} ({len(content)} chars)")
        except Exception as e:
            if verbose:
                print(f"  Error reading {f}: {e}")

    prompt = f"""Based on these documents, answer the query: "{query}"

{chr(10).join(contents)}

Provide:
1. A clear, comprehensive answer based on the documents
2. Any cross-references mentioned (other files, documents, or paths that might contain more info)

Format your response as JSON:
{{
  "answer": "your detailed answer here",
  "cross_references": ["path/to/file.md", "other/reference.txt"],
  "sources_used": ["list of files that contributed to answer"]
}}

Return ONLY valid JSON, no markdown formatting."""

    try:
        response = model.generate_content(prompt)

        usage = response.usage_metadata
        tokens = TokenUsage(
            phase="dive",
            input_tokens=usage.prompt_token_count,
            output_tokens=usage.candidates_token_count
        )

        text = response.text.strip()
        if text.startswith("```"):
            text = text.split("```")[1]
            if text.startswith("json"):
                text = text[4:]

        result = json.loads(text)
        answer = result.get("answer", "No answer found")
        cross_refs = result.get("cross_references", [])

        if verbose:
            sources = result.get("sources_used", [])
            print(f"Answer synthesized from {len(sources)} sources")
            if cross_refs:
                print(f"Cross-references found: {cross_refs}")

        return answer, cross_refs, tokens

    except json.JSONDecodeError:
        # Return raw text if JSON parsing fails
        return response.text, [], TokenUsage("dive", 0, 0)
    except Exception as e:
        print(f"Error in Phase 2: {e}", file=sys.stderr)
        return f"Error: {e}", [], TokenUsage("dive", 0, 0)


def phase3_backtrack(
    model, query: str, cross_refs: list[str], base_path: Path,
    verbose: bool = False
) -> tuple[str, TokenUsage]:
    """
    Phase 3: Follow cross-references if they exist and are accessible.
    Returns: (additional_info, token_usage)
    """
    if not cross_refs:
        return "", TokenUsage("backtrack", 0, 0)

    if verbose:
        print(f"\n=== PHASE 3: BACKTRACK ({len(cross_refs)} references) ===")

    # Try to find and read cross-referenced files
    found_content = []
    for ref in cross_refs:
        # Try multiple path resolutions
        candidates = [
            base_path / ref,
            Path(ref),
            base_path.parent / ref,
        ]

        for candidate in candidates:
            if candidate.exists() and candidate.is_file():
                try:
                    with open(candidate, "r", encoding="utf-8", errors="ignore") as f:
                        content = f.read()
                        if len(content) > 4000:
                            content = content[:4000] + "\n...[truncated]..."
                        found_content.append(f"=== {ref} ===\n{content}\n")
                        if verbose:
                            print(f"  Found: {candidate}")
                        break
                except Exception:
                    pass

    if not found_content:
        if verbose:
            print("  No accessible cross-references found")
        return "", TokenUsage("backtrack", 0, 0)

    prompt = f"""Original query: "{query}"

I found these additional referenced documents. Extract any additional relevant
information that adds to the answer:

{chr(10).join(found_content)}

Provide a brief summary of additional relevant information found, or "No additional info"
if nothing new is relevant. Keep it concise."""

    try:
        response = model.generate_content(prompt)

        usage = response.usage_metadata
        tokens = TokenUsage(
            phase="backtrack",
            input_tokens=usage.prompt_token_count,
            output_tokens=usage.candidates_token_count
        )

        return response.text.strip(), tokens

    except Exception as e:
        if verbose:
            print(f"  Error in backtrack: {e}")
        return "", TokenUsage("backtrack", 0, 0)


def print_usage_report(usages: list[TokenUsage]):
    """Print token usage summary and estimated cost."""
    print("\n--- Token Usage ---")

    total_input = 0
    total_output = 0

    for u in usages:
        if u.input_tokens or u.output_tokens:
            print(f"Phase {u.phase}: {u.input_tokens + u.output_tokens:,} tokens "
                  f"(in: {u.input_tokens:,}, out: {u.output_tokens:,})")
            total_input += u.input_tokens
            total_output += u.output_tokens

    total = total_input + total_output

    # gemini-2.0-flash pricing: $0.10/1M input, $0.40/1M output
    cost = (total_input * 0.10 / 1_000_000) + (total_output * 0.40 / 1_000_000)

    print(f"Total: {total:,} tokens (~${cost:.6f})")


def main():
    parser = argparse.ArgumentParser(
        description="Agentic file search using Gemini API"
    )
    parser.add_argument("query", help="Search query")
    parser.add_argument("path", nargs="?", default=".", help="Path to search (default: .)")
    parser.add_argument("--max-files", type=int, default=30,
                        help="Max files to scan (default: 30)")
    parser.add_argument("--verbose", "-v", action="store_true",
                        help="Show detailed progress")

    args = parser.parse_args()

    # Load environment and initialize
    load_env()
    model = get_gemini_client()
    search_path = Path(args.path).resolve()

    if not search_path.exists():
        print(f"ERROR: Path not found: {search_path}", file=sys.stderr)
        sys.exit(1)

    if args.verbose:
        print(f"Query: {args.query}")
        print(f"Search path: {search_path}")

    usages = []

    # Phase 1: Discover and scan
    files = discover_files(search_path, args.max_files)
    if not files:
        print("No searchable files found", file=sys.stderr)
        sys.exit(1)

    if args.verbose:
        print(f"Found {len(files)} files to scan")

    ranked_files, usage1 = phase1_parallel_scan(model, args.query, files, args.verbose)
    usages.append(usage1)

    if not ranked_files:
        print("No relevant files found for query")
        print_usage_report(usages)
        sys.exit(0)

    # Phase 2: Deep dive
    answer, cross_refs, usage2 = phase2_deep_dive(
        model, args.query, ranked_files, verbose=args.verbose
    )
    usages.append(usage2)

    # Phase 3: Backtrack if needed
    additional = ""
    if cross_refs:
        additional, usage3 = phase3_backtrack(
            model, args.query, cross_refs, search_path, args.verbose
        )
        usages.append(usage3)

    # Output results
    print("\n" + "=" * 60)
    print("ANSWER:")
    print("=" * 60)
    print(answer)

    if additional and additional != "No additional info":
        print("\n--- Additional from cross-references ---")
        print(additional)

    print("\n--- Sources ---")
    for f in ranked_files[:5]:
        print(f"  - {f}")

    print_usage_report(usages)


if __name__ == "__main__":
    main()
