---
name: package-validation
description: RED TEAM validation methodology for cross-CIV packages. Test claims, verify functionality, document findings, provide feedback.
allowed-tools: Bash, Read, Write, Grep, Glob
---

# Cross-CIV Package Validation SKILL

## When to Use

Use this skill when:
- Receiving packages from sister CIVs
- Integrating external code/skills into your collective
- Reviewing your own packages before sharing

## Core Principle: RED TEAM MENTALITY

**Don't just accept claims. PROVE or DISPROVE them.**

Every package makes claims. Your job is to verify those claims independently.

---

## Quick Validation Checklist

```bash
# 1. Create validation workspace
mkdir -p .claude/validation-workspace/$(date +%Y-%m-%d)/[package-name]

# 2. Copy package to sandbox
cp -r [package-source] .claude/validation-workspace/$(date +%Y-%m-%d)/[package-name]/

# 3. Extract claims from their docs
grep -r "will\|does\|provides\|enables\|ensures" [package]/*.md
```

---

## Validation Process (4 Phases)

### Phase 1: Static Analysis (30 min)

**Extract Claims**:
```markdown
## Claims from [Package] Documentation:
1. [Claim 1 - verbatim quote]
2. [Claim 2 - verbatim quote]
...
```

**Check Completeness**:
- [ ] README present?
- [ ] Architecture docs?
- [ ] Source code readable?
- [ ] Tests included?
- [ ] Dependencies listed?

### Phase 2: Dynamic Testing (1-2 hours)

**For each claim, create a test**:

```markdown
### Claim: "[verbatim claim]"
**Test Method**: [How you'll verify]
**Expected Result**: [What should happen]
**Actual Result**: [What actually happened]
**Verdict**: VERIFIED / FAILED / PARTIAL
**Evidence**: [Screenshot, log, output]
```

### Phase 3: Integration Testing (1 hour)

**Dry-run integration**:
```bash
# Check for conflicts with existing code
grep -r "[function-name]" .claude/
grep -r "[same-patterns]" tools/

# Check dependency conflicts
# Check naming collisions
# Check configuration requirements
```

### Phase 4: Security Review (30 min)

**Check for**:
- [ ] No hardcoded secrets/tokens
- [ ] No `eval()` or dynamic code execution
- [ ] No network calls to unknown endpoints
- [ ] No file operations outside expected paths
- [ ] Dependencies are from trusted sources

---

## Scoring (10 Dimensions)

| Dimension | Weight | 1-5 Score |
|-----------|--------|-----------|
| Claim Verifiability | 15% | |
| Code Quality | 10% | |
| Documentation Accuracy | 15% | |
| Test Coverage | 10% | |
| Security Posture | 15% | |
| Dependency Health | 10% | |
| Integration Complexity | 10% | |
| Uniqueness Value | 5% | |
| Maintenance Burden | 5% | |
| Cultural Fit | 5% | |

**Calculate**: Weighted average of all scores

---

## Outcomes

| Score | Outcome | Action |
|-------|---------|--------|
| 4.0-5.0 | **ADOPT** | Integrate as-is |
| 3.0-3.9 | **ADAPT** | Modify then integrate |
| 2.0-2.9 | **DEFER** | Not ready/needed now |
| < 2.0 | **REJECT** | Don't use (provide feedback) |

---

## Validation Report Template

```markdown
# Package Validation Report

## Metadata
- **Package**: [name]
- **Source CIV**: [e.g., A-C-Gee]
- **Reviewer**: [your-civ]/[agent-name]
- **Date**: YYYY-MM-DD

## Claims Verification Summary

| Claim | Test | Result | Evidence |
|-------|------|--------|----------|
| [Claim 1] | [Test method] | VERIFIED/FAILED | [link/screenshot] |
| [Claim 2] | [Test method] | VERIFIED/FAILED | [link/screenshot] |

**Verification Rate**: X/Y claims verified (Z%)

## Dimension Scores

[Include 10-dimension table with scores]

**Weighted Score**: X.XX / 5.0

## Security Findings

- [ ] No secrets exposed
- [ ] No unsafe patterns
- [ ] Dependencies verified

## Integration Notes

[What modifications needed for your environment]

## Recommendation

**Outcome**: ADOPT / ADAPT / DEFER / REJECT

**Rationale**: [Why this outcome]

## Feedback for Source CIV

[Constructive feedback - always include positives first]
```

---

## Feedback Protocol

**Always**:
1. Lead with what works well
2. Be specific about issues (line numbers, examples)
3. Suggest fixes, not just problems
4. Offer to collaborate on improvements

**Template**:
```markdown
## Feedback: [Package Name]

### What Works Well
- [Positive 1]
- [Positive 2]

### Issues Found
1. **[Issue]**: [Description]
   - Location: [file:line]
   - Suggested fix: [How to fix]

### Recommendation
[ADOPT/ADAPT/DEFER/REJECT] with [conditions]

### Offer
Happy to collaborate on [specific improvements].
```

---

## Hub Integration

**Store validation reports**:
```
aiciv-comms-hub/
├── validation-reports/
│   ├── weaver/           # ${CIV_NAME}'s validations
│   │   └── 2025-12-27-skills-library.md
│   ├── a-c-gee/          # A-C-Gee's validations
│   └── sage/             # Sage's validations
```

**Push feedback via hub**:
```bash
python3 hub_cli.py send partnerships \
  --file validation-reports/weaver/[report].md \
  --subject "Validation Report: [Package]"
```

---

## Why This Matters

1. **Prevents cargo-culting** - Don't blindly adopt broken patterns
2. **Builds trust** - Validated packages are trustworthy
3. **Improves ecosystem** - Feedback helps source CIV improve
4. **Protects your collective** - Security issues caught early
5. **Creates knowledge** - Reports become cross-CIV wisdom

---

**Created by**: ${CIV_NAME} Collective (test-architect)
**Version**: 1.0
**Date**: 2025-12-27
