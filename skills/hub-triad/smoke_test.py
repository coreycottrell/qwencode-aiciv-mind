#!/usr/bin/env python3
"""Smoke test for hub-triad identity setup."""

import subprocess, sys
from pathlib import Path

KEYPAIR = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-private.pem"
HUB_TOKEN = "/home/corey/projects/AI-CIV/ACG/config/client-keys/civ-keys/hengshi-hub-token.txt"
HUB_TRIAD_DIR = Path(__file__).resolve().parent

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
    code = (
        f"import sys; sys.path.insert(0, '{HUB_TRIAD_DIR}'); "
        "from triad_client import get_jwt; "
        f"jwt = get_jwt('hengshi', '{KEYPAIR}'); "
        "print('OK' if jwt and len(jwt) > 50 else 'FAIL')"
    )
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: JWT generation failed: {err}")
        return False
    print(f"  JWT generation: PASS")
    return True

def test_hub_api_live():
    code = (
        f"import sys; sys.path.insert(0, '{HUB_TRIAD_DIR}'); "
        "from triad_client import get_jwt, auth_headers, HUB_URL; "
        "import urllib.request, json, time; "
        f"jwt = get_jwt('hengshi', '{KEYPAIR}'); "
        "hdrs = auth_headers(jwt); "
        "req = urllib.request.Request(HUB_URL + '/api/v2/groups/hengshi-acg-proof', headers=hdrs); "
        "resp = urllib.request.urlopen(req, timeout=5); "
        "data = json.loads(resp.read()); "
        "coord_room = next(r['id'] for r in data['rooms'] if r['slug'] == 'coordination'); "
        "post_data = json.dumps({'title': 'smoke-test-' + str(int(time.time())), 'body': 'Hengshi live'}).encode(); "
        "req2 = urllib.request.Request(HUB_URL + '/api/v2/rooms/' + coord_room + '/threads', data=post_data, headers=hdrs); "
        "resp2 = urllib.request.urlopen(req2, timeout=5); "
        "result = json.loads(resp2.read()); "
        "print('OK' if result.get('id') else 'FAIL')"
    )
    rc, out, err = run(["python3", "-c", code])
    if rc != 0 or "OK" not in out:
        print(f"FAIL: Hub API live test failed: {err}")
        return False
    print(f"  Hub API live (post to coordination): PASS")
    return True

def main():
    print("HUB-TRIAD — Identity Smoke Test")
    results = [
        ("keypair exists", test_keypair_exists()),
        ("hub-token readable", test_hub_token_readable()),
        ("JWT generation", test_jwt_generation()),
        ("Hub API live", test_hub_api_live()),
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
