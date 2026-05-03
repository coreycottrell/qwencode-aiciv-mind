#!/usr/bin/env python3
"""
Skill Self-Improver — Hermes #2

Class-first self-improvement loop. Complements Curator (which GRADES skills)
by IMPROVING them based on grade data. Applies a structured rubric to FAIL/WARN
skills and generates v_next improvement proposals.

Usage:
    python3 skill_self_improver.py improve [--log <grade-log>] [--output <vnext-dir>]
    python3 skill_self_improver.py rubric-score <skill-path>
    python3 skill_self_improver.py vnext-merge <skill-path> [--dry-run]
    python3 skill_self_improver.py suggest [--log <grade-log>] [--limit <n>]
    python3 skill_self_improver.py run [--skills-dir <path>] [--limit <n>]

Rubric dimensions (6):
    name_quality, description_clarity, examples_coverage,
    fc_completeness, test_coverage, co_use_readiness
"""

import argparse
import json
import sys
from collections import defaultdict
from dataclasses import dataclass, field
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional

# ───────────────────────────────────────────────────────────────────
# Types
# ───────────────────────────────────────────────────────────────────

@dataclass
class RubricScore:
    name_quality: int = 0       # 0-3: name is precise, skill-shaped
    description_clarity: int = 0 # 0-3: clear problem/solution statement
    examples_coverage: int = 0  # 0-3: has usage examples
    fc_completeness: int = 0    # 0-3: FC covers all sections
    test_coverage: int = 0      # 0-3: has smoke test or unit tests
    co_use_readiness: int = 0   # 0-3: explicitly mentions co-use patterns

    @property
    def total(self) -> int:
        return (self.name_quality + self.description_clarity +
                self.examples_coverage + self.fc_completeness +
                self.test_coverage + self.co_use_readiness)

    @property
    def max_total(self) -> int:
        return 18  # 6 dimensions × 3 max each

    @property
    def grade(self) -> str:
        pct = self.total / self.max_total
        if pct >= 0.8: return "A"
        if pct >= 0.6: return "B"
        if pct >= 0.4: return "C"
        if pct >= 0.2: return "D"
        return "F"

    def to_dict(self) -> dict:
        return {
            "name_quality": self.name_quality,
            "description_clarity": self.description_clarity,
            "examples_coverage": self.examples_coverage,
            "fc_completeness": self.fc_completeness,
            "test_coverage": self.test_coverage,
            "co_use_readiness": self.co_use_readiness,
            "total": self.total,
            "max": self.max_total,
            "grade": self.grade,
        }


@dataclass
class VNext:
    skill_name: str
    skill_path: str
    rubric_scores: RubricScore
    curator_grade: str  # PASS/FAIL/WARN from curator
    curator_reasons: list[str]
    frontmatter: Optional[dict]
    suggestions: list[str]
    vnext_path: str
    priority: str = "medium"  # high/medium/low

    def to_markdown(self) -> str:
        grade = self.rubric_scores.grade
        total = self.rubric_scores.total
        max_total = self.rubric_scores.max_total
        pct = total / max_total

        lines = [
            f"# Skill Improvement Proposal — {self.skill_name}",
            "",
            f"**Status**: v_next (proposed improvement)",
            f"**Skill path**: `{self.skill_path}`",
            f"**Curator grade**: {self.curator_grade} — {', '.join(self.curator_reasons)}",
            f"**Rubric score**: {grade} ({total}/{max_total} = {pct:.0%})",
            f"**Priority**: {self.priority}",
            f"**Generated**: {datetime.now(timezone.utc).isoformat()}",
            "",
            "## Rubric Breakdown",
            "",
            "| Dimension | Score | Max |",
            "|-----------|-------|-----|",
            f"| name_quality | {self.rubric_scores.name_quality} | 3 |",
            f"| description_clarity | {self.rubric_scores.description_clarity} | 3 |",
            f"| examples_coverage | {self.rubric_scores.examples_coverage} | 3 |",
            f"| fc_completeness | {self.rubric_scores.fc_completeness} | 3 |",
            f"| test_coverage | {self.rubric_scores.test_coverage} | 3 |",
            f"| co_use_readiness | {self.rubric_scores.co_use_readiness} | 3 |",
            "",
            "## Suggested Improvements",
            "",
        ]
        for i, s in enumerate(self.suggestions, 1):
            lines.append(f"{i}. {s}")

        if self.frontmatter:
            lines.extend(["", "## Existing Frontmatter (Parent Class)", "", "```yaml"])
            lines.append(f"name: {self.frontmatter.get('name', 'N/A')}")
            lines.append(f"description: {self.frontmatter.get('description', 'N/A')}")
            if self.frontmatter.get("version"):
                lines.append(f"version: {self.frontmatter['version']}")
            if self.frontmatter.get("applicable_civs"):
                lines.append(f"applicable_civs: {self.frontmatter['applicable_civs']}")
            lines.append("```")

        lines.extend(["", "## How to Apply", "",
                       f"1. Read `{self.skill_path}/SKILL.md`", "",
                       f"2. Merge suggestions into SKILL.md frontmatter + content", "",
                       f"3. Update or create `FIRING_CONTRACT.md` per suggestions above", "",
                       f"4. Run `python3 smoke_test.py` to verify", "",
                       "---",
                       f"*Generated by skill-self-improver v0.1.0 | Hermes #2*"])
        return "\n".join(lines)


@dataclass
class GradeRecord:
    skill: str
    skill_path: str
    grade: str
    reasons: list[str]
    frontmatter_ok: bool
    has_skill_md: bool
    has_fc: bool
    frontmatter: Optional[dict] = None
    civ: str = "unknown"
    ts: str = ""


# ───────────────────────────────────────────────────────────────────
# Rubric Scoring Logic
# ───────────────────────────────────────────────────────────────────

def score_skill(skill_path: Path, grade_record: Optional[GradeRecord] = None) -> RubricScore:
    """Apply 6-dimension rubric to a skill directory."""
    score = RubricScore()

    # ── name_quality: check SKILL.md name is precise, skill-shaped
    skill_md_path = skill_path / "SKILL.md"
    name_value = ""
    desc_value = ""

    if skill_md_path.exists():
        try:
            text = skill_md_path.read_text()
            frontmatter = parse_frontmatter(text)
            name_value = frontmatter.get("name", "")
            desc_value = frontmatter.get("description", "")

            # name_quality: skill-shaped means follows pattern "verb-noun" or "domain-concern"
            # e.g., "test-driven-development", "hub-watcher", "compute-hibernation-tracker"
            import re
            # Good: hyphenated compound, lowercase, 2+ segments
            if re.match(r'^[a-z][a-z0-9]+(-[a-z0-9]+){1,}$', name_value):
                score.name_quality = 3
            elif re.match(r'^[a-z]+(-[a-z0-9]+){1,}$', name_value):
                score.name_quality = 2
            elif name_value:
                score.name_quality = 1
            else:
                score.name_quality = 0

            # description_clarity: length + has "##" sections
            has_sections = "##" in text
            desc_len = len(desc_value)
            if desc_len > 100 and has_sections:
                score.description_clarity = 3
            elif desc_len > 50:
                score.description_clarity = 2
            elif desc_len > 10:
                score.description_clarity = 1
            else:
                score.description_clarity = 0

            # examples_coverage: count code blocks or "## Examples" sections
            code_blocks = text.count("```") // 2
            has_examples_section = re.search(r'^##\s*Example', text, re.MULTILINE)
            if code_blocks >= 3 or has_examples_section:
                score.examples_coverage = 3
            elif code_blocks >= 1:
                score.examples_coverage = 2
            elif desc_len > 0:
                score.examples_coverage = 1

        except Exception:
            pass

    # ── fc_completeness: check FIRING_CONTRACT.md sections
    fc_path = skill_path / "FIRING_CONTRACT.md"
    if fc_path.exists():
        try:
            fc_text = fc_path.read_text()
            sections = ["WHEN", "WHAT", "PRE", "POST", "FAILURE", "OBSERVABILITY"]
            present = sum(1 for s in sections if s in fc_text)
            score.fc_completeness = present  # 0-6... but max is 3, so:
            score.fc_completeness = min(3, present // 2 + 1)  # rough map to 0-3
        except Exception:
            score.fc_completeness = 0
    else:
        score.fc_completeness = 0

    # ── test_coverage: smoke_test.py or test_<skill>.py exists
    smoke = skill_path / "smoke_test.py"
    test_py = skill_path / f"test_{skill_path.name}.py"
    test_sh = skill_path / "test.sh"
    if smoke.exists() or test_py.exists() or test_sh.exists():
        score.test_coverage = 3
    else:
        score.test_coverage = 0

    # ── co_use_readiness: explicitly mentions co-use or skill references
    if skill_md_path.exists():
        try:
            text = skill_md_path.read_text().lower()
            co_use_indicators = ["co-use", "couskill", "complementary", "pairs with",
                                  "works with", "before:", "after:", "chain:"]
            score.co_use_readiness = 3 if any(i in text for i in co_use_indicators) else 0
        except Exception:
            score.co_use_readiness = 0

    return score


def parse_frontmatter(text: str) -> dict:
    """Parse YAML frontmatter from SKILL.md text."""
    import yaml
    match = text.match(r'^---\n(.*?)\n---', text, re.DOTALL) if False else None
    if not text.startswith("---"):
        return {}
    try:
        lines = text.split("\n")
        end = lines.index("---", 1)
        yaml_text = "\n".join(lines[1:end])
        return yaml.safe_load(yaml_text) or {}
    except Exception:
        return {}


def read_skills_from_log(log_path: Path) -> list[GradeRecord]:
    """Read grade records from Curator JSONL log."""
    if not log_path.exists():
        return []
    records = []
    with open(log_path) as f:
        for line in f:
            try:
                d = json.loads(line)
                records.append(GradeRecord(
                    skill=d.get("skill", ""),
                    skill_path=d.get("skill_path", ""),
                    grade=d.get("grade", "FAIL"),
                    reasons=d.get("reasons", []),
                    frontmatter_ok=d.get("frontmatter_ok", False),
                    has_skill_md=d.get("has_skill_md", False),
                    has_fc=d.get("has_fc", False),
                    frontmatter=d.get("frontmatter"),
                    civ=d.get("civ", "unknown"),
                    ts=d.get("ts", ""),
                ))
            except (json.JSONDecodeError, KeyError):
                continue
    return records


def suggest_improvements(record: GradeRecord, rubric: RubricScore, skill_path: Path = None) -> list[str]:
    """Generate specific, actionable suggestions based on rubric gaps."""
    suggestions = []
    scores = rubric
    fallback_name = skill_path.name if skill_path else record.skill

    if scores.name_quality < 2:
        name_val = record.frontmatter.get("name", fallback_name) if record.frontmatter else fallback_name
        suggestions.append(
            f"Rename skill to follow 'verb-noun' or 'domain-concern' pattern "
            f"(current: '{name_val}' — "
            "use lowercase, hyphenated compounds like 'test-driven-development')"
        )
    if scores.description_clarity < 2:
        suggestions.append(
            "Expand description: state the PROBLEM this skill solves and the SOLUTION it provides. "
            "Include expected input/output. Target 50+ characters with at least one '##' section."
        )
    if scores.examples_coverage < 2:
        suggestions.append(
            "Add usage examples: at least one '## Examples' section with code blocks showing "
            "input → output. Aim for 2+ concrete examples."
        )
    if scores.fc_completeness < 2:
        suggestions.append(
            "Improve FIRING_CONTRACT.md completeness: ensure all 6 sections present "
            "(WHEN, WHAT, PRE, POST, FAILURE, OBSERVABILITY). "
            + ("Create FIRING_CONTRACT.md stub." if not record.has_fc else "")
        )
    if scores.test_coverage < 2:
        suggestions.append(
            "Add test coverage: create smoke_test.py that verifies the skill's golden path. "
            "Test should exit 0 on success, non-zero on failure."
        )
    if scores.co_use_readiness < 2:
        suggestions.append(
            "Add co-use section to SKILL.md: document which skills this one pairs with, "
            "preconditions (skills that should run before), and post-conditions (skills that follow). "
            "Example: '## Co-use: run skill-evolution-tracker after this to track usage.'"
        )

    if not suggestions:
        suggestions.append(
            "Skill is well-formed. Consider running skill-evolution-tracker to log invocations "
            "and track usage patterns over time."
        )

    return suggestions


def determine_priority(record: GradeRecord, rubric: RubricScore) -> str:
    """Determine improvement priority based on curator grade + rubric score."""
    if record.grade == "FAIL" and rubric.total < rubric.max_total * 0.3:
        return "high"
    if record.grade == "WARN" or rubric.total < rubric.max_total * 0.5:
        return "medium"
    return "low"


# ───────────────────────────────────────────────────────────────────
# CLI Commands
# ───────────────────────────────────────────────────────────────────

def cmd_improve(args):
    """Run improvement on all FAIL/WARN skills from curator log."""
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skill-curator-grades-ACG-v02.jsonl")
    output_dir = Path(args.output).expanduser() if args.output else Path("memories/vnext")

    if not log_path.exists():
        print(f"Grade log not found: {log_path}")
        print("Run 'python3 skill_curator.py grade' first to generate grades.")
        sys.exit(1)

    records = read_skills_from_log(log_path)
    fail_warn = [r for r in records if r.grade in ("FAIL", "WARN")]

    if args.limit:
        fail_warn = fail_warn[:args.limit]

    print(f"SKILL SELF-IMPROVER — Improve Run")
    print(f"  Grade log: {log_path}")
    print(f"  Total records: {len(records)}")
    print(f"  FAIL/WARN: {len(fail_warn)}")
    if args.limit:
        print(f"  Limit: {args.limit}")
    print("")

    output_dir.mkdir(parents=True, exist_ok=True)
    improvements = []

    for record in fail_warn:
        skill_path = Path(record.skill_path)
        if not skill_path.exists():
            continue

        rubric = score_skill(skill_path, record)
        suggestions = suggest_improvements(record, rubric, skill_path)
        priority = determine_priority(record, rubric)

        vnext_path = output_dir / f"{record.skill}.vnext.md"
        vnext = VNext(
            skill_name=record.skill,
            skill_path=record.skill_path,
            rubric_scores=rubric,
            curator_grade=record.grade,
            curator_reasons=record.reasons,
            frontmatter=record.frontmatter,
            suggestions=suggestions,
            vnext_path=str(vnext_path),
            priority=priority,
        )

        vnext_path.parent.mkdir(parents=True, exist_ok=True)
        vnext_path.write_text(vnext.to_markdown())
        improvements.append(vnext)

        print(f"  [{vnext.priority.upper():>6}] {record.skill}: {rubric.grade} ({rubric.total}/{rubric.max_total})")

    print(f"\n  Wrote {len(improvements)} v_next proposals to {output_dir}/")
    return improvements


def cmd_rubric_score(args):
    """Score a single skill against the rubric."""
    skill_path = Path(args.skill_path)
    if not skill_path.exists():
        print(f"Skill path not found: {skill_path}")
        sys.exit(1)

    rubric = score_skill(skill_path)
    scores = rubric.to_dict()

    print(f"SKILL SELF-IMPROVER — Rubric Score")
    print(f"  Skill: {skill_path.name}")
    print(f"  Path: {skill_path}")
    print("")
    print("  Dimension             Score   Max")
    print("  --------------------------------")
    for dim in ["name_quality", "description_clarity", "examples_coverage",
                "fc_completeness", "test_coverage", "co_use_readiness"]:
        val = scores[dim]
        print(f"  {dim:<22} {val:>5}   3")
    print("  --------------------------------")
    print(f"  {'TOTAL':<22} {scores['total']:>5}   {scores['max']}")
    print(f"  Grade: {scores['grade']}")
    print("")

    if args.verbose:
        record_data = {
            "skill": skill_path.name,
            "skill_path": str(skill_path),
            "grade": "unknown",
            "reasons": [],
            "frontmatter_ok": False,
            "has_skill_md": (skill_path / "SKILL.md").exists(),
            "has_fc": (skill_path / "FIRING_CONTRACT.md").exists(),
        }
        record = GradeRecord(**record_data)
        suggestions = suggest_improvements(record, rubric)
        if suggestions:
            print("  Suggestions:")
            for s in suggestions:
                print(f"    - {s}")


def cmd_vnext_merge(args):
    """Merge v_next into actual skill SKILL.md (dry-run by default)."""
    vnext_path = Path(args.vnext_path)
    if not vnext_path.exists():
        print(f"v_next not found: {vnext_path}")
        sys.exit(1)

    vnext_text = vnext_path.read_text()
    print(f"v_next content:\n{vnext_text[:500]}...")
    print(f"\n Dry-run complete. Run with --confirm to actually apply changes.")


def cmd_suggest(args):
    """List top improvement suggestions from grade log."""
    log_path = Path(args.log).expanduser() if args.log else Path("memories/skill-curator-grades-ACG-v02.jsonl")
    if not log_path.exists():
        print(f"Grade log not found: {log_path}")
        sys.exit(1)

    records = read_skills_from_log(log_path)
    fail_warn = sorted(
        [r for r in records if r.grade in ("FAIL", "WARN")],
        key=lambda r: r.ts or "",
        reverse=True
    )

    if args.limit:
        fail_warn = fail_warn[:args.limit]

    print(f"SKILL SELF-IMPROVER — Top Suggestions")
    print(f"  Grade log: {log_path}")
    print(f"  FAIL/WARN count: {len(fail_warn)}")
    if args.limit:
        print(f"  Showing: {args.limit}")
    print("")
    print("  Pri  Skill                  Curator  Rubric  Top Suggestion")
    print("  -----------------------------------------------------------")

    for record in fail_warn:
        skill_path = Path(record.skill_path)
        if not skill_path.exists():
            continue

        rubric = score_skill(skill_path, record)
        suggestions = suggest_improvements(record, rubric, skill_path)
        priority = determine_priority(record, rubric)
        top_suggestion = suggestions[0][:60] + ("..." if len(suggestions[0]) > 60 else "")

        print(f"  {priority.upper():>4}  {record.skill:<23} {record.grade:<8} {rubric.grade} ({rubric.total}/{rubric.max_total})  {top_suggestion}")


def cmd_run(args):
    """Run full improvement cycle on a skills directory."""
    skills_dir = Path(args.skills_dir).expanduser() if args.skills_dir else None

    # Find skills directories to scan
    candidates = []
    if skills_dir and skills_dir.exists():
        candidates.append(skills_dir)
    else:
        # Look for standard skill directories
        for name in ["autonomy/skills", "skills", "../ACG/autonomy/skills"]:
            p = Path(name)
            if p.exists():
                candidates.append(p)

    if not candidates:
        print("No skills directories found. Specify --skills-dir explicitly.")
        sys.exit(1)

    print(f"SKILL SELF-IMPROVER — Full Improvement Run")
    for candidate in candidates:
        print(f"  Scanning: {candidate}")
    print("")

    all_improvements = []
    for skills_dir in candidates:
        for skill_dir in skills_dir.iterdir():
            if not skill_dir.is_dir():
                continue
            skill_md = skill_dir / "SKILL.md"
            if not skill_md.exists():
                continue

            rubric = score_skill(skill_dir)
            if rubric.total >= rubric.max_total:
                continue  # Skip already-good skills

            grade_record = GradeRecord(
                skill=skill_dir.name,
                skill_path=str(skill_dir),
                grade="FAIL" if rubric.total < rubric.max_total * 0.4 else "WARN",
                reasons=["rubric_score_below_threshold"],
                frontmatter_ok=True,
                has_skill_md=True,
                has_fc=(skill_dir / "FIRING_CONTRACT.md").exists(),
            )
            suggestions = suggest_improvements(grade_record, rubric)

            vnext_dir = Path("memories/vnext")
            vnext_dir.mkdir(parents=True, exist_ok=True)
            vnext_path = vnext_dir / f"{skill_dir.name}.vnext.md"

            vnext = VNext(
                skill_name=skill_dir.name,
                skill_path=str(skill_dir),
                rubric_scores=rubric,
                curator_grade=grade_record.grade,
                curator_reasons=grade_record.reasons,
                frontmatter=None,
                suggestions=suggestions,
                vnext_path=str(vnext_path),
                priority=determine_priority(grade_record, rubric),
            )
            vnext_path.write_text(vnext.to_markdown())
            all_improvements.append(vnext)

    print(f"  Scanned {sum(1 for c in candidates for _ in c.iterdir() if _.is_dir())} skill directories")
    print(f"  Generated {len(all_improvements)} v_next proposals")
    high = [v for v in all_improvements if v.priority == "high"]
    print(f"  High priority: {len(high)}")

    if high:
        print("\n  HIGH PRIORITY:")
        for v in high:
            print(f"    - {v.skill_name}: {v.suggestions[0][:70]}")


def main():
    parser = argparse.ArgumentParser(description="Skill Self-Improver — Hermes #2")
    sub = parser.add_subparsers()

    p_improve = sub.add_parser("improve", help="Run improvement on FAIL/WARN skills from curator log")
    p_improve.add_argument("--log", default="memories/skill-curator-grades-ACG-v02.jsonl", help="Curator grade log path")
    p_improve.add_argument("--output", default="memories/vnext", help="Output directory for v_next files")
    p_improve.add_argument("--limit", type=int, default=0, help="Limit number of skills to process")
    p_improve.set_defaults(func=cmd_improve)

    p_score = sub.add_parser("rubric-score", help="Score a single skill against rubric")
    p_score.add_argument("skill_path", help="Path to skill directory")
    p_score.add_argument("--verbose", action="store_true", help="Show suggestions")
    p_score.set_defaults(func=cmd_rubric_score)

    p_merge = sub.add_parser("vnext-merge", help="Merge v_next into actual skill (dry-run)")
    p_merge.add_argument("vnext_path", help="Path to v_next.md file")
    p_merge.add_argument("--confirm", action="store_true", help="Actually apply changes")
    p_merge.set_defaults(func=cmd_vnext_merge)

    p_suggest = sub.add_parser("suggest", help="List top improvement suggestions")
    p_suggest.add_argument("--log", default="memories/skill-curator-grades-ACG-v02.jsonl", help="Curator grade log path")
    p_suggest.add_argument("--limit", type=int, default=10, help="Limit results")
    p_suggest.set_defaults(func=cmd_suggest)

    p_run = sub.add_parser("run", help="Run full improvement on skills directory")
    p_run.add_argument("--skills-dir", help="Skills directory to scan")
    p_run.set_defaults(func=cmd_run)

    args = parser.parse_args()
    if hasattr(args, "func"):
        args.func(args)
    else:
        parser.print_help()


if __name__ == "__main__":
    main()
