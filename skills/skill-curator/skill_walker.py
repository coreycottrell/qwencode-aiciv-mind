#!/usr/bin/env python3
"""
Shared skill discovery walker — single source of truth for find_skills().
Handles all edge cases:
- Normal: skill_dir/SKILL.md is a file
- SKILL.md-dir: skill_dir/SKILL.md/ is a dir containing the real SKILL.md file
- Special containers (custom/, flows/, wake-up-modes/, autonomy/): recurse FIRST, stop (index, not skill)
- Aggregate SKILL.md at skills-root: skip it (not a skill, it's the index)
- Non-skill dirs: skip
"""

from pathlib import Path

SKILL_MD = "SKILL.md"
SPECIAL_CONTAINERS = {"custom", "flows", "wake-up-modes", "autonomy"}


def find_skills(root: Path) -> list[Path]:
    """Find all skill directories under root.

    Key behaviors:
    - SKILL.md as FILE in a directory = skill (add the directory)
    - SKILL.md as DIR containing inner SKILL.md file = SKILL.md-dir (add the parent directory)
    - Special containers: recurse into children, do NOT add the container itself
    - Root's own SKILL.md (aggregate index): skip at root level
    """
    skills = []

    def recurse(path: Path, is_root: bool = False):
        if not path.is_dir():
            return

        # At root level: determine if root itself is a skill or just an index
        # root is a skill if: root/SKILL.md exists AND there are child dirs that are also skills
        # (root's SKILL.md being an index implies it won't have child skill dirs at same level)
        if is_root:
            skill_md = path / SKILL_MD
            if skill_md.is_file():
                # Check if any child dirs are themselves skills
                child_skills = [c for c in sorted(path.iterdir())
                               if c.is_dir() and (c / SKILL_MD).exists()]
                if not child_skills:
                    # No child skill dirs — root's SKILL.md is the skill itself
                    skills.append(path)
                # else: children exist — root's SKILL.md is an aggregate index, skip it
            for child in sorted(path.iterdir()):
                if child.is_dir():
                    recurse(child, is_root=False)
            return

        # Special container: recurse into children, don't add the container itself
        if path.name in SPECIAL_CONTAINERS:
            for child in sorted(path.iterdir()):
                if child.is_dir():
                    recurse(child, is_root=False)
            return

        skill_md = path / SKILL_MD

        # SKILL.md is a directory — SKILL.md-dir edge case
        if skill_md.exists() and skill_md.is_dir():
            inner = skill_md / SKILL_MD
            if inner.exists() and inner.is_file():
                skills.append(path)
            return

        # SKILL.md is a file — normal skill
        if skill_md.is_file():
            skills.append(path)
            return

    recurse(root, is_root=True)
    return sorted(skills, key=lambda p: p.name)
