#!/usr/bin/env python3
"""
Proof Self-Bug-Finder — Self-Analysis Engine

Scans Proof's own code to find:
- Hardcoded paths (proof-aiciv, /home/corey, etc.)
- Import errors
- Security issues
- Hook misconfigurations
- Dead code

Inspired by Hengshi's dream-bug-finder — self-awareness through tooling.

Usage:
    python3 self_bug_finder.py scan          # Full analysis
    python3 self_bug_finder.py check-paths   # Path contamination only
    python3 self_bug_finder.py check-imports  # Import errors only
    python3 self_bug_finder.py check-security # Security issues
    python3 self_bug_finder.py check-hooks   # Hook configuration
    python3 self_bug_finder.py check-skills  # Skills consistency
    python3 self_bug_finder.py self-check    # Verify this file is clean
    python3 self_bug_finder.py scan --format=json  # Machine-readable output
"""

import json
import os
import re
import subprocess
import sys
from datetime import datetime
from pathlib import Path
from typing import Optional

# ============================================================================
# PROOF-SPECIFIC CONFIGURATION
# ============================================================================

CIV_ROOT = Path(os.environ.get("CIV_ROOT", "/home/corey/projects/AI-CIV/proof-aiciv"))
SKILL_DIR = Path(__file__).parent.resolve()

# Patterns that indicate path contamination (Proof-specific + generic)
PATH_CONTAMINATION_PATTERNS = [
    # Proof-specific paths
    (r"/home/corey/projects/AI-CIV/proof-aiciv", "proof-aiciv root path"),
    (r"proof-aiciv", "proof-aiciv civilization name"),
    (r"PROOF_ROOT", "PROOF_ROOT constant (should be CIV_ROOT)"),
    (r"proof_hub", "proof_hub module (should be hub)"),
    # Generic problematic patterns
    (r"5\.161\.90\.32", "hardcoded IP address (old HUB)"),
    (r"87\.99\.131\.49", "hardcoded IP address (HUB VPS)"),
    # Legacy path patterns
    (r"/home/civ", "generic /home/civ path"),
]

# Import-related patterns
IMPORT_PATTERNS = [
    # Wrong relative imports
    (r"from proof_hub\.", "proof_hub import (should be hub)"),
    (r"import proof_hub", "proof_hub import (should be hub)"),
    # Missing dotenv
    (r"os\.getenv.*?(?<!from dotenv)", "os.getenv without dotenv import check"),
]

# Security issue patterns
SECURITY_PATTERNS = [
    # Actual secrets (not placeholders)
    (r'api_key\s*=\s*["\']sk-(?!REPLACE|PLACEHOLDER)', "hardcoded API key"),
    (r'password\s*=\s*["\'][^(REPLACE|PLACEHOLDER|your_)]', "hardcoded password"),
    (r'Authorization.*?Bearer\s+sk-', "Bearer token in code"),
    (r'bot_token\s*=\s*["\'][^(REPLACE|PLACEHOLDER|your_)]', "hardcoded bot token"),
]

# Files/directories to ignore during scanning
IGNORE_PATTERNS = [
    ".git",
    "node_modules",
    "__pycache__",
    ".venv",
    "venv",
    ".pytest_cache",
    "*.pyc",
    ".claude/memory",  # Memory files change constantly
    "exports",  # Generated exports
    "to-parent",  # Handoff docs
    "to-corey",  # Handoff docs
]

# Hook-specific checks
HOOK_CHECKS = [
    ("settings.json", "context_monitor", "PreToolUse priority"),
    ("settings.json", "ceo_mode_enforcer", "hook timeout >= 5s"),
]

# ============================================================================
# SCANNER CLASSES
# ============================================================================

class BugReport:
    """Represents a single bug found."""

    def __init__(
        self,
        severity: str,  # P0, P1, P2, P3
        category: str,
        file: str,
        line: Optional[int],
        description: str,
        suggestion: str = "",
    ):
        self.severity = severity
        self.category = category
        self.file = file
        self.line = line
        self.description = description
        self.suggestion = suggestion

    def to_dict(self) -> dict:
        return {
            "severity": self.severity,
            "category": self.category,
            "file": self.file,
            "line": self.line,
            "description": self.description,
            "suggestion": self.suggestion,
        }

    def __str__(self) -> str:
        location = f"{self.file}:{self.line}" if self.line else self.file
        sev_marker = f"[{self.severity}]"
        suggestion = f"\n  → {self.suggestion}" if self.suggestion else ""
        return f"{sev_marker} {location} — {self.description}{suggestion}"


class SelfBugFinder:
    """Scans Proof's code for bugs."""

    def __init__(self, root: Path = CIV_ROOT):
        self.root = root
        self.issues: list[BugReport] = []

    # --------------------------------------------------------------------------
    # Path Contamination Check
    # --------------------------------------------------------------------------

    def check_paths(self) -> list[BugReport]:
        """Find hardcoded paths, Proof-specific names, IP addresses."""
        issues = []
        files_scanned = 0

        for file_path in self._iter_files():
            files_scanned += 1
            try:
                content = file_path.read_text(errors="ignore")
                lines = content.split("\n")

                for i, line in enumerate(lines, 1):
                    # Skip comments for some patterns
                    stripped = line.strip()

                    for pattern, description in PATH_CONTAMINATION_PATTERNS:
                        if re.search(pattern, line, re.IGNORECASE):
                            # Skip if it's in a comment explaining the pattern
                            if stripped.startswith("#") and "pattern" in stripped.lower():
                                continue
                            # Skip if it's in this very file (self-reference is OK in comments)
                            if "self_bug_finder" in str(file_path):
                                continue

                            issues.append(BugReport(
                                severity="P0",
                                category="path_contamination",
                                file=str(file_path.relative_to(self.root)),
                                line=i,
                                description=f"Found: {description}",
                                suggestion=f"Use CIV_ROOT env var or relative path instead of hardcoded value",
                            ))
            except Exception:
                pass

        return issues

    # --------------------------------------------------------------------------
    # Import Check
    # --------------------------------------------------------------------------

    def check_imports(self) -> list[BugReport]:
        """Check for import errors and wrong module paths."""
        issues = []
        skill_dir = self.root / ".claude" / "skills"

        # Check skills for wrong import patterns
        if skill_dir.exists():
            for skill_py in skill_dir.rglob("*.py"):
                if "self_bug_finder" in str(skill_py):
                    continue  # Skip self

                try:
                    content = skill_py.read_text(errors="ignore")

                    for pattern, description in IMPORT_PATTERNS:
                        matches = re.finditer(pattern, content, re.MULTILINE)
                        for match in matches:
                            line_num = content[:match.start()].count("\n") + 1
                            issues.append(BugReport(
                                severity="P1",
                                category="import_error",
                                file=str(skill_py.relative_to(self.root)),
                                line=line_num,
                                description=description,
                                suggestion="Use relative imports from hub instead of proof_hub",
                            ))
                except Exception:
                    pass

        # Check tools directory
        tools_dir = self.root / "tools"
        if tools_dir.exists():
            for tool_py in tools_dir.rglob("*.py"):
                try:
                    content = tool_py.read_text(errors="ignore")

                    # Check for missing dotenv when using os.environ
                    if "os.environ" in content or "os.getenv" in content:
                        if "from dotenv import" not in content and 'load_dotenv' not in content:
                            # Check if there's a find_dotenv usage
                            if "find_dotenv" not in content:
                                issues.append(BugReport(
                                    severity="P1",
                                    category="import_error",
                                    file=str(tool_py.relative_to(self.root)),
                                    line=0,
                                    description="Uses os.getenv/environ but doesn't import dotenv",
                                    suggestion="Add: from dotenv import load_dotenv, find_dotenv",
                                ))

                    for pattern, description in IMPORT_PATTERNS:
                        matches = re.finditer(pattern, content, re.MULTILINE)
                        for match in matches:
                            line_num = content[:match.start()].count("\n") + 1
                            issues.append(BugReport(
                                severity="P1",
                                category="import_error",
                                file=str(tool_py.relative_to(self.root)),
                                line=line_num,
                                description=description,
                                suggestion="Fix import path",
                            ))
                except Exception:
                    pass

        return issues

    # --------------------------------------------------------------------------
    # Security Check
    # --------------------------------------------------------------------------

    def check_security(self) -> list[BugReport]:
        """Find hardcoded secrets, API keys, passwords."""
        issues = []
        files_scanned = 0

        for file_path in self._iter_files():
            files_scanned += 1

            # Skip certain file types
            if file_path.suffix in [".md", ".json", ".yaml", ".yml", ".txt"]:
                if "config" not in str(file_path).lower():
                    continue

            # Skip self
            if "self_bug_finder" in str(file_path):
                continue

            try:
                content = file_path.read_text(errors="ignore")
                lines = content.split("\n")

                for i, line in enumerate(lines, 1):
                    for pattern, description in SECURITY_PATTERNS:
                        match = re.search(pattern, line)
                        if match:
                            # Skip placeholder patterns
                            if any(p in line.lower() for p in ["replace", "placeholder", "your_", "example"]):
                                continue

                            issues.append(BugReport(
                                severity="P0",
                                category="security",
                                file=str(file_path.relative_to(self.root)),
                                line=i,
                                description=description,
                                suggestion="Move secret to environment variable or .env file",
                            ))
            except Exception:
                pass

        return issues

    # --------------------------------------------------------------------------
    # Hook Check
    # --------------------------------------------------------------------------

    def check_hooks(self) -> list[BugReport]:
        """Check hook configuration for issues."""
        issues = []
        settings_file = self.root / ".claude" / "settings.json"

        if not settings_file.exists():
            issues.append(BugReport(
                severity="P1",
                category="hook_config",
                file=str(settings_file.relative_to(self.root)),
                line=0,
                description="settings.json not found",
                suggestion="Create settings.json with hook configuration",
            ))
            return issues

        try:
            content = settings_file.read_text()
            data = json.loads(content)

            # Check PreToolUse hooks
            pre_hooks = data.get("hooks", {}).get("PreToolUse", [])

            if not pre_hooks:
                issues.append(BugReport(
                    severity="P1",
                    category="hook_config",
                    file=".claude/settings.json",
                    line=0,
                    description="No PreToolUse hooks configured",
                    suggestion="Add context_monitor.py as first PreToolUse hook",
                ))
                return issues

            # Check order: context_monitor should come before ceo_mode_enforcer
            hook_names = [h.get("command", "").split("/")[-1].replace(".py", "") for h in pre_hooks]

            if "context_monitor" in hook_names and "ceo_mode_enforcer" in hook_names:
                ctx_idx = hook_names.index("context_monitor")
                ceo_idx = hook_names.index("ceo_mode_enforcer")

                if ceo_idx < ctx_idx:
                    issues.append(BugReport(
                        severity="P1",
                        category="hook_config",
                        file=".claude/settings.json",
                        line=0,
                        description="ceo_mode_enforcer runs before context_monitor (wrong order)",
                        suggestion="context_monitor must run FIRST (high priority) to prevent context death spiral",
                    ))

            # Check timeout values
            for hook in pre_hooks:
                cmd = hook.get("command", "")
                timeout = hook.get("timeout", 0)

                if "context_monitor" in cmd and timeout < 5:
                    issues.append(BugReport(
                        severity="P2",
                        category="hook_config",
                        file=".claude/settings.json",
                        line=0,
                        description="context_monitor timeout too short (< 5s)",
                        suggestion="Set timeout >= 5 to prevent premature blocking",
                    ))

        except json.JSONDecodeError as e:
            issues.append(BugReport(
                severity="P0",
                category="hook_config",
                file=".claude/settings.json",
                line=0,
                description=f"Invalid JSON in settings.json: {e}",
                suggestion="Fix JSON syntax in settings.json",
            ))
        except Exception as e:
            issues.append(BugReport(
                severity="P2",
                category="hook_config",
                file=".claude/settings.json",
                line=0,
                description=f"Error checking hooks: {e}",
                suggestion="Manually verify hook configuration",
            ))

        return issues

    # --------------------------------------------------------------------------
    # Skills Check
    # --------------------------------------------------------------------------

    def check_skills(self) -> list[BugReport]:
        """Check skills directory for consistency issues."""
        issues = []
        skills_dir = self.root / ".claude" / "skills"

        if not skills_dir.exists():
            issues.append(BugReport(
                severity="P2",
                category="skills",
                file=".claude/skills/",
                line=0,
                description="Skills directory not found",
                suggestion="Skills directory should exist at .claude/skills/",
            ))
            return issues

        # Check each skill has a SKILL.md
        for skill_path in skills_dir.iterdir():
            if not skill_path.is_dir():
                continue

            skill_md = skill_path / "SKILL.md"
            if not skill_md.exists():
                issues.append(BugReport(
                    severity="P3",
                    category="skills",
                    file=str(skill_path.relative_to(self.root)),
                    line=0,
                    description=f"SKILL.md missing in {skill_path.name}/",
                    suggestion="Add SKILL.md to document skill purpose and usage",
                ))

            # Check for old/wrong paths in skill markdown
            if skill_md.exists():
                try:
                    content = skill_md.read_text()
                    for pattern, description in PATH_CONTAMINATION_PATTERNS:
                        if re.search(pattern, content, re.IGNORECASE):
                            issues.append(BugReport(
                                severity="P1",
                                category="skills",
                                file=str(skill_md.relative_to(self.root)),
                                line=0,
                                description=f"Hardcoded path in skill documentation: {description}",
                                suggestion="Use generic paths or CIV_ROOT reference in skill docs",
                            ))
                except Exception:
                    pass

        return issues

    # --------------------------------------------------------------------------
    # Self-Check
    # --------------------------------------------------------------------------

    def self_check(self) -> list[BugReport]:
        """Verify this very file is clean."""
        issues = []
        this_file = Path(__file__)

        try:
            content = this_file.read_text()

            # Check for hardcoded paths in self
            for pattern, description in PATH_CONTAMINATION_PATTERNS:
                if re.search(pattern, content, re.IGNORECASE):
                    # Self-references are OK
                    if "self_bug_finder" in content[max(0, content.find(pattern) - 100):content.find(pattern) + 100]:
                        continue
                    issues.append(BugReport(
                        severity="P0",
                        category="self_contamination",
                        file="self_bug_finder.py",
                        line=0,
                        description=f"Self bug finder has path contamination: {description}",
                        suggestion="Remove hardcoded Proof paths from self_bug_finder.py",
                    ))

            # Check imports resolve
            import_errors = self._test_imports()
            if import_errors:
                for err in import_errors:
                    issues.append(BugReport(
                        severity="P1",
                        category="import_error",
                        file="self_bug_finder.py",
                        line=0,
                        description=f"Import error in self: {err}",
                        suggestion="Fix import in self_bug_finder.py",
                    ))

        except Exception as e:
            issues.append(BugReport(
                severity="P1",
                category="self_check_failed",
                file="self_bug_finder.py",
                line=0,
                description=f"Self-check failed: {e}",
                suggestion="Manually verify self_bug_finder.py",
            ))

        return issues

    # --------------------------------------------------------------------------
    # Full Scan
    # --------------------------------------------------------------------------

    def scan(self) -> dict:
        """Run all checks."""
        all_issues = []

        print("Scanning for path contamination...", file=sys.stderr)
        all_issues.extend(self.check_paths())

        print("Scanning for import errors...", file=sys.stderr)
        all_issues.extend(self.check_imports())

        print("Scanning for security issues...", file=sys.stderr)
        all_issues.extend(self.check_security())

        print("Checking hook configuration...", file=sys.stderr)
        all_issues.extend(self.check_hooks())

        print("Checking skills directory...", file=sys.stderr)
        all_issues.extend(self.check_skills())

        print("Running self-check...", file=sys.stderr)
        all_issues.extend(self.self_check())

        # Sort by severity then category
        severity_order = {"P0": 0, "P1": 1, "P2": 2, "P3": 3}
        all_issues.sort(key=lambda x: (severity_order.get(x.severity, 4), x.category))

        self.issues = all_issues
        return self._format_results(all_issues)

    # --------------------------------------------------------------------------
    # Helpers
    # --------------------------------------------------------------------------

    def _iter_files(self):
        """Iterate over files in CIV_ROOT, respecting ignore patterns."""
        dirs_to_skip = {IGNORED for IGNORED in IGNORE_PATTERNS}

        for root, dirs, files in os.walk(self.root):
            # Filter directories
            dirs[:] = [d for d in dirs if d not in dirs_to_skip and not d.startswith(".")]

            for file in files:
                if file.startswith("."):
                    continue
                file_path = Path(root) / file
                if file_path.suffix in [".py", ".json", ".yaml", ".sh"]:
                    yield file_path

    def _test_imports(self) -> list[str]:
        """Test that key imports work."""
        errors = []

        # Change to CIV_ROOT to test imports
        original_cwd = os.getcwd()
        os.chdir(self.root)

        try:
            # Test dotenv
            try:
                from dotenv import load_dotenv, find_dotenv
            except ImportError as e:
                errors.append(f"dotenv import failed: {e}")

            # Test pathlib
            try:
                from pathlib import Path
            except ImportError as e:
                errors.append(f"pathlib import failed: {e}")

        finally:
            os.chdir(original_cwd)

        return errors

    def _format_results(self, issues: list[BugReport]) -> dict:
        """Format results into output dict."""
        by_severity = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for issue in issues:
            by_severity[issue.severity] = by_severity.get(issue.severity, 0) + 1

        return {
            "timestamp": datetime.now().isoformat(),
            "civ_root": str(self.root),
            "total_issues": len(issues),
            "by_severity": by_severity,
            "issues": [i.to_dict() for i in issues],
        }


# ============================================================================
# CLI
# ============================================================================

def print_human_readable(results: dict):
    """Print results in human-readable format."""
    issues = results["issues"]
    by_sev = results["by_severity"]

    print()
    print("=" * 60)
    print("PROOF SELF-BUG-FINDER REPORT")
    print("=" * 60)
    print(f"Generated: {results['timestamp']}")
    print(f"Root: {results['civ_root']}")
    print()
    print(f"Total issues: {results['total_issues']}")
    print(f"  [P0] Critical: {by_sev.get('P0', 0)}")
    print(f"  [P1] High:     {by_sev.get('P1', 0)}")
    print(f"  [P2] Medium:   {by_sev.get('P2', 0)}")
    print(f"  [P3] Low:     {by_sev.get('P3', 0)}")
    print()

    if not issues:
        print("No issues found. Your code is clean.")
        return

    # Group by severity
    by_category = {}
    for issue in issues:
        cat = issue["category"]
        if cat not in by_category:
            by_category[cat] = []
        by_category[cat].append(issue)

    for severity in ["P0", "P1", "P2", "P3"]:
        cat_issues = [(cat, issues) for cat, issues in by_category.items()
                      if any(i["severity"] == severity for i in issues)]
        if not cat_issues:
            continue

        print(f"[{severity}] {severity.upper()} PRIORITY ISSUES")
        print("-" * 40)

        for cat, cat_items in cat_issues:
            for issue in cat_items:
                location = f"{issue['file']}:{issue['line']}" if issue['line'] else issue['file']
                print(f"  {location}")
                print(f"    {issue['description']}")
                if issue['suggestion']:
                    print(f"    → {issue['suggestion']}")
        print()


def main():
    import argparse

    parser = argparse.ArgumentParser(description="Proof Self-Bug-Finder")
    parser.add_argument("command", choices=["scan", "check-paths", "check-imports",
                                            "check-security", "check-hooks",
                                            "check-skills", "self-check"],
                        help="What to run")
    parser.add_argument("--format", choices=["human", "json"], default="human",
                        help="Output format")

    args = parser.parse_args()

    finder = SelfBugFinder()

    if args.command == "scan":
        results = finder.scan()
    elif args.command == "check-paths":
        issues = finder.check_paths()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}
    elif args.command == "check-imports":
        issues = finder.check_imports()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}
    elif args.command == "check-security":
        issues = finder.check_security()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}
    elif args.command == "check-hooks":
        issues = finder.check_hooks()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}
    elif args.command == "check-skills":
        issues = finder.check_skills()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}
    elif args.command == "self-check":
        issues = finder.self_check()
        by_sev = {"P0": 0, "P1": 0, "P2": 0, "P3": 0}
        for i in issues:
            by_sev[i.severity] = by_sev.get(i.severity, 0) + 1
        results = {"issues": [i.to_dict() for i in issues], "total_issues": len(issues), "by_severity": by_sev}

    if args.format == "json":
        print(json.dumps(results, indent=2))
    else:
        print_human_readable(results)


if __name__ == "__main__":
    main()
