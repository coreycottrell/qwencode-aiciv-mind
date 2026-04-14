---
name: fortress-protocol
description: Security-first code review with CVSS scoring, threat surface mapping, and multi-agent vulnerability analysis
version: 1.0.0
source: AI-CIV/${CIV_NAME} (flow-brainstorm-2025-10-02.md)
allowed-tools: [Task, Read, Write, Grep, Glob, Bash, WebFetch]
agents-required: [security-auditor, api-architect, performance-optimizer, test-architect, doc-synthesizer]
status: AWAITING-VALIDATION
---

# Fortress Protocol Skill

A comprehensive security auditing flow that catches vulnerabilities traditional testing misses by combining multiple specialist perspectives with CVSS-like severity scoring.

## When to Use

**Invoke when**:
- Before merging PRs that add authentication, payment processing, or sensitive data handling
- Auditing dependencies for CVE vulnerabilities
- Security-critical configuration changes
- Post-incident security review
- Preparing for external security audit

**Do not use when**:
- Simple code changes with no security surface
- Urgent production hotfix (too slow for emergencies)
- Already validated by external security team

## Prerequisites

**Agents Required**:
- **security-auditor** (Lead) - Orchestrates analysis, assigns CVSS scores, creates remediation roadmap
- **api-architect** - Analyzes architectural security patterns and design vulnerabilities
- **performance-optimizer** - Identifies performance-based attack vectors (DoS, timing attacks)
- **test-architect** - Reviews security test coverage and missing edge cases
- **doc-synthesizer** - Audits security docs, checks CVE databases, validates secure defaults

**Context Needed**:
- Access to code under review
- Existing security policies or requirements
- List of sensitive operations (auth, payments, PII)

## Procedure

### Step 1: Initial Threat Surface Mapping
**Duration**: ~15 minutes
**Agent(s)**: security-auditor

Analyze changed code for attack surfaces:

1. Identify entry points (APIs, user inputs, file uploads)
2. Map data flow through sensitive operations
3. List authentication/authorization boundaries
4. Note third-party dependencies and their versions
5. Create initial threat surface diagram

**Deliverable**: Threat surface map with attack vectors annotated

---

### Step 2: Parallel Deep Dives
**Duration**: ~30-45 minutes
**Agent(s)**: api-architect, performance-optimizer, test-architect (parallel)

Specialists investigate simultaneously:

**api-architect**:
- Architectural security patterns
- Design-level vulnerabilities
- Trust boundary violations
- Data exposure risks

**performance-optimizer**:
- DoS vulnerability points
- Timing attack vectors
- Resource exhaustion possibilities
- Rate limiting gaps

**test-architect**:
- Security test coverage gaps
- Missing edge cases
- Injection test scenarios
- Fuzzing recommendations

**Deliverable**: Three specialized security perspectives

---

### Step 3: Documentation & Dependency Audit
**Duration**: ~20 minutes
**Agent(s)**: doc-synthesizer

Review security documentation and dependencies:

1. Audit security docs for accuracy
2. Check CVE databases for known vulnerabilities
3. Validate secure defaults in configurations
4. Review error handling for information leakage
5. Check logging for sensitive data exposure

**Deliverable**: Dependency and documentation audit report

---

### Step 4: Synthesis & Risk Ranking
**Duration**: ~15 minutes
**Agent(s)**: security-auditor

Synthesize all findings with CVSS-like scoring:

1. Consolidate findings from all specialists
2. Assign severity scores (Critical/High/Medium/Low)
3. Calculate exploitability and impact
4. Rank by blast radius and business risk
5. Identify any blocking issues

**Deliverable**: Risk-ranked vulnerability report

---

### Step 5: Remediation Roadmap
**Duration**: ~15 minutes
**Agent(s)**: security-auditor + team

Produce actionable recommendations:

1. Create specific fix recommendations with code examples
2. Generate test cases to verify fixes
3. Assign ownership and priority
4. Estimate remediation effort
5. Define verification criteria

**Deliverable**: Prioritized remediation roadmap with code samples

---

## Parallelization

**Can run in parallel**:
- Step 2: All deep dives execute simultaneously
- Step 3 can overlap with Step 2

**Must be sequential**:
- Step 1 before Step 2 (threat surface guides deep dives)
- Step 2+3 before Step 4 (synthesis needs all findings)
- Step 4 before Step 5 (prioritization guides roadmap)

## Success Indicators

- [ ] Threat surface map complete with attack vectors
- [ ] All OWASP Top 10 categories checked
- [ ] CVSS-like severity assigned to each finding
- [ ] No Critical/High issues left unaddressed
- [ ] Remediation roadmap has specific code examples
- [ ] Test cases generated for each vulnerability
- [ ] Dependencies checked against CVE databases

## Example

**Scenario**: Trading Arena API security review before production

```
Step 1 (Map): 4 API endpoints identified as attack surface
         Ed25519 auth boundary mapped
         WebSocket upgrade path flagged

Step 2 (Deep Dive):
  api-architect: Auth token handling needs timing-safe compare
  performance-optimizer: No rate limiting on signature verification
  test-architect: Missing fuzz tests for malformed signatures

Step 3 (Docs): 2 dependencies have known CVEs (need upgrade)
         Config allows debug mode in production

Step 4 (Score):
  HIGH: Timing attack on signature verification (CVSS 7.5)
  MEDIUM: Missing rate limiting (CVSS 5.3)
  LOW: Debug mode default (CVSS 3.7)

Step 5 (Roadmap):
  P0: Use constant-time compare for signatures
  P1: Add rate limiting middleware (60 req/min)
  P2: Default debug=false in config

Result: 3 vulnerabilities caught before production
```

## Notes

- **Typical Duration**: 90-120 minutes for comprehensive review
- **Error Handling**: If specialists disagree on severity, default to higher rating
- **CVSS Reference**: Use CVSS 3.1 base metrics for consistency
- **Key Insight**: Security vulnerabilities are expensive in production - multi-perspective review catches issues single reviewers miss
- **Trading Arena Integration**: Ideal for validating Ed25519 authentication flows

---

**Source**: flow-brainstorm-2025-10-02.md (Section 5: Fortress Protocol)
**Status**: AWAITING-VALIDATION
**Conversion Date**: 2025-12-27
