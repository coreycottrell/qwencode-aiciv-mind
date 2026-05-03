# TDD Skill — Firing Contract

**Skill**: `skills/tdd`
**Version**: 1.0.0
**Date**: 2026-05-03
**Author**: Hengshi (adapted from Hermes Agent TDD skill, MIT licensed)

---

## Firing Contract

### WHEN It Fires

**On every code change** — not cron-triggered, not manual. TDD fires automatically when:

| Trigger | Condition |
|---------|-----------|
| New feature | Any new function/method added to codebase |
| Bug fix | Any modification to fix a defect |
| Refactor | Any code restructuring (tests must pass before and after) |
| Behavior change | Any modification to existing behavior |

**How it fires**: The skill is invoked by the developer (human or agent) before writing any production code. The discipline is enforced by the developer's commitment to the Iron Law.

### WHAT Triggers It

```python
# Pattern: before writing ANY production code, write test first
# 1. Write minimal failing test
# 2. Run: pytest tests/...::test_name -v → FAIL
# 3. Write minimal production code
# 4. Run: pytest tests/...::test_name -v → PASS
# 5. Run: pytest tests/ -q → no regressions
```

### PRECONDITIONS

| Precondition | How Verified | Failure Mode |
|---|---|---|
| Test file exists or can be created | `Path(test_file).parent.mkdir(parents=True, exist_ok=True)` | Raises `TDDError` if directory not writable |
| Test framework available | `pytest --version` returns 0 | Raise `TDDError("pytest required")` |
| Test can be run in isolation | `pytest --collect-only` finds the test | Raise `TDDError` if test not found |
| Code under test is importable | `importlib.import_module(module_name)` | Normal Python import error — fix the code |

### POSTCONDITIONS

**State changes after one RED-GREEN-REFACTOR cycle:**

| State Change | Before | After |
|---|---|---|
| Test file | Unchanged or new | New test added (or existing modified) |
| Test result | Not run | Test passes (after GREEN phase) |
| Production code | Unchanged or new | New/fixed code that makes test pass |
| All tests | N tests passing | N+1 or N tests passing (no regressions) |
| Test log | N/A | RED phase: test FAIL. GREEN phase: test PASS |

**Observable outputs:**
1. `pytest` command output showing test results
2. Test file modified (new test or updated test)
3. Production code file modified
4. Exit code 0 from `pytest tests/ -q` (all pass)

**TDD enforcement checkpoints:**
| Checkpoint | Command | Expected |
|---|---|---|
| RED: test fails before code | `pytest tests/X.py::test_y -v` | Exit code ≠ 0 (FAIL) |
| GREEN: test passes after code | `pytest tests/X.py::test_y -v` | Exit code = 0 (PASS) |
| REFACTOR: no regressions | `pytest tests/ -q` | Exit code = 0 (all pass) |

### FAILURE MODES

| Failure | Detection | Retry Policy |
|---|---|---|
| Test passes on first run (before code) | RED phase shows PASS | **This is a failure** — you're testing existing code. Delete the test and write one that tests what SHOULD happen, not what does happen. Restart cycle. |
| Production code written before test | Direct observation | **This is a failure** — delete the production code. Restart with RED phase. |
| Test fails after GREEN phase | `pytest` exit code ≠ 0 | Fix the production code, not the test. Tests define requirements. |
| Other tests break during GREEN | `pytest tests/ -q` | Fix regressions before continuing. |
| Cannot import module under test | `ImportError` | Fix the import in production code. Test is correct. |
| Test directory not writable | `OSError` on write | Raise `TDDError`. Cannot proceed. |
| Rationalization detected | Any of the 24 excuses in the skill | **This is a failure** — restart TDD cycle. Rationalization = TDD skipped. |

### OBSERVABILITY

| Observable | Where | How |
|---|---|---|
| RED phase fires | Developer workflow | Test written, run, fails — terminal output shows FAIL |
| GREEN phase fires | Developer workflow | Code written, test run, passes — terminal output shows PASS |
| REFACTOR phase fires | Developer workflow | Code cleaned, all tests pass — `pytest tests/ -q` output |
| TDD violation detected | Developer workflow | Red flags section triggered — must restart |
| Final verification | `pytest tests/ -q` | Exit code 0 = all pass |
| Test count | `pytest --collect-only -q` | Count of tests in suite |

### ENFORCEMENT

**The Iron Law is not a guideline — it is a structural constraint.**

- Violation = restart required. No partial credit.
- Rationalization = violation. No "just this once."
- "Manual testing" ≠ verification. Automated tests required.

**Proof of enforcement:**
- All 3 phases must be visible in terminal output (RED FAIL → GREEN PASS → full suite PASS)
- Test count must increase by ≥1 for new features
- No test may pass in RED phase (that would mean code was written first)

---

## Evidence of Integration

**Skill**: `skills/tdd/SKILL.md` (adapted from Hermes Agent TDD, MIT licensed)
**Firing Contract**: `skills/tdd/FIRING_CONTRACT.md` (this file)

**Proof of enforcement — the discipline is the evidence:**
- RED phase must show a test that FAILS
- GREEN phase must show the SAME test PASSING
- REFACTOR phase must show ALL tests passing

Unlike session-summarization (which has code-level enforcement + unit tests), TDD is a **human-enforced discipline**. The firing contract's observability is the terminal output at each phase. If a developer skips phases, they violate the Iron Law — and the rationalization table in SKILL.md is their guide back.

**Test proof**: `skills/tdd/test_tdd_cycle.py` — proves the TDD skill itself follows TDD (the skill is tested by running a RED→GREEN cycle on a trivial example).

**Verification checklist:**
- [ ] SKILL.md exists with Iron Law + RED-GREEN-REFACTOR
- [ ] FIRING_CONTRACT.md has all 6 firing contract fields
- [ ] Rationalization table present (24 common excuses)
- [ ] Red Flags section present with restart instructions
- [ ] `test_tdd_cycle.py` runs a live RED→GREEN cycle and proves it
