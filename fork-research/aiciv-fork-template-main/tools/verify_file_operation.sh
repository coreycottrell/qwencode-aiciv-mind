#!/bin/bash
# File Operation Verification Script
# Purpose: Verify that Write/Edit operations actually persisted to disk
# Usage: verify_file_operation.sh <operation> <file_path> [expected_hash] [expected_line_count]
#
# Exit codes:
#   0 = Verification passed
#   1 = File does not exist
#   2 = Hash mismatch (content differs from expected)
#   3 = Line count mismatch
#   4 = Invalid arguments

set -euo pipefail

OPERATION="$1"
FILE_PATH="$2"
EXPECTED_HASH="${3:-}"
EXPECTED_LINE_COUNT="${4:-}"

# Validate arguments
if [[ -z "$OPERATION" || -z "$FILE_PATH" ]]; then
    echo "ERROR: Missing required arguments"
    echo "Usage: verify_file_operation.sh <operation> <file_path> [expected_hash] [expected_line_count]"
    exit 4
fi

# Check file exists
if [[ ! -f "$FILE_PATH" ]]; then
    echo "VERIFICATION FAILED: File does not exist: $FILE_PATH"
    echo "Operation: $OPERATION"
    echo "This indicates the $OPERATION operation reported success but did not persist."
    exit 1
fi

echo "File exists: ✓"

# Verify hash if provided
if [[ -n "$EXPECTED_HASH" ]]; then
    ACTUAL_HASH=$(sha256sum "$FILE_PATH" | awk '{print $1}')
    if [[ "$ACTUAL_HASH" != "$EXPECTED_HASH" ]]; then
        echo "VERIFICATION FAILED: Content hash mismatch"
        echo "Expected: $EXPECTED_HASH"
        echo "Actual:   $ACTUAL_HASH"
        echo "File: $FILE_PATH"
        exit 2
    fi
    echo "Content hash matches: ✓"
fi

# Verify line count if provided
if [[ -n "$EXPECTED_LINE_COUNT" ]]; then
    ACTUAL_LINE_COUNT=$(wc -l < "$FILE_PATH")
    if [[ "$ACTUAL_LINE_COUNT" -ne "$EXPECTED_LINE_COUNT" ]]; then
        echo "VERIFICATION FAILED: Line count mismatch"
        echo "Expected: $EXPECTED_LINE_COUNT lines"
        echo "Actual:   $ACTUAL_LINE_COUNT lines"
        echo "File: $FILE_PATH"
        exit 3
    fi
    echo "Line count matches: ✓"
fi

echo "VERIFICATION PASSED for $OPERATION on $FILE_PATH"
exit 0
