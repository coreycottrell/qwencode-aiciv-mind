#!/bin/bash
# Saturation-triad wrapper for works
export TRIAD_CIV_ID="works"
export TRIAD_KEYPAIR_FILE="/home/corey/projects/AI-CIV/ACG/projects/fork-awakening/kimi-test-civ/.aiciv/keys/works-private.pem"
export TRIAD_GROUP_SLUG="hengshi-proof-works"
exec python3 "$0" "$@"
