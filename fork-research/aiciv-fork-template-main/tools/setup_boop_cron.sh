#!/bin/bash
# Setup BOOP cron with 20-minute cadence
# Run this script to enable autonomous BOOPs

set -e

ACG_DIR="/home/corey/projects/AI-CIV/ACG"
LOGS_DIR="${ACG_DIR}/logs"
BOOP_SCRIPT="${ACG_DIR}/tools/autonomy_nudge.sh"

# Ensure logs directory exists
mkdir -p "$LOGS_DIR"
echo "Created logs directory: $LOGS_DIR"

# Define cron entry (20-minute cadence)
CRON_ENTRY="*/20 * * * * ${BOOP_SCRIPT} >> ${LOGS_DIR}/boop.log 2>&1"

# Check if cron entry already exists
if crontab -l 2>/dev/null | grep -q "autonomy_nudge.sh"; then
    echo "BOOP cron entry already exists. Updating..."
    # Remove old entry and add new one
    (crontab -l 2>/dev/null | grep -v "autonomy_nudge.sh"; echo "$CRON_ENTRY") | crontab -
else
    echo "Adding BOOP cron entry..."
    (crontab -l 2>/dev/null; echo "$CRON_ENTRY") | crontab -
fi

echo ""
echo "BOOP cron setup complete!"
echo ""
echo "Configuration:"
echo "  - Cadence: Every 20 minutes"
echo "  - Script: ${BOOP_SCRIPT}"
echo "  - Log: ${LOGS_DIR}/boop.log"
echo ""
echo "Current crontab:"
crontab -l
echo ""
echo "To verify it's working, wait for the next 20-minute mark or run:"
echo "  ${BOOP_SCRIPT} --status"
