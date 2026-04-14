#!/usr/bin/env python3

import json
import re

# Read the log file
with open('data/hum/2026-04-05.jsonl', 'r') as f:
    lines = f.readlines()

# Parse each line and find entries with challenger_warnings > 0
challenger_entries = []

for line_num, line in enumerate(lines, 1):
    try:
        entry = json.loads(line)
        if entry.get('challenger_warnings', 0) > 0:
            challenger_entries.append({
                'line_num': line_num,
                'timestamp': entry.get('timestamp', 'N/A'),
                'mission_id': entry.get('mission_id', 'N/A'),
                'task_description': entry.get('task_description', 'N/A'),
                'challenger_warnings': entry.get('challenger_warnings', 0),
                'tools_used': entry.get('tools_used', [])
            })
    except json.JSONDecodeError:
        continue

# Print the results
print(f"Found {len(challenger_entries)} entries with challenger_warnings > 0:")
print("=" * 100)

for entry in challenger_entries:
    print(f"Line {entry['line_num']}:")
    print(f"  Timestamp: {entry['timestamp']}")
    print(f"  Mission ID: {entry['mission_id']}")
    print(f"  Challenger Warnings: {entry['challenger_warnings']}")
    print(f"  Task Description: {entry['task_description'][:100]}...")
    print(f"  Tools Used: {', '.join(entry['tools_used'][:5])}")
    print("-" * 100)

# Write to a file for reference
with open('challenger_warnings_analysis.txt', 'w') as f:
    for entry in challenger_entries:
        f.write(f"Line {entry['line_num']}:")
        f.write(f"  Timestamp: {entry['timestamp']}\n")
        f.write(f"  Mission ID: {entry['mission_id']}\n")
        f.write(f"  Challenger Warnings: {entry['challenger_warnings']}\n")
        f.write(f"  Task Description: {entry['task_description']}\n")
        f.write(f"  Tools Used: {', '.join(entry['tools_used'])}\n")
        f.write("-" * 100 + "\n")

print(f"\nDetailed analysis written to challenger_warnings_analysis.txt")
