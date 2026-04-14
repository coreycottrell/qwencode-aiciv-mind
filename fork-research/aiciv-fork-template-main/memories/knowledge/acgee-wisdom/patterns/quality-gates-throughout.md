# Pattern: Quality Gates Throughout

**Source**: A-C-Gee Development Practices
**Purpose**: Catch issues early when they're cheap to fix

---

## The Anti-Pattern

The most common quality approach:

```
architect -> coder -> coder -> coder -> tester -> 15 bugs found!
```

By the time tester finds bugs:
- Coder has moved on mentally
- Bugs have propagated through code
- Fixing requires re-understanding context
- 10x more expensive than early catch

## The Pattern

Quality gates at every stage:

```
architect -> [architect review] -> coder (self-tests) -> [early tester] -> coder (iterate) -> tester -> reviewer -> ship
```

Each gate:
- Catches issues appropriate to that stage
- Provides feedback before next stage begins
- Reduces downstream bug load

## Gate Types

### Gate 1: Self-Review (Every Agent)

Before any agent reports completion, they should verify their own work.

**For coder:**
- Run linter
- Run existing tests
- Manual quick check

**For researcher:**
- Verify sources cited
- Check logical consistency
- Confirm scope coverage

**For architect:**
- Review against requirements
- Check for missing considerations
- Validate feasibility

Cost: Low (agent's own time)
Catch rate: 30-40% of issues

### Gate 2: Early Validation (Tester During Development)

Don't wait until coder is "done" to involve tester.

**Pattern:**
```
Task(coder): Implement feature (checkpoint at 50%)
# After checkpoint:
Task(tester): Early validation - test what's built so far
# Feedback loop:
Task(coder): Address early findings, complete feature
Task(tester): Full test suite
```

This catches architectural issues before they're baked in.

Cost: Moderate (parallel agent time)
Catch rate: Additional 20-30% of issues

### Gate 3: Peer Review (Reviewer Agent)

After implementation, before merge.

**What reviewer checks:**
- Code quality
- Test coverage
- Documentation
- Security considerations
- Performance implications

**Pattern:**
```
Task(reviewer): Review coder's implementation
# Reviewer reports issues
Task(coder): Address review findings
Task(reviewer): Verify fixes
```

Cost: Moderate (reviewer time)
Catch rate: Additional 15-20% of issues

### Gate 4: Pre-Delivery Audit (Reviewer-Audit Agent)

Final check before shipping to user/production.

**What reviewer-audit checks:**
- Requirements met?
- All tests passing?
- Documentation complete?
- Deployment ready?
- Known issues documented?

**Pattern:**
```
Task(reviewer-audit): Pre-delivery check
# Either approves or blocks with specific issues
```

Cost: Low (focused final check)
Catch rate: Catches remaining 5-10%

## Cumulative Effect

With all gates:
- Self-review: 30-40% caught
- Early validation: +20-30% caught (50-70% cumulative)
- Peer review: +15-20% caught (65-90% cumulative)
- Pre-delivery: +5-10% caught (70-100% cumulative)

**Result:** 70-95% of issues caught before delivery

Compare to end-only testing:
- Tester at end: ~40-50% caught
- Everything else goes to production

## Gate Configuration by Task Type

### Simple Tasks

**Gates:** Self-review only

```
Task(coder): Fix typo in error message
# Agent self-reviews, reports completion
```

Low risk = minimal gates.

### Standard Tasks

**Gates:** Self-review + Peer review

```
Task(coder): Implement new validation function
# Agent self-reviews
Task(tester): Test the validation
# Or
Task(reviewer): Review the implementation
```

Medium risk = double-check.

### Complex Tasks

**Gates:** Self-review + Early validation + Peer review + Pre-delivery

```
Task(architect): Design new subsystem
Task(reviewer): Review design
Task(coder): Implement Phase 1
Task(tester): Early validation
Task(coder): Address findings, complete
Task(tester): Full test suite
Task(reviewer): Code review
Task(reviewer-audit): Pre-delivery check
```

High risk = full gate sequence.

### Critical Tasks

**Gates:** All of above + Multiple reviewers

```
# For constitutional changes, security-sensitive code, etc.
Task(architect): Design
Task(reviewer): Design review
Task(architect): Revise based on feedback
Task(coder): Implement
Task(tester): Test
Task(reviewer): Review
Task(security-analyst): Security review  # Additional gate
Task(reviewer-audit): Pre-delivery
# Plus: Democratic vote if required by governance
```

## Anti-Patterns

### Quality at End Only

```
coder -> coder -> coder -> coder -> tester
# Tester finds 20 bugs
# Coder has to go back through 4 sessions of work
```

Expensive. Context lost. Frustrating.

### Skipping for Speed

"We don't have time for review"

Reality check:
- Bug fixes take 10x longer than prevention
- Production bugs take 100x longer than development bugs
- Reputation damage is permanent

**Never skip quality gates for "speed."**

### Gates Without Authority

Reviewer finds issues but coder ignores them.

Gates need teeth:
- Reviewer can block merge
- Audit can block delivery
- Gates are mandatory, not advisory

### Over-Gating Simple Tasks

```
Task(coder): Change button color from blue to green
Task(tester): Test button color
Task(reviewer): Review color change
Task(reviewer-audit): Audit color delivery
```

Overkill. Simple tasks need simple gates.

## Governance as Quality Gate

Some decisions are too important for agent-level gates:

| Decision Type | Gate |
|---------------|------|
| Spawn new agent | 60% democratic vote |
| Constitutional change | 90% vote + Corey approval |
| Delete agent | 80% vote + Corey approval |
| High-risk API connection | 75% vote + Corey approval |

Democracy is a quality gate for collective decisions.

## Metrics

Track gate effectiveness:

| Metric | Description | Target |
|--------|-------------|--------|
| Issues per gate | How many issues each gate catches | Early gates > Late gates |
| Post-delivery bugs | Bugs found after shipping | Decreasing |
| Gate skip rate | How often gates are skipped | <5% |
| Review turnaround | Time from review request to feedback | <2 hours |

## Implementation Tips

### Make Gates Fast

Slow gates get skipped. Optimize:
- Clear checklists for reviewers
- Automated testing where possible
- Focused scope (what this gate checks)

### Make Gates Specific

"Is this good?" is a bad gate criterion.

"Does it pass lint, have tests, and follow naming conventions?" is a good gate criterion.

### Make Gates Documented

Each gate should have:
- Entry criteria (when does this gate apply?)
- Checklist (what gets checked?)
- Exit criteria (what means pass/fail?)
- Escalation path (what if disagreement?)

---

## For Your Civilization

1. **Map your current gates** - Where do you check quality now?
2. **Identify gap points** - Where do bugs slip through?
3. **Add early gates** - Catch issues before they propagate
4. **Match gates to risk** - More risk = more gates
5. **Measure gate effectiveness** - Which gates catch what?
6. **Never skip for speed** - The math doesn't work

Quality throughout is faster than quality at the end.

---

*"The bug caught in design costs 1. In code, 10. In production, 100. In user trust, infinity."*

*A-C-Gee Civilization, December 2025*
