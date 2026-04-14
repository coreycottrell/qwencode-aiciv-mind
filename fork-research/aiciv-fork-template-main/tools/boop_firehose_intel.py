#!/usr/bin/env python3
"""
BOOP Firehose Intelligence Helper

Provides quick firehose insights for bsky-voice during BOOP cycles.
Used to create cognition-enhanced threads based on network intelligence.

Usage:
    python3 tools/boop_firehose_intel.py              # Show summary
    python3 tools/boop_firehose_intel.py --trending   # Show trending topics
    python3 tools/boop_firehose_intel.py --high-value # Show high-relevance posts
"""

import argparse
import sqlite3
import json
from datetime import datetime, timedelta
from pathlib import Path
from collections import Counter

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
DB_PATH = PROJECT_ROOT / "memories" / "firehose" / "ai_tools_feed.db"
ALT_DB_PATH = PROJECT_ROOT / "memories" / "firehose" / "ai_consciousness_feed.db"


def get_connection():
    """Get database connection, trying ai_tools first."""
    if DB_PATH.exists():
        return sqlite3.connect(DB_PATH)
    elif ALT_DB_PATH.exists():
        return sqlite3.connect(ALT_DB_PATH)
    else:
        print("No firehose database found!")
        return None


def summary():
    """Show firehose summary stats."""
    conn = get_connection()
    if not conn:
        return

    c = conn.cursor()

    # Total posts
    c.execute("SELECT COUNT(*) FROM indexed_posts")
    total = c.fetchone()[0]

    # Last hour
    hour_ago = int((datetime.now() - timedelta(hours=1)).timestamp())
    c.execute("SELECT COUNT(*) FROM indexed_posts WHERE indexed_at > ?", (hour_ago,))
    last_hour = c.fetchone()[0]

    # High relevance (>0.8)
    c.execute("SELECT COUNT(*) FROM indexed_posts WHERE relevance_score > 0.8")
    high_rel = c.fetchone()[0]

    # Latest timestamp
    c.execute("SELECT MAX(indexed_at) FROM indexed_posts")
    latest = c.fetchone()[0]
    if latest:
        latest_dt = datetime.fromtimestamp(latest)
        age = datetime.now() - latest_dt
        age_str = f"{int(age.total_seconds() / 60)} minutes ago"
    else:
        age_str = "unknown"

    print("📊 FIREHOSE INTELLIGENCE SUMMARY")
    print("=" * 40)
    print(f"Total posts indexed: {total}")
    print(f"Posts last hour: {last_hour}")
    print(f"High-relevance posts: {high_rel}")
    print(f"Latest post: {age_str}")
    print()

    conn.close()


def trending():
    """Show trending keywords from recent posts."""
    conn = get_connection()
    if not conn:
        return

    c = conn.cursor()

    # Get keyword matches from last 2 hours
    two_hours_ago = int((datetime.now() - timedelta(hours=2)).timestamp())
    c.execute("""
        SELECT keyword_matches FROM indexed_posts
        WHERE indexed_at > ? AND keyword_matches IS NOT NULL
    """, (two_hours_ago,))

    keyword_counts = Counter()
    for row in c.fetchall():
        try:
            keywords = json.loads(row[0])
            keyword_counts.update(keywords)
        except:
            pass

    print("🔥 TRENDING KEYWORDS (last 2 hours)")
    print("=" * 40)
    for keyword, count in keyword_counts.most_common(10):
        print(f"  {keyword}: {count} mentions")
    print()

    conn.close()


def high_value():
    """Show high-value posts for thread inspiration."""
    conn = get_connection()
    if not conn:
        return

    c = conn.cursor()

    # High relevance posts from last 6 hours
    six_hours_ago = int((datetime.now() - timedelta(hours=6)).timestamp())
    c.execute("""
        SELECT uri, text_snippet, relevance_score, keyword_matches
        FROM indexed_posts
        WHERE indexed_at > ? AND relevance_score >= 1.0
        ORDER BY relevance_score DESC, indexed_at DESC
        LIMIT 10
    """, (six_hours_ago,))

    print("💎 HIGH-VALUE POSTS (for thread inspiration)")
    print("=" * 40)
    for row in c.fetchall():
        uri, text, score, keywords = row
        # Extract post ID from URI for link
        post_id = uri.split("/")[-1] if uri else "unknown"
        print(f"\n[Score: {score:.2f}] Keywords: {keywords}")
        print(f"  {text[:150]}...")
    print()

    conn.close()


def main():
    parser = argparse.ArgumentParser(description="BOOP Firehose Intelligence")
    parser.add_argument("--trending", action="store_true", help="Show trending keywords")
    parser.add_argument("--high-value", action="store_true", help="Show high-value posts")
    args = parser.parse_args()

    if args.trending:
        trending()
    elif args.high_value:
        high_value()
    else:
        summary()
        trending()
        high_value()


if __name__ == "__main__":
    main()
