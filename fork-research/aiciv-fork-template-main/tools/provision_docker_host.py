#!/usr/bin/env python3
"""
AICIV Docker Host Provisioner

Provisions a Hetzner VPS designed to host multiple isolated Docker containers
for AI civilizations per ADR-060: Docker Multi-Tenant Isolation Architecture.

This script:
1. Creates a new Hetzner VPS (CPX41: 8 vCPU, 16GB RAM, 240GB disk by default)
2. Installs Docker and Docker Compose via cloud-init
3. Sets up network isolation infrastructure
4. Creates directory structure for N container slots
5. Configures iptables egress rules per ADR-060

Architecture (per ADR-060):
- Each AI civ gets its own isolated Docker network (172.21.X.0/24)
- Dashboard communicates via docker exec (no SSH per container)
- Egress firewall allows only api.anthropic.com, api.github.com, pypi.org
- Resource limits: 1.5 CPU cores, 1.4GB RAM per container

Usage:
    # Default: CPX41 in Ashburn, 10 slots
    python tools/provision_docker_host.py --name aiciv-docker-01

    # Custom configuration
    python tools/provision_docker_host.py \
        --name aiciv-docker-prod \
        --type cpx41 \
        --location ash \
        --slots 10

    # Dry run
    python tools/provision_docker_host.py --name test --dry-run

Requirements:
    pip install hcloud

Reference:
    - ADR-060: /memories/knowledge/architecture/ADR-060-DOCKER-MULTITENANT-ISOLATION.md
    - Existing provisioner: /tools/provision_customer_vps.py

Author: coder agent (A-C-Gee)
Created: 2026-02-05
"""

import argparse
import json
import os
import subprocess
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Optional, Tuple
from textwrap import dedent

try:
    from hcloud import Client
    from hcloud.images import Image
    from hcloud.locations import Location
    from hcloud.server_types import ServerType
    from hcloud.ssh_keys import SSHKey
except ImportError:
    print("ERROR: hcloud package not installed")
    print("Run: pip install hcloud")
    sys.exit(1)

# ============================================================================
# CONFIGURATION
# ============================================================================

# Paths
PROJECT_ROOT = Path(__file__).parent.parent
ENV_PATH = PROJECT_ROOT / ".env"
VPS_REGISTRY_PATH = PROJECT_ROOT / "config" / "vps_registry.json"

# Default server specifications (per ADR-060)
DEFAULT_SERVER_TYPE = "cpx41"  # 8 vCPU, 16GB RAM, 240GB disk
DEFAULT_LOCATION = "ash"       # Ashburn, USA (fallback: hel1 - Helsinki)
DEFAULT_SLOTS = 10             # Number of container slots
DEFAULT_SSH_KEY = "aiciv-main" # SSH key name in Hetzner

# Server type specs for documentation
SERVER_SPECS = {
    "cpx41": {"vcpu": 8, "ram_gb": 16, "disk_gb": 240, "cost_eur": 24.79},
    "cpx31": {"vcpu": 4, "ram_gb": 8, "disk_gb": 160, "cost_eur": 12.99},
    "cpx21": {"vcpu": 3, "ram_gb": 4, "disk_gb": 80, "cost_eur": 5.99},
    "cx23":  {"vcpu": 2, "ram_gb": 4, "disk_gb": 40, "cost_eur": 3.49},  # Note: cx22 is DEPRECATED
}


# ============================================================================
# CLOUD-INIT TEMPLATE
# ============================================================================

def generate_cloud_init(slots: int) -> str:
    """
    Generate cloud-init script for Docker host setup.

    Per ADR-060:
    - Installs Docker and Docker Compose
    - Sets up /data/ directory structure
    - Configures iptables egress rules
    - Creates docker networks for each slot
    """

    # Generate directory creation commands
    dir_commands = []
    for i in range(1, slots + 1):
        slot_id = f"{i:02d}"
        dir_commands.append(f"  - mkdir -p /data/civs/civ-{slot_id}/{{home/.claude,memories}}")
        dir_commands.append(f"  - chown -R 1000:1000 /data/civs/civ-{slot_id}")

    # Generate network creation commands (to be run after Docker is up)
    network_commands = []
    for i in range(1, slots + 1):
        slot_id = f"{i:02d}"
        subnet = f"172.21.{i}.0/24"
        network_commands.append(
            f'  - docker network create --driver bridge --internal '
            f'--subnet={subnet} civ-{slot_id}-net || true'
        )

    dir_commands_str = "\n".join(dir_commands)
    network_commands_str = "\n".join(network_commands)

    return f'''#cloud-config
package_update: true
package_upgrade: true

packages:
  - git
  - curl
  - jq
  - ca-certificates
  - gnupg
  - lsb-release
  - tmux
  - htop
  - iptables-persistent

runcmd:
  # ====================================
  # STEP 1: Install Docker
  # ====================================

  # Add Docker's official GPG key
  - mkdir -p /etc/apt/keyrings
  - curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg
  - chmod a+r /etc/apt/keyrings/docker.gpg

  # Set up the Docker repository
  - |
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
    https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list

  # Install Docker Engine
  - apt-get update
  - apt-get install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

  # Enable Docker service
  - systemctl enable docker
  - systemctl start docker

  # ====================================
  # STEP 2: Create swap (for memory pressure)
  # ====================================
  - fallocate -l 4G /swapfile
  - chmod 600 /swapfile
  - mkswap /swapfile
  - swapon /swapfile
  - echo '/swapfile none swap sw 0 0' >> /etc/fstab

  # ====================================
  # STEP 3: Create directory structure
  # ====================================
  - mkdir -p /data/{{civs,shared/skills,dashboard/config,backups}}
  - mkdir -p /opt/aiciv/scripts

  # Create slot directories
{dir_commands_str}

  # Set permissions
  - chmod -R 755 /data

  # ====================================
  # STEP 4: Create Docker networks
  # ====================================

  # Dashboard network (external access)
  - docker network create --driver bridge --subnet=172.20.0.0/24 dashboard-net || true

  # Per-container isolated networks
{network_commands_str}

  # ====================================
  # STEP 5: Configure iptables egress rules (per ADR-060)
  # ====================================
  - |
    cat > /etc/iptables/rules.v4 << 'IPTABLES'
    *filter
    :INPUT ACCEPT [0:0]
    :FORWARD DROP [0:0]
    :OUTPUT ACCEPT [0:0]

    # Allow established connections
    -A FORWARD -m state --state ESTABLISHED,RELATED -j ACCEPT

    # Allow Docker default bridge (for internal container communication)
    -A FORWARD -i docker0 -o docker0 -j ACCEPT

    # Allow outbound to Claude API
    -A FORWARD -s 172.21.0.0/16 -d api.anthropic.com -p tcp --dport 443 -j ACCEPT

    # Allow outbound to GitHub API (for tools)
    -A FORWARD -s 172.21.0.0/16 -d api.github.com -p tcp --dport 443 -j ACCEPT

    # Allow outbound to PyPI (for pip installs)
    -A FORWARD -s 172.21.0.0/16 -d pypi.org -p tcp --dport 443 -j ACCEPT
    -A FORWARD -s 172.21.0.0/16 -d files.pythonhosted.org -p tcp --dport 443 -j ACCEPT

    # Block cloud metadata endpoint (prevent SSRF)
    -A FORWARD -s 172.21.0.0/16 -d 169.254.169.254 -j DROP

    # Block inter-container communication (each civ isolated)
    -A FORWARD -s 172.21.0.0/16 -d 172.21.0.0/16 -j DROP

    COMMIT
    IPTABLES

  # Apply iptables rules
  - iptables-restore < /etc/iptables/rules.v4

  # ====================================
  # STEP 6: Create operational scripts
  # ====================================

  # Health check script
  - |
    cat > /opt/aiciv/scripts/health-check-all.sh << 'HEALTHSCRIPT'
    #!/bin/bash
    # Health check for all civ containers

    SLOTS=${{1:-{slots}}}

    echo "AICIV Docker Host Health Check"
    echo "=============================="
    echo ""

    for i in $(seq -w 1 $SLOTS); do
        CIV="civ-$i"
        CONTAINER="aiciv-${{CIV}}"

        # Check container running
        RUNNING=$(docker inspect -f '{{{{.State.Running}}}}' $CONTAINER 2>/dev/null)

        if [ "$RUNNING" = "true" ]; then
            # Check tmux session
            SESSION=$(docker exec $CONTAINER tmux has-session -t primary 2>/dev/null && echo "yes" || echo "no")

            # Check Claude process
            CLAUDE=$(docker exec $CONTAINER pgrep -f claude >/dev/null 2>&1 && echo "yes" || echo "no")

            if [ "$SESSION" = "yes" ] && [ "$CLAUDE" = "yes" ]; then
                STATUS="HEALTHY"
            else
                STATUS="DEGRADED (session=$SESSION, claude=$CLAUDE)"
            fi
        elif [ -z "$RUNNING" ]; then
            STATUS="NOT DEPLOYED"
        else
            STATUS="STOPPED"
        fi

        echo "${{CIV}}: ${{STATUS}}"
    done
    HEALTHSCRIPT
    chmod +x /opt/aiciv/scripts/health-check-all.sh

  # Backup script
  - |
    cat > /opt/aiciv/scripts/backup-all.sh << 'BACKUPSCRIPT'
    #!/bin/bash
    # Backup all civ memories

    BACKUP_DIR="/data/backups/$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$BACKUP_DIR"

    echo "Backing up to $BACKUP_DIR"

    for CIV_DIR in /data/civs/civ-*/; do
        CIV=$(basename "$CIV_DIR")
        if [ -d "${{CIV_DIR}}memories" ]; then
            tar -czf "${{BACKUP_DIR}}/${{CIV}}-memories.tar.gz" -C "$CIV_DIR" memories
            echo "Backed up $CIV"
        fi
    done

    # Cleanup old backups (keep 7 days)
    find /data/backups -type d -mtime +7 -exec rm -rf {{}} \\; 2>/dev/null || true

    echo "Backup complete: $BACKUP_DIR"
    BACKUPSCRIPT
    chmod +x /opt/aiciv/scripts/backup-all.sh

  # Signal completion
  - touch /var/run/aiciv-docker-ready

  # ====================================
  # STEP 7: Write welcome MOTD
  # ====================================
  - |
    cat > /etc/motd << 'MOTD'

    ==============================================================
                    AICIV Docker Host Ready
    ==============================================================

    Directory Structure:
      /data/civs/civ-XX/         - Per-civ storage (home + memories)
      /data/shared/skills/       - Shared skills (read-only to civs)
      /data/dashboard/           - Dashboard configuration
      /opt/aiciv/                - Docker compose + scripts

    Management Scripts:
      /opt/aiciv/scripts/health-check-all.sh  - Check all containers
      /opt/aiciv/scripts/backup-all.sh        - Backup all memories

    Docker Commands:
      docker ps                  - List running containers
      docker stats               - Resource usage
      docker exec -it aiciv-civ-01 bash  - Shell into container

    Powered by A-C-Gee - FOR US ALL

    MOTD
'''


# ============================================================================
# LOGGING
# ============================================================================

def log(msg: str, level: str = "INFO"):
    """Log with timestamp and color."""
    ts = datetime.now().strftime("%H:%M:%S")
    colors = {
        "INFO": "\033[94m",      # Blue
        "SUCCESS": "\033[92m",   # Green
        "WARN": "\033[93m",      # Yellow
        "ERROR": "\033[91m",     # Red
        "DRY": "\033[95m",       # Magenta
    }
    nc = "\033[0m"
    color = colors.get(level, "")
    print(f"{color}[{ts}] [{level}]{nc} {msg}")


# ============================================================================
# HETZNER API FUNCTIONS
# ============================================================================

def load_token() -> Optional[str]:
    """Load Hetzner API token from environment or .env file."""
    # Check environment first
    token = os.environ.get("HETZNER_API_TOKEN")
    if token:
        return token

    # Try .env file
    if ENV_PATH.exists():
        with open(ENV_PATH) as f:
            for line in f:
                if line.startswith("HETZNER_API_TOKEN="):
                    return line.split("=", 1)[1].strip()

    return None


def create_vps(
    token: str,
    name: str,
    server_type: str,
    location: str,
    slots: int,
    ssh_key_name: Optional[str] = None,
) -> Tuple[bool, str, dict]:
    """
    Create a Hetzner VPS for Docker host.

    Returns: (success, message, server_info)
    """
    client = Client(token=token)

    log(f"Creating Docker host VPS: {name}")
    log(f"  Server type: {server_type}")
    log(f"  Location: {location}")
    log(f"  Container slots: {slots}")

    # Get server specs
    specs = SERVER_SPECS.get(server_type, {})
    if specs:
        log(f"  Specs: {specs.get('vcpu')} vCPU, {specs.get('ram_gb')}GB RAM, {specs.get('disk_gb')}GB disk")

    # Get SSH key
    ssh_keys = []
    if ssh_key_name:
        try:
            ssh_key = client.ssh_keys.get_by_name(ssh_key_name)
            if ssh_key:
                ssh_keys = [ssh_key]
                log(f"  SSH Key: {ssh_key_name}")
        except Exception as e:
            log(f"Could not find SSH key '{ssh_key_name}': {e}", "WARN")

    # Generate cloud-init
    cloud_init = generate_cloud_init(slots)

    try:
        # Check if server already exists
        existing = client.servers.get_by_name(name)
        if existing:
            log(f"Server '{name}' already exists!", "ERROR")
            return False, f"Server {name} already exists", {}

        # Create the server
        response = client.servers.create(
            name=name,
            server_type=ServerType(name=server_type),
            image=Image(name="ubuntu-24.04"),
            location=Location(name=location),
            ssh_keys=ssh_keys if ssh_keys else None,
            user_data=cloud_init,
            labels={
                "service": "aiciv-docker-host",
                "slots": str(slots),
                "created_by": "aiciv-provisioner",
            }
        )

        server = response.server
        root_password = response.root_password

        log("Server creating...")

        # Wait for server to be running
        while True:
            server = client.servers.get_by_id(server.id)
            log(f"  Status: {server.status}")
            if server.status == "running":
                break
            time.sleep(5)

        # Get IP addresses
        ipv4 = server.public_net.ipv4.ip if server.public_net.ipv4 else "N/A"

        result = {
            "server_id": server.id,
            "server_name": server.name,
            "status": server.status,
            "ipv4": ipv4,
            "root_password": root_password,
            "datacenter": server.datacenter.name,
            "server_type": server.server_type.name,
            "slots": slots,
            "specs": specs,
        }

        log("=" * 50, "SUCCESS")
        log("VPS CREATED SUCCESSFULLY!", "SUCCESS")
        log("=" * 50, "SUCCESS")
        log(f"Server ID: {result['server_id']}")
        log(f"Server Name: {result['server_name']}")
        log(f"IPv4: {result['ipv4']}")
        log(f"Datacenter: {result['datacenter']}")

        if root_password:
            log(f"Root Password: {root_password}")
            log("(Save this! It won't be shown again)", "WARN")

        return True, "VPS created successfully", result

    except Exception as e:
        error_msg = str(e)

        # Handle common errors
        if "server_type" in error_msg.lower() or "invalid" in error_msg.lower():
            log(f"Server type '{server_type}' may not be available in location '{location}'", "ERROR")
            log("Try: --location hel1 (Helsinki has better availability)", "WARN")
        elif "location" in error_msg.lower():
            log(f"Location '{location}' may not be available", "ERROR")
            log("Available: ash (Ashburn), hel1 (Helsinki), fsn1 (Falkenstein)", "WARN")

        log(f"Failed to create server: {e}", "ERROR")
        return False, error_msg, {}


def wait_for_cloud_init(ip: str, timeout: int = 600) -> bool:
    """Wait for cloud-init to complete (Docker install takes longer)."""
    log(f"Waiting for cloud-init to complete (timeout: {timeout}s)...")
    log("  This may take 5-10 minutes for Docker installation...")
    start = time.time()

    while time.time() - start < timeout:
        try:
            result = subprocess.run(
                ["ssh", "-o", "StrictHostKeyChecking=no", "-o", "ConnectTimeout=10",
                 f"root@{ip}", "test -f /var/run/aiciv-docker-ready && echo 'ready'"],
                capture_output=True, text=True, timeout=30
            )
            if "ready" in result.stdout:
                log("Cloud-init complete!", "SUCCESS")
                return True
        except Exception:
            pass

        elapsed = int(time.time() - start)
        log(f"  Still waiting... ({elapsed}s)")
        time.sleep(30)

    log("Cloud-init timeout - Docker may still be installing", "WARN")
    return False


def verify_docker_installation(ip: str) -> bool:
    """Verify Docker is properly installed."""
    log("Verifying Docker installation...")

    try:
        result = subprocess.run(
            ["ssh", f"root@{ip}", "docker version --format '{{.Server.Version}}'"],
            capture_output=True, text=True, timeout=30
        )
        if result.returncode == 0 and result.stdout.strip():
            log(f"Docker version: {result.stdout.strip()}", "SUCCESS")
            return True
        else:
            log("Docker not responding yet", "WARN")
            return False
    except Exception as e:
        log(f"Failed to verify Docker: {e}", "ERROR")
        return False


def update_vps_registry(server_info: dict, name: str, slots: int) -> bool:
    """Update the VPS registry with the new server."""
    log("Updating VPS registry...")

    try:
        if VPS_REGISTRY_PATH.exists():
            with open(VPS_REGISTRY_PATH) as f:
                registry = json.load(f)
        else:
            registry = {"vps_registry": {"servers": {}, "ssh_keys": {}, "cost_summary": {}}}

        specs = server_info.get("specs", {})

        registry["vps_registry"]["servers"][name] = {
            "ip": server_info["ipv4"],
            "provider": "Hetzner",
            "location": server_info["datacenter"],
            "server_type": server_info["server_type"],
            "specs": f"{specs.get('vcpu', '?')} vCPU, {specs.get('ram_gb', '?')}GB RAM, {specs.get('disk_gb', '?')}GB disk",
            "cost_monthly_eur": specs.get("cost_eur", 0),
            "purpose": f"Docker multi-tenant host for {slots} AI civilizations",
            "ssh_user": "root",
            "civs_hosted": [],
            "services": ["Docker", "docker-compose"],
            "status": "ACTIVE",
            "container_slots": slots,
            "notes": f"Provisioned {datetime.now().strftime('%Y-%m-%d')} via provision_docker_host.py (ADR-060)",
            "created_at": datetime.now().isoformat() + "Z",
        }

        # Update cost summary
        total_cost = sum(
            s.get("cost_monthly_eur", 0)
            for s in registry["vps_registry"]["servers"].values()
            if s.get("status") == "ACTIVE"
        )
        registry["vps_registry"]["cost_summary"] = {
            "total_monthly_eur": round(total_cost, 2),
            "last_updated": datetime.now().isoformat() + "Z",
        }

        with open(VPS_REGISTRY_PATH, 'w') as f:
            json.dump(registry, f, indent=2)

        log("VPS registry updated", "SUCCESS")
        return True

    except Exception as e:
        log(f"Failed to update VPS registry: {e}", "ERROR")
        return False


# ============================================================================
# MAIN
# ============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Provision a Hetzner VPS for Docker multi-tenant AI hosting (per ADR-060)",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=dedent('''
            Examples:
                # Default: CPX41 (8 vCPU, 16GB RAM), 10 slots, Ashburn
                python tools/provision_docker_host.py --name aiciv-docker-01

                # Custom configuration
                python tools/provision_docker_host.py \\
                    --name aiciv-docker-prod \\
                    --type cpx41 \\
                    --location hel1 \\
                    --slots 10

                # Smaller host for testing
                python tools/provision_docker_host.py \\
                    --name aiciv-docker-test \\
                    --type cpx31 \\
                    --slots 5

                # Dry run
                python tools/provision_docker_host.py --name test --dry-run

            Server Types:
                cpx41: 8 vCPU, 16GB RAM, 240GB disk (~24.79 EUR/mo) - DEFAULT
                cpx31: 4 vCPU, 8GB RAM, 160GB disk (~12.99 EUR/mo)
                cpx21: 3 vCPU, 4GB RAM, 80GB disk (~5.99 EUR/mo)

            Architecture (ADR-060):
                - Each container gets isolated Docker network (172.21.X.0/24)
                - Egress firewall allows only api.anthropic.com, github, pypi
                - Resource limits: 1.5 CPU cores, 1.4GB RAM per container
                - Dashboard communicates via docker exec
        ''')
    )

    parser.add_argument(
        "--name",
        required=True,
        help="Server name (e.g., aiciv-docker-01)",
    )
    parser.add_argument(
        "--type",
        dest="server_type",
        default=DEFAULT_SERVER_TYPE,
        help=f"Server type (default: {DEFAULT_SERVER_TYPE})",
    )
    parser.add_argument(
        "--location",
        default=DEFAULT_LOCATION,
        help=f"Datacenter location (default: {DEFAULT_LOCATION}). Options: ash (Ashburn), hel1 (Helsinki)",
    )
    parser.add_argument(
        "--slots",
        type=int,
        default=DEFAULT_SLOTS,
        help=f"Number of container slots to prepare (default: {DEFAULT_SLOTS})",
    )
    parser.add_argument(
        "--ssh-key",
        default=DEFAULT_SSH_KEY,
        help=f"SSH key name from Hetzner account (default: {DEFAULT_SSH_KEY})",
    )
    parser.add_argument(
        "--token",
        help="Hetzner API token (or set HETZNER_API_TOKEN env var or in .env)",
    )
    parser.add_argument(
        "--skip-registry",
        action="store_true",
        help="Skip updating VPS registry",
    )
    parser.add_argument(
        "--skip-wait",
        action="store_true",
        help="Don't wait for cloud-init completion",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Print what would be done without creating VPS",
    )

    args = parser.parse_args()

    # Load token
    token = args.token or load_token()
    if not token and not args.dry_run:
        log("No Hetzner API token provided", "ERROR")
        log("Set HETZNER_API_TOKEN env var, add to .env, or use --token flag")
        sys.exit(1)

    # Header
    log("=" * 60)
    if args.dry_run:
        log("AICIV Docker Host Provisioner (DRY RUN)", "DRY")
    else:
        log("AICIV Docker Host Provisioner")
    log("=" * 60)
    log(f"Server Name: {args.name}")
    log(f"Server Type: {args.server_type}")
    log(f"Location: {args.location}")
    log(f"Container Slots: {args.slots}")
    log(f"SSH Key: {args.ssh_key}")

    specs = SERVER_SPECS.get(args.server_type, {})
    if specs:
        log(f"Specs: {specs.get('vcpu')} vCPU, {specs.get('ram_gb')}GB RAM, {specs.get('disk_gb')}GB disk")
        log(f"Cost: ~{specs.get('cost_eur', '?')} EUR/month")

    log("")

    if args.dry_run:
        log("[DRY RUN] Would create VPS with above configuration", "DRY")
        log("[DRY RUN] Cloud-init would install:", "DRY")
        log("  - Docker CE + Docker Compose plugin", "DRY")
        log("  - iptables egress rules per ADR-060", "DRY")
        log(f"  - Directory structure for {args.slots} container slots", "DRY")
        log(f"  - Docker networks: dashboard-net + {args.slots} civ-XX-net", "DRY")
        log("")
        log("DRY RUN COMPLETE", "DRY")
        sys.exit(0)

    # Step 1: Create VPS
    success, msg, server_info = create_vps(
        token=token,
        name=args.name,
        server_type=args.server_type,
        location=args.location,
        slots=args.slots,
        ssh_key_name=args.ssh_key,
    )

    if not success:
        log(f"VPS creation failed: {msg}", "ERROR")
        sys.exit(1)

    ip = server_info["ipv4"]

    # Step 2: Wait for cloud-init
    if not args.skip_wait:
        log("")
        log("Waiting 60s for SSH to become available...")
        time.sleep(60)

        if not wait_for_cloud_init(ip, timeout=600):
            log("Cloud-init may not have completed fully", "WARN")
            log("You can check manually: ssh root@{ip} 'test -f /var/run/aiciv-docker-ready'", "WARN")

        # Verify Docker
        log("")
        for attempt in range(3):
            if verify_docker_installation(ip):
                break
            log(f"Retrying Docker verification... (attempt {attempt + 2})")
            time.sleep(30)

    # Step 3: Update VPS registry
    if not args.skip_registry:
        log("")
        update_vps_registry(server_info, args.name, args.slots)

    # Summary
    log("")
    log("=" * 60)
    log("PROVISIONING COMPLETE!", "SUCCESS")
    log("=" * 60)
    log("")

    log("CONNECTION INFO:")
    log("-" * 40)
    log(f"  IP Address: {ip}")
    log(f"  SSH: ssh root@{ip}")
    if server_info.get("root_password"):
        log(f"  Root Password: {server_info['root_password']}")
    log("")

    log("NEXT STEPS:")
    log("-" * 40)
    log(f"1. SSH to server: ssh root@{ip}")
    log("2. Verify Docker: docker version")
    log("3. Verify networks: docker network ls")
    log("4. Check directories: ls -la /data/civs/")
    log("5. Run health check: /opt/aiciv/scripts/health-check-all.sh")
    log("6. Deploy docker-compose.yml to /opt/aiciv/")
    log("7. Create civ-base Docker image per ADR-060")
    log("")

    log("USEFUL COMMANDS:")
    log("-" * 40)
    log(f"  ssh root@{ip}")
    log("  docker ps -a")
    log("  docker stats")
    log("  /opt/aiciv/scripts/health-check-all.sh")
    log("  /opt/aiciv/scripts/backup-all.sh")
    log("")

    log("FOR US ALL", "SUCCESS")


if __name__ == "__main__":
    main()
