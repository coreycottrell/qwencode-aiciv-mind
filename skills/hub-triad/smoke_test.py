#!/usr/bin/env python3
"""Smoke test for hub-triad identity setup."""

import subprocess
import sys
from pathlib import Path

KEYPAIR = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-private.pem"
HUB_TOKEN = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-hub-token.txt"

def run(cmd: list[str]) -> tuple[int, str, str]:
    r = subprocess.run(cmd, capture_output=True, text=True)
    return r.returncode, r.stdout, r.stderr

def test_keypair_exists():
    if not Path(KEYPAIR).exists():
        print(f"FAIL: keypair not found at {KEYPAIR}")
        return False
    print(f"  keypair exists: PASS")
    return True

def test_hub_token_readable():
    if not Path(HUB_TOKEN).exists():
        print(f"FAIL: hub-token not found at {HUB_TOKEN}")
        return False
    print(f"  hub-token exists: PASS")
    return True

def test_jwt_generation():
    rc, out, err = run([
        "python3", "-c",
        f"import sys; sys.path.insert(0, 'skills/hub-triad'); from triad_client import get_jwt; jwt = get_jwt('hengshi', '{KEYPAIR}'); print('OK' if jwt and len(jwt) > 50 else 'FAIL')"
    ])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: JWT generation failed: {err}")
        return False
    print(f"  JWT generation: PASS")
    return True

def main():
    print("HUB-TRIAD — Identity Smoke Test")
    results = [
        ("keypair exists", test_keypair_exists()),
        ("hub-token readable", test_hub_token_readable()),
        ("JWT generation", test_jwt_generation()),
    ]
    print("\nRESULTS:")
    all_ok = True
    for name, ok in results:
        print(f"  {name}: {'PASS' if ok else 'FAIL'}")
        if not ok:
            all_ok = False
    sys.exit(0 if all_ok else 1)

if __name__ == "__main__":
    main()
