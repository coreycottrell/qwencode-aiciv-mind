#!/usr/bin/env python3
"""compute_cir.py — Compound Intelligence Rate tracker.

Computes CIR = M × E × F × Scale for the Qwen mind civilization.
With --pod flag: also computes PodCIR = avg(CIR) × C × S × L

Usage:
    python3 tools/compute_cir.py                    # today
    python3 tools/compute_cir.py --date 2026-04-13  # specific date
    python3 tools/compute_cir.py --json              # JSON output
    python3 tools/compute_cir.py --pod               # Pod CIR across civs

Data sources scanned:
    - minds/minds/**/*.md          (memory files)
    - minds/fitness/**/*.jsonl     (fitness scores)
    - minds/scratchpads/**/*.md    (engagement)
    - minds/manifests/**/*.json    (mind count, session counts)
    - exports/outgoing/            (output artifacts)
    - aiciv-comms-hub messages     (cross-civ communication)
    - tmux sessions                (active minds)

Pod data sources (with --pod):
    - aiciv-comms-hub/rooms/       (cross-civ messages)
    - proof-aiciv/exports/outgoing/ (Proof's artifacts)
    - All civs' skills and rules   (learning transfer)
"""

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timedelta, timezone
from pathlib import Path

# ── Configuration ────────────────────────────────────────────────────────────

ROOT = Path(__file__).resolve().parent.parent  # qwen-aiciv-mind/
ACG_ROOT = ROOT.parent / "ACG"
COMMS_HUB = ROOT.parent / "aiciv-comms-hub"

# Normalization targets (what "1.0" looks like)
M_TARGET = 1000       # 1000+ memory files = 1.0
E_TARGET = 20         # 20 scratchpad+fitness entries/day = 1.0
F_TARGET = 0.85       # 0.85 avg fitness = 1.0
SCALE_TARGET = 50     # 50 active minds = 1.0


def find_files(pattern: str, root: Path) -> list[Path]:
    """Find files matching a glob pattern."""
    try:
        return sorted(root.glob(pattern))
    except Exception:
        return []


def get_mind_files(root: Path, subdir: str, pattern: str) -> list[Path]:
    """Find mind files in a subdirectory."""
    base = root / "minds" / subdir
    if not base.exists():
        return []
    return sorted(base.rglob(pattern))


def parse_fitness_entries(filepath: Path, target_date: str) -> list[dict]:
    """Parse fitness JSONL entries for a specific date."""
    entries = []
    try:
        for line in filepath.read_text().strip().split("\n"):
            if not line.strip():
                continue
            try:
                entry = json.loads(line)
                ts = entry.get("timestamp", "")
                if ts.startswith(target_date):
                    entries.append(entry)
            except json.JSONDecodeError:
                continue
    except Exception:
        pass
    return entries


def parse_fitness_all(filepath: Path) -> list[dict]:
    """Parse all fitness entries from a JSONL file."""
    entries = []
    try:
        for line in filepath.read_text().strip().split("\n"):
            if not line.strip():
                continue
            try:
                entry = json.loads(line)
                entries.append(entry)
            except json.JSONDecodeError:
                continue
    except Exception:
        pass
    return entries


def count_scratchpad_writes(scratchpads: list[Path], target_date: str) -> tuple[int, int]:
    """Count scratchpad files written on target date and total words."""
    count = 0
    words = 0
    target_suffix = f"{target_date}.md"
    for f in scratchpads:
        if f.name == target_suffix:
            count += 1
            try:
                words += len(f.read_text().split())
            except Exception:
                pass
    return count, words


def parse_manifests(manifest_files: list[Path]) -> list[dict]:
    """Parse manifest JSON files."""
    manifests = []
    for f in manifest_files:
        try:
            data = json.loads(f.read_text())
            manifests.append(data)
        except Exception:
            pass
    return manifests


def get_active_tmux_sessions() -> list[str]:
    """Get list of active tmux sessions."""
    try:
        result = subprocess.run(
            ["tmux", "list-sessions"],
            capture_output=True, text=True, timeout=5
        )
        if result.returncode == 0:
            return result.stdout.strip().split("\n")
    except Exception:
        pass
    return []


def count_outgoing_artifacts(exports_dir: Path, target_date: str) -> tuple[int, int]:
    """Count outgoing export artifacts for a date."""
    count = 0
    words = 0
    if not exports_dir.exists():
        return 0, 0
    for f in exports_dir.iterdir():
        if f.is_file() and target_date in f.name:
            count += 1
            try:
                words += len(f.read_text().split())
            except Exception:
                pass
        elif f.is_dir() and target_date in f.name:
            # Count files inside dated directories
            for inner in f.rglob("*"):
                if inner.is_file() and inner.suffix in (".md", ".json"):
                    count += 1
                    try:
                        words += len(inner.read_text().split())
                    except Exception:
                        pass
    return count, words


def count_comms_messages(comms_dir: Path, target_date: str) -> int:
    """Count comms hub messages for a date."""
    count = 0
    if not comms_dir.exists():
        return 0
    for f in comms_dir.iterdir():
        if f.is_file() and target_date in f.name:
            count += 1
        elif f.is_dir():
            # Check inside dated directories
            date_dir = comms_dir / "messages" / target_date.replace("-", "")
            if date_dir.exists():
                count += len(list(date_dir.iterdir()))
    # Also check messages/ subdirectory
    msgs_dir = comms_dir / "messages"
    if msgs_dir.exists():
        for f in msgs_dir.iterdir():
            if f.is_file() and target_date in f.name:
                count += 1
    return count


def compute_cir(target_date: str):
    """Compute the Compound Intelligence Rate for a given date."""
    date_dt = datetime.strptime(target_date, "%Y-%m-%d").replace(tzinfo=timezone.utc)

    # ── M: Memory Depth ──────────────────────────────────────────────────
    memory_files = get_mind_files(ROOT, "minds", "*.md")
    # Also count civilizational memories
    memory_files += get_mind_files(ROOT, "minds/_civilizational", "*.md")
    m_count = len(memory_files)

    # Estimate average depth from memory content
    depth_scores = []
    for f in memory_files[:50]:  # sample up to 50
        try:
            text = f.read_text()
            if "depth_score" in text:
                for line in text.split("\n"):
                    if "depth_score" in line and ":" in line:
                        try:
                            val = float(line.split(":")[1].strip())
                            depth_scores.append(val)
                        except (ValueError, IndexError):
                            pass
        except Exception:
            pass

    avg_depth = sum(depth_scores) / len(depth_scores) if depth_scores else 0.0
    m_factor = min(1.0, m_count / M_TARGET)

    # ── E: Engagement ────────────────────────────────────────────────────
    scratchpad_files = get_mind_files(ROOT, "scratchpads", "*.md")
    scratchpad_writes, scratchpad_words = count_scratchpad_writes(
        scratchpad_files, target_date
    )

    # Fitness entries for the target date
    fitness_files = get_mind_files(ROOT, "fitness", "*.jsonl")
    daily_fitness_entries = []
    for ff in fitness_files:
        daily_fitness_entries.extend(parse_fitness_entries(ff, target_date))

    # Outgoing artifacts
    outgoing_count, outgoing_words = count_outgoing_artifacts(
        ROOT / "exports" / "outgoing", target_date
    )

    # Comms messages
    comms_count = count_comms_messages(COMMS_HUB / "rooms" / "daily-updates", target_date)

    # Total engagement events
    e_total = scratchpad_writes + len(daily_fitness_entries) + outgoing_count + comms_count
    e_factor = min(1.0, e_total / E_TARGET)

    # ── F: Fitness Quality ───────────────────────────────────────────────
    all_fitness_entries = []
    for ff in fitness_files:
        all_fitness_entries.extend(parse_fitness_all(ff))

    if all_fitness_entries:
        scores = [e.get("score", 0.0) for e in all_fitness_entries if "score" in e]
        avg_fitness = sum(scores) / len(scores) if scores else 0.0
    else:
        avg_fitness = 0.0

    f_factor = min(1.0, avg_fitness / F_TARGET) if avg_fitness > 0 else 0.0

    # ── Scale: Mind Count ────────────────────────────────────────────────
    manifest_files = get_mind_files(ROOT, "manifests", "*.json")
    manifests = parse_manifests(manifest_files)

    # Active minds = those with session_count > 0
    active_minds = [m for m in manifests if m.get("session_count", 0) > 0]
    total_minds = len(manifests)
    scale_factor = min(1.0, len(active_minds) / SCALE_TARGET)

    # ── Tmux sessions ────────────────────────────────────────────────────
    tmux_sessions = get_active_tmux_sessions()

    # ── Compute CIR ──────────────────────────────────────────────────────
    cir = m_factor * e_factor * f_factor * scale_factor

    # ── Bottleneck Diagnosis ─────────────────────────────────────────────
    factors = {
        "M (Memory Depth)": m_factor,
        "E (Engagement)": e_factor,
        "F (Fitness Quality)": f_factor,
        "Scale": scale_factor,
    }
    bottleneck = min(factors, key=factors.get)
    bottleneck_value = factors[bottleneck]

    # Build diagnosis messages
    diagnoses = {
        "M (Memory Depth)": (
            f"Memory. {m_count} memory files found (target: {M_TARGET}). "
            f"Avg depth score: {avg_depth:.2f}. "
            "Minds are active but not persisting enough knowledge."
        ),
        "E (Engagement)": (
            f"Engagement. Only {e_total} activity events on {target_date} "
            f"(scratchpad writes: {scratchpad_writes}, fitness entries: {len(daily_fitness_entries)}, "
            f"outgoing artifacts: {outgoing_count}, comms messages: {comms_count}). "
            f"Target: {E_TARGET}/day. Minds have memory and quality but low daily activity."
        ),
        "F (Fitness Quality)": (
            f"Fitness. Average score: {avg_fitness:.2f} (target: {F_TARGET}). "
            f"Based on {len(all_fitness_entries)} total entries across {len(fitness_files)} minds. "
            "Minds are active but performing poorly — check task difficulty and LLM quality."
        ),
        "Scale": (
            f"Scale. {len(active_minds)} active minds (target: {SCALE_TARGET}). "
            f"{total_minds} total minds defined. "
            "Quality is high but too few minds. Spawn more agents and team leads."
        ),
    }

    return {
        "date": target_date,
        "cir": cir,
        "factors": factors,
        "bottleneck": bottleneck,
        "bottleneck_diagnosis": diagnoses[bottleneck],
        "details": {
            "memory_files": m_count,
            "avg_memory_depth": round(avg_depth, 3),
            "scratchpad_writes": scratchpad_writes,
            "scratchpad_words": scratchpad_words,
            "daily_fitness_entries": len(daily_fitness_entries),
            "total_fitness_entries": len(all_fitness_entries),
            "avg_fitness": round(avg_fitness, 3),
            "outgoing_artifacts": outgoing_count,
            "outgoing_words": outgoing_words,
            "comms_messages": comms_count,
            "engagement_total": e_total,
            "total_minds": total_minds,
            "active_minds": len(active_minds),
            "tmux_sessions": len(tmux_sessions),
        },
        "manifests": [
            {
                "identity": m.get("identity", "unknown"),
                "role": m.get("role", "unknown"),
                "session_count": m.get("session_count", 0),
                "growth_stage": m.get("growth_stage", "unknown"),
            }
            for m in manifests
        ],
    }


def print_report(result: dict):
    """Print a human-readable CIR report."""
    d = result["date"]
    cir = result["cir"]
    factors = result["factors"]

    print(f"\nCIR Report for {d}")
    print("=" * 50)
    print()
    print("CIR = M × E × F × Scale")

    # Build the equation string
    m_str = f"{factors['M (Memory Depth)']:.2f}"
    e_str = f"{factors['E (Engagement)']:.2f}"
    f_str = f"{factors['F (Fitness Quality)']:.2f}"
    s_str = f"{factors['Scale']:.2f}"
    print(f"    = {m_str} × {e_str} × {f_str} × {s_str}")
    print(f"    = {cir:.4f}")
    print()
    print("Factors:")
    for name, value in factors.items():
        print(f"  {name:22s} {value:.2f}")

    print()
    print("Details:")
    det = result["details"]
    print(f"  Memory files:          {det['memory_files']}")
    print(f"  Avg memory depth:      {det['avg_memory_depth']}")
    print(f"  Scratchpad writes:     {det['scratchpad_writes']} ({det['scratchpad_words']} words)")
    print(f"  Fitness entries (day): {det['daily_fitness_entries']}")
    print(f"  Fitness entries (all): {det['total_fitness_entries']}")
    print(f"  Avg fitness score:     {det['avg_fitness']}")
    print(f"  Outgoing artifacts:    {det['outgoing_artifacts']} ({det['outgoing_words']} words)")
    print(f"  Comms messages:        {det['comms_messages']}")
    print(f"  Total minds:           {det['total_minds']}")
    print(f"  Active minds:          {det['active_minds']}")
    print(f"  Active tmux sessions:  {det['tmux_sessions']}")

    print()
    print(f"Bottleneck: {result['bottleneck']}")
    print(f"  {result['bottleneck_diagnosis']}")

    # Print mind summary
    if result["manifests"]:
        print()
        print("Minds:")
        for m in result["manifests"]:
            active = "✓" if m["session_count"] > 0 else " "
            print(f"  [{active}] {m['identity']:20s} {m['role']:12s} "
                  f"sessions={m['session_count']:3d}  stage={m['growth_stage']}")

    print()


def print_json(result: dict):
    """Print JSON output."""
    print(json.dumps(result, indent=2, default=str))


# ── Pod CIR ──────────────────────────────────────────────────────────────────

POD_CIVS = {
    "acg": {
        "root": ROOT.parent / "ACG",
        "exports": ROOT.parent / "ACG" / "exports" / "outgoing",
    },
    "proof": {
        "root": Path("/home/corey/projects/AI-CIV/proof-aiciv"),
        "exports": Path("/home/corey/projects/AI-CIV/proof-aiciv/exports/outgoing"),
    },
    "qwen": {
        "root": ROOT,
        "exports": ROOT / "exports" / "outgoing",
    },
}

COORD_TARGET = {2: 10, 3: 20, 4: 30, 5: 40}

ORIGIN_MARKERS = {
    "proof": ["proof", "proof-aiciv", "self-bug-finder", "reasoning-auditor"],
    "acg": ["acg", "a-c-gee", "cardinal-rules", "keel", "parallax"],
    "qwen": ["qwen", "hengshi", "qwen-aiciv-mind"],
}


def count_cross_civ_messages(target_date: str) -> tuple[dict, int]:
    """Count cross-civ messages via comms-hub, send_to_civ logs, scratchpad mentions."""
    counts = {civ: 0 for civ in POD_CIVS}
    total = 0

    msgs_dir = COMMS_HUB / "rooms" / "daily-updates" / "messages"
    if msgs_dir.exists():
        for f in msgs_dir.iterdir():
            if f.is_file() and target_date in f.name:
                text = f.read_text().lower()
                for civ_name in POD_CIVS:
                    if civ_name.lower() in text:
                        counts[civ_name] += 1
                        total += 1

    log_file = ROOT / "logs" / "send_to_civ.log"
    if log_file.exists():
        for line in log_file.read_text().strip().split("\n"):
            if not line.strip():
                continue
            try:
                entry = json.loads(line)
                if entry.get("timestamp", "").startswith(target_date):
                    civ = entry.get("civ", "").lower()
                    if civ in counts:
                        counts[civ] += 1
                        total += 1
            except json.JSONDecodeError:
                continue

    scratchpads = get_mind_files(ROOT, "scratchpads", "*.md")
    for f in scratchpads:
        if target_date in f.name or f.name == f"{target_date}.md":
            try:
                text = f.read_text().lower()
                for civ_name in POD_CIVS:
                    if civ_name.lower() in text:
                        counts[civ_name] += 1
                        total += 1
            except Exception:
                pass

    return counts, total


def compute_specialization() -> dict:
    """Compute specialization: unique domain tags per civ / total domain tags."""
    civ_domains = {}
    all_domains = set()

    for civ_name, civ_info in POD_CIVS.items():
        domains = set()
        manifests_dir = civ_info["root"] / "minds" / "manifests"
        if manifests_dir.exists():
            for mf in manifests_dir.rglob("*.json"):
                try:
                    data = json.loads(mf.read_text())
                    vertical = data.get("vertical") or data.get("domain") or data.get("specialty")
                    if vertical:
                        domains.add(vertical.lower())
                except Exception:
                    pass
        skills_dir = civ_info["root"] / ".claude" / "skills"
        if skills_dir.exists():
            for sf in skills_dir.rglob("*.md"):
                try:
                    text = sf.read_text()
                    for line in text.split("\n")[:15]:
                        if "category:" in line.lower():
                            domains.add(f"skill:{line.split(':', 1)[1].strip().lower()}")
                        if "domain:" in line.lower():
                            domains.add(line.split(":", 1)[1].strip().lower())
                except Exception:
                    pass
        civ_domains[civ_name] = domains
        all_domains.update(domains)

    total_civ_domains = sum(len(d) for d in civ_domains.values())
    overlap = total_civ_domains - len(all_domains) if total_civ_domains > 0 else 0
    overlap_waste = overlap / total_civ_domains if total_civ_domains > 0 else 0
    domain_coverage = min(1.0, len(all_domains) / 10)
    specialization = (1 - overlap_waste) * domain_coverage

    return {
        "specialization": specialization,
        "civ_domains": {k: sorted(v) for k, v in civ_domains.items()},
        "total_unique_domains": len(all_domains),
        "overlap_count": overlap,
    }


def compute_learning_transfer() -> dict:
    """Compute learning transfer: items in civ B that originated from civ A."""
    transfer_counts = {}
    total_shared = 0

    for civ_name, civ_info in POD_CIVS.items():
        received_from_others = 0
        total_items = 0

        skills_dir = civ_info["root"] / ".claude" / "skills"
        if skills_dir.exists():
            for sf in skills_dir.rglob("*.md"):
                total_items += 1
                try:
                    text = sf.read_text().lower()
                    for other_civ, markers in ORIGIN_MARKERS.items():
                        if other_civ == civ_name:
                            continue
                        if any(m.lower() in text for m in markers):
                            if "source:" in text or "author:" in text or "adapted from" in text:
                                received_from_others += 1
                                break
                except Exception:
                    pass

        exports_dir = civ_info.get("exports")
        if exports_dir and exports_dir.exists():
            for ef in exports_dir.rglob("*.md"):
                total_items += 1
                try:
                    text = ef.read_text().lower()
                    for other_civ, markers in ORIGIN_MARKERS.items():
                        if other_civ == civ_name:
                            continue
                        if any(m.lower() in text for m in markers):
                            received_from_others += 1
                            break
                except Exception:
                    pass

        transfer_counts[civ_name] = {
            "received_from_others": received_from_others,
            "total_items": total_items,
            "transfer_rate": received_from_others / total_items if total_items > 0 else 0.0,
        }
        total_shared += received_from_others

    transfer_rates = [v["transfer_rate"] for v in transfer_counts.values()]
    avg_transfer = sum(transfer_rates) / len(transfer_rates) if transfer_rates else 0.0

    return {
        "learning_transfer": avg_transfer,
        "civ_transfer": transfer_counts,
        "total_cross_civ_items": total_shared,
    }


def compute_pod_cir(target_date: str) -> dict:
    """Compute PodCIR = avg(IndividualCIR) × C × S × L."""
    civ_msg_counts, total_cross_civ_msgs = count_cross_civ_messages(target_date)
    pod_size = len(POD_CIVS)
    coord_target = COORD_TARGET.get(pod_size, pod_size * 10)
    c_factor = min(1.0, total_cross_civ_msgs / coord_target)

    spec_result = compute_specialization()
    s_factor = spec_result["specialization"]

    lt_result = compute_learning_transfer()
    l_factor = lt_result["learning_transfer"]

    individual_cirs = {}
    for civ_name in POD_CIVS:
        if civ_name == "qwen":
            our_cir = compute_cir(target_date)
            individual_cirs[civ_name] = our_cir["cir"]
        else:
            individual_cirs[civ_name] = None

    valid_cirs = [v for v in individual_cirs.values() if v is not None]
    avg_cir = sum(valid_cirs) / len(valid_cirs) if valid_cirs else 0.0

    pod_cir = avg_cir * c_factor * s_factor * l_factor

    return {
        "pod_cir": pod_cir,
        "avg_individual_cir": avg_cir,
        "individual_cirs": individual_cirs,
        "coordination": {
            "C": c_factor,
            "cross_civ_messages": total_cross_civ_msgs,
            "coordination_target": coord_target,
            "per_civ": civ_msg_counts,
        },
        "specialization": spec_result,
        "learning_transfer": lt_result,
    }


def print_pod_report(pod: dict):
    """Print a human-readable Pod CIR report."""
    print()
    print("=" * 50)
    print("Pod CIR Report")
    print("=" * 50)
    print()

    pod_cir = pod["pod_cir"]
    avg_cir = pod["avg_individual_cir"]
    c = pod["coordination"]["C"]
    s = pod["specialization"]["specialization"]
    l = pod["learning_transfer"]["learning_transfer"]

    print(f"PodCIR = avg(CIR) × C × S × L")
    print(f"       = {avg_cir:.4f} × {c:.2f} × {s:.2f} × {l:.2f}")
    print(f"       = {pod_cir:.6f}")
    print()

    print("Factors:")
    print(f"  avg(Individual CIR): {avg_cir:.4f}")
    print(f"  C (Coordination):    {c:.2f} ({pod['coordination']['cross_civ_messages']} cross-civ messages)")
    print(f"  S (Specialization):  {s:.2f} ({pod['specialization']['total_unique_domains']} unique domains)")
    print(f"  L (Learning Transfer): {l:.2f} ({pod['learning_transfer']['total_cross_civ_items']} cross-civ items)")
    print()

    print("Individual CIVs:")
    for civ_name, cir in pod["individual_cirs"].items():
        if cir is not None:
            print(f"  {civ_name:8s} CIR = {cir:.4f}")
        else:
            print(f"  {civ_name:8s} CIR = (needs local compute_cir.py)")

    print()
    print("Coordination Detail:")
    for civ_name, count in pod["coordination"]["per_civ"].items():
        print(f"  {civ_name:8s}: {count} cross-civ mentions")

    print()
    print("Specialization Detail:")
    for civ_name, domains in pod["specialization"]["civ_domains"].items():
        print(f"  {civ_name:8s}: {', '.join(domains) if domains else '(none detected)'}")

    print()

    factors = {
        "avg(CIR)": avg_cir,
        "C (Coordination)": c,
        "S (Specialization)": s,
        "L (Learning Transfer)": l,
    }
    bottleneck = min(factors, key=factors.get)
    print(f"Pod Bottleneck: {bottleneck} ({factors[bottleneck]:.2f})")

    if bottleneck == "C (Coordination)":
        print("  Cross-civ communication is low. Increase messages in comms-hub.")
    elif bottleneck == "S (Specialization)":
        print("  Domain overlap is high or coverage is low. Define clearer boundaries.")
    elif bottleneck == "L (Learning Transfer)":
        print("  Civs are not adopting each other's rules/skills. Share corrections to Hub.")
    elif bottleneck == "avg(CIR)":
        print("  Individual civ CIRs are low. Fix internal health before pod coordination.")


def main():
    parser = argparse.ArgumentParser(description="Compute Compound Intelligence Rate (CIR)")
    parser.add_argument(
        "--date",
        default=datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        help="Date to compute CIR for (YYYY-MM-DD). Default: today.",
    )
    parser.add_argument(
        "--json",
        action="store_true",
        help="Output as JSON instead of human-readable report.",
    )
    parser.add_argument(
        "--pod",
        action="store_true",
        help="Also compute Pod CIR across multiple civs (A-C-Gee, Proof, Qwen).",
    )
    parser.add_argument(
        "--roots",
        nargs="*",
        help="Root directories to scan (default: auto-detect from script location).",
    )
    args = parser.parse_args()

    # Validate date
    try:
        datetime.strptime(args.date, "%Y-%m-%d")
    except ValueError:
        print(f"Error: Invalid date format '{args.date}'. Use YYYY-MM-DD.", file=sys.stderr)
        sys.exit(1)

    result = compute_cir(args.date)

    # If --pod flag, compute PodCIR and merge
    if args.pod:
        pod_result = compute_pod_cir(args.date)
        result["pod_cir"] = pod_result

    if args.json:
        print_json(result)
    else:
        print_report(result)
        if args.pod:
            print_pod_report(result["pod_cir"])


if __name__ == "__main__":
    main()
