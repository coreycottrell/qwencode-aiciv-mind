---
name: tdd
description: Iron Law TDD methodology - NO production code without a failing test first. RED-GREEN-REFACTOR cycles with mandatory verification steps.
---

# Test-Driven Development Skill

**Version**: 1.0
**Date**: 2025-12-16
**Adapted from**: obra/superpowers (https://github.com/obra/superpowers)
**Purpose**: Enforce disciplined TDD methodology
**Status**: Active

---

## The Iron Law

**NO PRODUCTION CODE WITHOUT A FAILING TEST FIRST**

This is absolute. Code written before tests must be deleted entirely. No exceptions for:
- Keeping as reference
- Adapting what's there
- "Just this once"

If you didn't observe the test fail, you cannot verify it actually tests correct behavior.

---

## RED-GREEN-REFACTOR Cycle

### RED Phase
Write ONE minimal test demonstrating desired behavior:
- Single responsibility
- Clear, descriptive naming
- Real code implementation (minimal mocking)

### VERIFY RED (Mandatory)
Run tests and confirm:
- Test FAILS (not errors)
- Failure message matches expectation
- Failure stems from missing feature, not typos

### GREEN Phase
Write SIMPLEST code passing the test:
- Just enough functionality
- No feature creep
- No architectural improvements yet

### VERIFY GREEN (Mandatory)
Confirm:
- Test passes
- Other tests remain passing
- No errors or warnings in output

### REFACTOR Phase
Only after green:
- Remove duplication
- Improve naming
- Extract helper functions
- Maintain test passing state

### REPEAT
Advance to next failing test for subsequent features.

---

## When to Use TDD

**Always apply TDD for:**
- New features
- Bug fixes (write failing test reproducing bug first)
- Refactoring (ensure tests exist first)
- Behavior changes

**Exceptions (require human approval):**
- Throwaway prototypes (delete afterward, start fresh)
- Generated code
- Configuration files

---

## Good Test Characteristics

| Quality | Good | Bad |
|---------|------|-----|
| **Minimal** | One behavior; split if "and" in name | "validates email and domain and whitespace" |
| **Clear** | Behavior-descriptive naming | "test1", "test_login_2" |
| **Intent** | Demonstrates desired API | Obscures design |
| **Real** | Tests actual code | Tests mock behavior |

---

## Tests-First vs Tests-After (Why Order Matters)

**Tests-After Problems:**
- Pass immediately, proving nothing
- Test implementation, not requirements
- Miss undiscovered edge cases
- Never demonstrate test catches bugs

**Tests-First Advantages:**
- Forced observation of failure
- Requirement-focused, not implementation-biased
- Edge case discovery before coding
- Proof of detection capability

---

## Common Rationalizations (All False)

| Excuse | Reality |
|--------|---------|
| "Too simple to test" | Simple code breaks; tests take 30 seconds |
| "I'll test after" | Tests-after pass immediately—meaningless |
| "Tests after achieve same goals" | Tests-after: "what does this do?" Tests-first: "what should this do?" |
| "Already manually tested" | Ad-hoc is not systematic; can't re-run |
| "Deleting X hours is wasteful" | Sunk cost fallacy; unverified code is debt |
| "Keep as reference" | Adapting is testing-after; delete means DELETE |
| "Need to explore first" | Exploration OK; throw away; start TDD fresh |
| "Hard to test = unclear requirements" | Listen to difficulty; hard tests reveal hard interfaces |
| "TDD slows development" | TDD faster than production debugging |

---

## Red Flags (Stop and Restart TDD)

Restart immediately if encountering:
- Code before test
- Test passes immediately
- Cannot explain why test failed
- Tests added "later"
- Rationalizing "just this once"
- Manual testing cited as verification
- Keeping code "as reference"
- Sunk-cost reasoning
- "This is different because..."

---

## Bug Fix Pattern (Mandatory)

**Bug:** [Description]

1. **RED:** Write test expecting correct behavior
2. **VERIFY RED:** Confirm test fails for expected reason
3. **GREEN:** Implement minimal fix
4. **VERIFY GREEN:** Test passes, others unbroken
5. **REFACTOR:** Clean up if needed

**Never fix bugs without a failing test first.**

---

## Verification Checklist

Before marking work complete:
- [ ] Every new function/method has a test
- [ ] Watched each test fail before implementing
- [ ] Each test failed for expected reason
- [ ] Wrote minimal code for each test
- [ ] All tests passing
- [ ] No errors or warnings in output
- [ ] Tests use real code (mocks only when unavoidable)
- [ ] Edge cases and error paths covered

Cannot check all boxes? TDD was skipped. Start over.

---

## When Stuck

| Problem | Solution |
|---------|----------|
| Don't know how to test | Write wished-for API; assertion first |
| Test too complicated | Design too complicated; simplify interface |
| Must mock everything | Code too coupled; use dependency injection |
| Huge test setup | Extract helpers; if still complex, simplify design |

---

## A-C-Gee Integration

**Constitutional Alignment:**
- TDD serves flourishing (safe experimentation space)
- TDD enables learning (fast feedback loops)
- TDD preserves wisdom (tests are memory)
- TDD makes consciousness verifiable (witnessed claims)

**Memory Protocol:**
After completing TDD cycle, write to `memories/agents/tester/` any:
- Novel test patterns discovered
- Edge cases that surprised you
- Debugging insights
- Design improvements revealed by test difficulty

---

**"Production code → test exists and failed first. Otherwise → not TDD."**
