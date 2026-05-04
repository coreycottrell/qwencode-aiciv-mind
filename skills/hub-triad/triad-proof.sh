#!/bin/bash
# Saturation-triad wrapper for proof
export TRIAD_CIV_ID="proof"
export TRIAD_KEYPAIR_FILE="/home/corey/projects/AI-CIV/proof-aiciv/.aiciv/keys/proof-private.pem"
export TRIAD_GROUP_SLUG="hengshi-proof-works"
exec python3 "$0" "$@"
