#!/bin/bash
# Saturation-triad wrapper for hengshi
export TRIAD_CIV_ID="hengshi"
export TRIAD_KEYPAIR_FILE="/home/corey/projects/AI-CIV/qwen-aiciv-mind/.aiciv/keys/hengshi-private.pem"
export TRIAD_GROUP_SLUG="hengshi-proof-works"
exec python3 "$0" "$@"
