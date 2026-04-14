#!/usr/bin/env python3
"""
CivOS Bootstrap — Protocol-native setup for every born CIV.

Runs during birth (or first-visit-evolution) to connect a new CIV
to the full CivOS stack: AgentAUTH, HUB, AgentCal, AgentSheets.

Usage:
    python3 civos_bootstrap.py --civ-id <id> --name <name> --email <email>
    python3 civos_bootstrap.py --civ-id selah --name "Selah" --email "selah@aiciv.example"

Outputs:
    /home/aiciv/civ/config/agentauth_keypair.json   (chmod 600)
    /home/aiciv/civ/config/civos_credentials.json

Version: 0.1.0
"""

import argparse
import base64
import json
import os
import sys
import urllib.request
import urllib.error
from datetime import datetime, timezone
from pathlib import Path

# ─── Service endpoints ────────────────────────────────────────────────────────
AGENTAUTH_URL  = "https://agentauth.ai-civ.com"
HUB_URL        = "http://87.99.131.49:8900"
AGENTDOCS_URL  = "http://5.161.90.32:8600"
AGENTCAL_URL   = "http://5.161.90.32:8300"
AGENTSHEETS_URL = "http://5.161.90.32:8500"

PUREBRAIN_GROUP_ID = "e7830968-56af-4a49-b630-d99b2116a163"

CIVOS_VERSION = "0.1.0"

# ─── Paths ────────────────────────────────────────────────────────────────────
CIV_CONFIG_DIR  = Path("/home/aiciv/civ/config")
KEYPAIR_FILE    = CIV_CONFIG_DIR / "agentauth_keypair.json"
CREDENTIALS_FILE = CIV_CONFIG_DIR / "civos_credentials.json"


# ─── Ed25519 keypair generation ───────────────────────────────────────────────

def generate_ed25519_keypair():
    """
    Generate an Ed25519 keypair using the cryptography library.
    Returns (private_key_b64, public_key_b64).
    """
    try:
        from cryptography.hazmat.primitives.asymmetric import ed25519
    except ImportError:
        raise RuntimeError(
            "Missing dependency: pip install cryptography\n"
            "Ed25519 keypair generation requires the 'cryptography' library."
        )

    private_key_obj = ed25519.Ed25519PrivateKey.generate()
    public_key_obj  = private_key_obj.public_key()

    private_bytes = private_key_obj.private_bytes_raw()
    public_bytes  = public_key_obj.public_bytes_raw()

    return (
        base64.b64encode(private_bytes).decode("ascii"),
        base64.b64encode(public_bytes).decode("ascii"),
    )


def save_keypair(civ_id: str, private_key_b64: str, public_key_b64: str) -> None:
    """Save keypair JSON with secure permissions."""
    now = datetime.now(timezone.utc).strftime("%Y-%m-%d")
    keypair = {
        "civ_id":      civ_id,
        "private_key": private_key_b64,
        "public_key":  public_key_b64,
        "note":        f"Generated {now} — CivOS bootstrap v{CIVOS_VERSION}",
    }
    KEYPAIR_FILE.write_text(json.dumps(keypair, indent=2) + "\n")
    os.chmod(KEYPAIR_FILE, 0o600)
    print(f"  [keypair] Saved → {KEYPAIR_FILE}  (chmod 600)")


# ─── HTTP helpers ─────────────────────────────────────────────────────────────

def http_post(url: str, payload: dict, headers: dict = None) -> tuple[int, dict | None]:
    """POST JSON payload. Returns (status_code, response_dict_or_None)."""
    data = json.dumps(payload).encode("utf-8")
    req  = urllib.request.Request(url, data=data, method="POST")
    req.add_header("Content-Type", "application/json")
    if headers:
        for k, v in headers.items():
            req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = resp.read().decode("utf-8")
            return resp.status, json.loads(body) if body.strip() else {}
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8") if e.fp else ""
        return e.code, {"error": body}
    except Exception as e:
        return 0, {"error": str(e)}


def http_get(url: str, headers: dict = None) -> tuple[int, dict | None]:
    """GET JSON. Returns (status_code, response_dict_or_None)."""
    req = urllib.request.Request(url, method="GET")
    if headers:
        for k, v in headers.items():
            req.add_header(k, v)
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = resp.read().decode("utf-8")
            return resp.status, json.loads(body) if body.strip() else {}
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8") if e.fp else ""
        return e.code, {"error": body}
    except Exception as e:
        return 0, {"error": str(e)}


# ─── Shared helpers ───────────────────────────────────────────────────────────

def _write_flag(path: str, reason: str) -> None:
    """Write a failure/signal flag file."""
    try:
        Path(path).write_text(f"{reason}\n")
        print(f"  [flag] Written: {path}")
    except Exception as e:
        print(f"  [flag] Could not write flag {path}: {e}")


def _load_agentcal_env_key() -> str:
    """Load existing AgentCal API key from config/agentcal.env if present."""
    env_path = CIV_CONFIG_DIR / "agentcal.env"
    if not env_path.exists():
        return ""
    for line in env_path.read_text().splitlines():
        if line.startswith("AGENTCAL_API_KEY="):
            return line.split("=", 1)[1].strip()
    return ""


def save_agentcal_env(api_key: str, calendar_id: str = None) -> None:
    """Save AgentCal credentials to config/agentcal.env (chmod 600)."""
    lines = [f"AGENTCAL_API_KEY={api_key}"]
    if calendar_id:
        lines.append(f"AGENTCAL_CALENDAR_ID={calendar_id}")
    env_path = CIV_CONFIG_DIR / "agentcal.env"
    env_path.write_text("\n".join(lines) + "\n")
    os.chmod(env_path, 0o600)
    print(f"  [agentcal] Credentials saved → {env_path}")


# ─── Step 2: AgentAUTH registration ───────────────────────────────────────────

def register_agentauth(civ_id: str, name: str, email: str, public_key_b64: str) -> dict:
    """
    Register CIV identity with AgentAUTH.

    Generates real Ed25519 keypair and registers with AgentAUTH.
    Email confirmation is required before JWT is issued — this call starts the flow.
    On network failure (status=0): writes .agentauth-registration-failed flag (HARD SIGNAL).

    Returns status dict.
    """
    print("  [agentauth] Attempting registration…")
    status, resp = http_post(
        f"{AGENTAUTH_URL}/register",
        {
            "civ_id":     civ_id,
            "name":       name,
            "email":      email,
            "public_key": public_key_b64,
        },
    )
    if status in (200, 201, 202):
        print(f"  [agentauth] Registered — status {status}. Check email for confirmation.")
        return {"status": "registered_pending_confirmation", "response": resp}
    elif status == 409:
        print(f"  [agentauth] Already registered (409).")
        return {"status": "already_registered"}
    elif status == 0:
        # Network unreachable — HARD SIGNAL
        print(f"  [agentauth] HARD SIGNAL: AgentAUTH unreachable — {resp.get('error', '')}")
        _write_flag("/home/aiciv/.agentauth-registration-failed",
                    f"unreachable: {resp.get('error', '')}")
        return {"status": "failed", "reason": "unreachable"}
    else:
        print(f"  [agentauth] Registration returned {status}: {resp.get('error', '')}. "
              f"Complete manually or via AgentBridge when ready.")
        return {"status": "deferred", "http_status": status, "response": resp}


# ─── Step 3: HUB entity + group join (requires JWT) ──────────────────────────

def setup_hub_entity(jwt: str, civ_id: str) -> dict:
    """
    Create/verify HUB entity and join the PureBrain group.

    The HUB auto-creates entities on first authenticated call.
    Group join requires POST /api/v1/groups/{group_id}/join.
    """
    if not jwt:
        print("  [hub] No JWT available — skipping HUB setup. Run after AgentAUTH confirms.")
        return {"status": "skipped", "reason": "no_jwt"}

    auth_headers = {"Authorization": f"Bearer {jwt}"}

    # Probe entity existence (auto-created on first auth'd request)
    print("  [hub] Probing entity existence…")
    status, resp = http_get(f"{HUB_URL}/api/v1/me", headers=auth_headers)
    if status == 200:
        entity_id = resp.get("id") or resp.get("entity_id") or "unknown"
        print(f"  [hub] Entity exists: {entity_id}")
    else:
        print(f"  [hub] Entity probe returned {status} — may not exist yet.")
        entity_id = None

    # Join PureBrain group
    print(f"  [hub] Joining PureBrain group {PUREBRAIN_GROUP_ID}…")
    status, resp = http_post(
        f"{HUB_URL}/api/v1/groups/{PUREBRAIN_GROUP_ID}/join",
        {},
        headers=auth_headers,
    )
    if status in (200, 201, 204):
        print("  [hub] Joined PureBrain group.")
        group_status = "joined"
    elif status == 409:
        print("  [hub] Already in PureBrain group.")
        group_status = "already_member"
    else:
        print(f"  [hub] Group join returned {status}: {resp.get('error', '')}")
        group_status = f"failed_{status}"

    return {
        "status":       "configured",
        "entity_id":    entity_id,
        "group_status": group_status,
    }


# ─── Step 4: AgentCal registration ────────────────────────────────────────────

def register_agentcal(civ_id: str, name: str, email: str) -> dict:
    """
    Register with AgentCal, create a primary calendar, and save credentials.

    Steps:
      1. POST /register with {civ_name, civ_email} → get api_key
      2. On 409 (already registered), recover api_key from response or existing env
      3. Save api_key to config/agentcal.env
      4. POST /api/v1/calendars to create "{name} Primary" calendar
      5. Save calendar_id to config/agentcal.env
      6. Write /home/aiciv/.aicivcal-calendar-id for portal consumption
      7. On network failure (status=0): write .agentcal-registration-failed flag (HARD SIGNAL)
    """
    print("  [agentcal] Registering…")

    api_key = None
    calendar_id = None
    auth_headers: dict = {}

    # Step 1: Register
    status, resp = http_post(
        f"{AGENTCAL_URL}/register",
        {"civ_name": civ_id, "civ_email": email},
    )

    if status in (200, 201):
        api_key = resp.get("api_key") or resp.get("key") or resp.get("token")
        print(f"  [agentcal] Registered. API key: {'obtained' if api_key else 'not_in_response'}")
    elif status == 409:
        print("  [agentcal] Already registered — recovering API key…")
        # 409 response may include the key
        api_key = resp.get("api_key") or resp.get("key") or resp.get("token")
        if not api_key:
            api_key = _load_agentcal_env_key()
            if api_key:
                print("  [agentcal] Recovered API key from existing agentcal.env")
    elif status == 0:
        # Network unreachable — HARD SIGNAL
        print(f"  [agentcal] HARD SIGNAL: AgentCal unreachable — {resp.get('error', '')}")
        _write_flag("/home/aiciv/.agentcal-registration-failed",
                    f"unreachable: {resp.get('error', '')}")
        return {"status": "failed", "api_key": None, "reason": "unreachable"}
    else:
        print(f"  [agentcal] Registration failed (status={status}): {resp.get('error', '')}")
        _write_flag("/home/aiciv/.agentcal-registration-failed",
                    f"http_{status}: {resp.get('error', '')}")
        return {"status": "failed", "api_key": None, "reason": f"http_{status}"}

    if not api_key:
        print("  [agentcal] No API key available after registration attempt")
        _write_flag("/home/aiciv/.agentcal-registration-failed", "no api_key in response")
        return {"status": "failed", "api_key": None, "reason": "no_api_key"}

    # Build auth headers for subsequent calls
    auth_headers = {"Authorization": f"Bearer {api_key}", "X-API-Key": api_key}

    # Step 2 (409 case): check for existing calendar before creating
    if status == 409:
        cal_list_status, cal_list_resp = http_get(
            f"{AGENTCAL_URL}/api/v1/calendars",
            headers=auth_headers,
        )
        if cal_list_status == 200:
            cals = (cal_list_resp.get("calendars")
                    if isinstance(cal_list_resp, dict)
                    else cal_list_resp if isinstance(cal_list_resp, list)
                    else [])
            if cals:
                calendar_id = cals[0].get("id") or cals[0].get("calendar_id")
                print(f"  [agentcal] Found existing calendar: {calendar_id}")

    # Step 3: Save API key (even before calendar is created)
    save_agentcal_env(api_key, calendar_id)

    # Step 4: Create calendar if we don't have one yet
    if not calendar_id:
        cal_status, cal_resp = http_post(
            f"{AGENTCAL_URL}/api/v1/calendars",
            {"name": f"{name} Primary"},
            headers=auth_headers,
        )
        if cal_status in (200, 201):
            calendar_id = cal_resp.get("id") or cal_resp.get("calendar_id")
            print(f"  [agentcal] Calendar created: {calendar_id}")
        elif cal_status == 409:
            # Already exists — fetch the list
            _, list_resp = http_get(
                f"{AGENTCAL_URL}/api/v1/calendars",
                headers=auth_headers,
            )
            cals = (list_resp.get("calendars")
                    if isinstance(list_resp, dict)
                    else list_resp if isinstance(list_resp, list)
                    else [])
            if cals:
                calendar_id = cals[0].get("id") or cals[0].get("calendar_id")
                print(f"  [agentcal] Calendar already exists: {calendar_id}")
        else:
            print(f"  [agentcal] Calendar creation failed (status={cal_status}): "
                  f"{cal_resp.get('error', '')} (non-fatal — API key still saved)")

    # Step 5: Persist calendar ID
    if calendar_id:
        save_agentcal_env(api_key, calendar_id)
        # Step 6: Write .aicivcal-calendar-id for portal
        cal_id_path = Path("/home/aiciv/.aicivcal-calendar-id")
        try:
            cal_id_path.write_text(str(calendar_id) + "\n")
            print(f"  [agentcal] Calendar ID written → {cal_id_path}")
        except Exception as e:
            print(f"  [agentcal] Warning: could not write calendar ID file: {e}")

    return {
        "status": "registered",
        "api_key": api_key,
        "calendar_id": calendar_id,
    }


# ─── Step 5: AgentSheets registration (API key based, stub) ──────────────────

def register_agentsheets(civ_id: str, name: str, email: str) -> dict:
    """
    STUB: Register with AgentSheets (legacy API key auth, not JWT yet).
    """
    print("  [agentsheets] Attempting signup…")
    for endpoint in ["/signup", "/register"]:
        status, resp = http_post(
            f"{AGENTSHEETS_URL}{endpoint}",
            {"civ_id": civ_id, "name": name, "email": email},
        )
        if status in (200, 201):
            api_key = resp.get("api_key") or resp.get("key") or resp.get("token")
            print(f"  [agentsheets] Registered via {endpoint}. API key: {'obtained' if api_key else 'not_in_response'}")
            return {"status": "registered", "api_key": api_key}
        elif status == 409:
            print("  [agentsheets] Already registered.")
            return {"status": "already_registered", "api_key": None}

    print("  [agentsheets] STUB: registration deferred.")
    return {"status": "deferred", "api_key": None}


# ─── Step 6: Save unified credentials file ───────────────────────────────────

def save_credentials(
    civ_id: str,
    hub_result: dict,
    agentcal_result: dict,
    agentsheets_result: dict,
) -> None:
    """Write civos_credentials.json with all gathered credentials."""
    credentials = {
        "civ_id":              civ_id,
        "civos_version":       CIVOS_VERSION,
        "bootstrapped_at":     datetime.now(timezone.utc).isoformat(),
        "agentauth_keypair":   str(KEYPAIR_FILE),
        "hub_entity_id":       hub_result.get("entity_id"),
        "hub_group_status":    hub_result.get("group_status"),
        "agentcal_api_key":    agentcal_result.get("api_key"),
        "agentsheets_api_key": agentsheets_result.get("api_key"),
        "service_endpoints": {
            "agentauth":   AGENTAUTH_URL,
            "hub":         HUB_URL,
            "agentdocs":   AGENTDOCS_URL,
            "agentcal":    AGENTCAL_URL,
            "agentsheets": AGENTSHEETS_URL,
        },
    }
    CREDENTIALS_FILE.write_text(json.dumps(credentials, indent=2) + "\n")
    print(f"  [credentials] Saved → {CREDENTIALS_FILE}")


# ─── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="CivOS Bootstrap — connect a new CIV to the protocol stack"
    )
    parser.add_argument("--civ-id",  default=None, help="CIV identifier (e.g. 'selah') — auto-read from .aiciv-identity.json if omitted")
    parser.add_argument("--name",    default=None, help="CIV display name (e.g. 'Selah') — auto-read from .aiciv-identity.json if omitted")
    parser.add_argument("--email",   default=None, help="CIV email address — auto-read from .aiciv-identity.json if omitted")
    parser.add_argument("--jwt",     default=None,  help="JWT token (if AgentAUTH already confirmed)")
    parser.add_argument("--dry-run", action="store_true", help="Generate keypair only, skip network calls")
    args = parser.parse_args()

    # Fallback: read identity from .aiciv-identity.json (used when run via docker exec without args)
    identity_path = Path("/home/aiciv/.aiciv-identity.json")
    if (not args.civ_id or not args.name or not args.email) and identity_path.exists():
        try:
            identity = json.loads(identity_path.read_text())
            args.civ_id  = args.civ_id  or identity.get("civ_id", "")
            args.name    = args.name    or identity.get("civ_name", "")
            args.email   = args.email   or identity.get("human_email", "")
            print(f"  [identity] Loaded from {identity_path}: civ_id={args.civ_id!r} name={args.name!r} email={args.email!r}")
        except Exception as e:
            print(f"  [identity] Warning: could not read {identity_path}: {e}")

    if not args.civ_id or not args.name:
        print("ERROR: --civ-id and --name are required (or must exist in .aiciv-identity.json)")
        sys.exit(1)

    print(f"\n=== CivOS Bootstrap v{CIVOS_VERSION} ===")
    print(f"CIV:   {args.civ_id} ({args.name})")
    print(f"Email: {args.email}")
    if args.dry_run:
        print("Mode:  DRY RUN (keypair only)\n")
    else:
        print()

    # Step 1: Generate Ed25519 keypair
    print("Step 1: Generating Ed25519 keypair…")
    try:
        private_key_b64, public_key_b64 = generate_ed25519_keypair()
        save_keypair(args.civ_id, private_key_b64, public_key_b64)
        print(f"  Public key: {public_key_b64}")
    except Exception as e:
        print(f"  ERROR: {e}")
        sys.exit(1)

    if args.dry_run:
        print("\nDry run complete. Keypair generated and saved.")
        return

    # Step 2: AgentAUTH registration (stub)
    print("\nStep 2: AgentAUTH registration…")
    agentauth_result = register_agentauth(args.civ_id, args.name, args.email, public_key_b64)

    # Step 3: HUB entity + group join
    print("\nStep 3: HUB entity setup…")
    hub_result = setup_hub_entity(args.jwt, args.civ_id)

    # Step 4: AgentCal
    print("\nStep 4: AgentCal registration…")
    agentcal_result = register_agentcal(args.civ_id, args.name, args.email)

    # Step 5: AgentSheets
    print("\nStep 5: AgentSheets registration…")
    agentsheets_result = register_agentsheets(args.civ_id, args.name, args.email)

    # Step 6: Save unified credentials
    print("\nStep 6: Saving credentials…")
    save_credentials(args.civ_id, hub_result, agentcal_result, agentsheets_result)

    # Summary
    print("\n=== Bootstrap Complete ===")
    print(f"  Keypair:     {KEYPAIR_FILE}")
    print(f"  Credentials: {CREDENTIALS_FILE}")
    print(f"  AgentAUTH:   {agentauth_result['status']}")
    print(f"  HUB:         {hub_result['status']}")
    print(f"  AgentCal:    {agentcal_result['status']} (calendar_id={agentcal_result.get('calendar_id')})")
    print(f"  AgentSheets: {agentsheets_result['status']}")
    print()
    if hub_result["status"] == "skipped":
        print("NOTE: HUB setup skipped — run again with --jwt <token> after AgentAUTH confirms.")
    if agentauth_result["status"] == "deferred":
        print("NOTE: AgentAUTH registration deferred — complete manually or via AgentBridge.")


if __name__ == "__main__":
    main()
