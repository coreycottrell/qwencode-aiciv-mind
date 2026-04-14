---
name: security-analysis
description: Static security analysis of code for A-C-Gee and sister civilizations. Use when reviewing code for security vulnerabilities - OWASP Top 10, Solana/Anchor patterns, dependency analysis.
---

# Security Analysis Skill

**Version**: 1.0
**Date**: 2025-12-18
**Status**: Production-ready

**Purpose**: Static security analysis of code for A-C-Gee and sister civilizations

**Invocation**: Use when reviewing code for security vulnerabilities

---

## Capabilities

### What This Skill Does
- Static code analysis for security patterns
- OWASP Top 10 vulnerability detection
- Solana/Anchor specific security checks
- Python/JavaScript/TypeScript security review
- Dependency vulnerability assessment
- Attack surface mapping

### What This Skill Does NOT Do
- Active testing against external systems
- Sending requests to any endpoints
- Penetration testing
- Bug bounty hunting on third-party systems

---

## Initial Reconnaissance Steps

When analyzing a codebase for security:

1. **Check for exposed secrets**
   - `.env.example` files (often contain hardcoded secrets)
   - Configuration files with credentials
   - API keys in source code

2. **Read security documentation**
   - `BugBounty.md` or `SECURITY.md` for scope
   - Previous security findings if documented

3. **Map directory structure**
   - Understand architecture
   - Identify entry points (APIs, webhooks, forms)

4. **Identify technology stack**
   - Check `requirements.txt`, `package.json`, `Cargo.toml`
   - Note framework versions for known CVEs

---

## Analysis Checklist

### General Web Security (OWASP Top 10)

- [ ] **Injection** - SQL/NoSQL injection patterns
- [ ] **Broken Authentication** - Session management, credential storage
- [ ] **Sensitive Data Exposure** - Unencrypted data, verbose errors
- [ ] **XML External Entities (XXE)** - XML parser configuration
- [ ] **Broken Access Control** - IDOR, privilege escalation
- [ ] **Security Misconfiguration** - Debug mode, default credentials
- [ ] **XSS (Cross-Site Scripting)** - User input reflection
- [ ] **Insecure Deserialization** - pickle, yaml.load(), JSON.parse of user data
- [ ] **Known Vulnerabilities** - Outdated dependencies with CVEs
- [ ] **Insufficient Logging** - Missing audit trails

### Solana/Anchor Specific

- [ ] **Account ownership verification** - Missing owner checks
- [ ] **Signer validation** - Unsigned accounts used as authority
- [ ] **PDA derivation correctness** - Bump seed handling
- [ ] **Integer overflow/underflow** - Arithmetic without checked_*
- [ ] **Reentrancy in CPI calls** - Cross-program invocation risks
- [ ] **Missing close constraints** - Account draining possible
- [ ] **Discriminator checks** - Account type confusion
- [ ] **Lamport balance manipulation** - Rent exemption attacks

### Dependency Analysis

- [ ] Known CVEs in dependencies
- [ ] Outdated packages (check against latest)
- [ ] Unpinned versions (supply chain risk)
- [ ] Typosquatting risk (package name verification)

---

## High-Value Search Patterns

### Command Injection / Code Execution

```bash
# Python
grep -rE "eval\(|exec\(|subprocess|shell=True|os\.system" .

# JavaScript/Node
grep -rE "eval\(|Function\(|child_process|exec\(|spawn\(" .
```

### XSS Vectors

```bash
# React
grep -rE "dangerouslySetInnerHTML|innerHTML" .

# General HTML
grep -rE "\.html\(|\.append\(|document\.write" .
```

### SQL Injection

```bash
# String interpolation in queries
grep -rE "SELECT.*\+|INSERT.*\+|UPDATE.*\+|DELETE.*\+" .
grep -rE "f\".*SELECT|f\".*INSERT|f\".*UPDATE" .
```

### Path Traversal

```bash
grep -rE "\.\.\/|\.\.\\\\|os\.path\.join.*\.\." .
```

### Secrets/Credentials

```bash
grep -rE "password|secret|token|api_key|apikey|private_key" . --include="*.py" --include="*.js" --include="*.ts"
```

### HTTP/API Calls (Potential SSRF)

```bash
grep -rE "fetch\(|axios\.|requests\.|urllib|XMLHttpRequest|http\.get" .
```

---

## Framework-Specific Checks

### Django

| Check | Pattern | Severity |
|-------|---------|----------|
| Debug mode in prod | `DEBUG = True` | Critical |
| CSRF disabled | `@csrf_exempt` | High |
| Open CORS | `CORS_ORIGIN_ALLOW_ALL = True` | Medium |
| Hardcoded secret | `SECRET_KEY = "..."` in settings | Critical |
| SQL injection | `raw()`, `extra()` with string interpolation | Critical |

### React/TypeScript

| Check | Pattern | Severity |
|-------|---------|----------|
| XSS via innerHTML | `dangerouslySetInnerHTML` | High |
| Token in localStorage | `localStorage.setItem("token"...)` | Medium |
| Verbose error display | Error messages with stack traces | Low |
| Unvalidated redirects | `window.location = userInput` | Medium |

### Node.js/Express/Fastify

| Check | Pattern | Severity |
|-------|---------|----------|
| Shell execution | `child_process` with user input | Critical |
| Path traversal | `fs.readFile(userInput)` | High |
| Open CORS | `cors({ origin: '*' })` | Medium |
| No rate limiting | Missing express-rate-limit | Medium |
| Prototype pollution | Object merge without sanitization | High |

### Anchor/Solana

| Check | Pattern | Severity |
|-------|---------|----------|
| Missing owner check | No `constraint = account.owner == expected` | Critical |
| Unsigned authority | `Signer` not in account constraint | Critical |
| Unsafe arithmetic | Using `+`, `-`, `*` instead of `checked_*` | High |
| PDA bump not stored | Regenerating bump on each call | Medium |
| Missing discriminator | `init` without `#[account]` derive | High |

---

## Usage Pattern

When invoked, the skill will:

1. **Identify language/framework** of target code
2. **Apply relevant security checklist** (see above)
3. **Search for dangerous patterns** using grep patterns
4. **Document findings** with severity ratings (Critical/High/Medium/Low)
5. **Provide remediation guidance** for each finding

### Output Format

```markdown
## Security Analysis Report

### Target: [codebase/path]
### Date: [analysis date]
### Analyst: [agent name]

---

### Summary
- Critical: X
- High: X
- Medium: X
- Low: X

---

### Findings

#### [SEV-001] Critical: SQL Injection in user_service.py
**Location**: `src/services/user_service.py:45`
**Pattern**: String interpolation in raw SQL query
**Evidence**:
```python
cursor.execute(f"SELECT * FROM users WHERE id = {user_id}")
```
**Remediation**: Use parameterized queries
```python
cursor.execute("SELECT * FROM users WHERE id = %s", (user_id,))
```

[... additional findings ...]

---

### Recommendations
1. [Priority-ordered remediation steps]
```

---

## Safety Boundaries

**CRITICAL**: This skill is for STATIC ANALYSIS ONLY.

| Action | Allowed |
|--------|---------|
| Read code | YES |
| Grep for patterns | YES |
| Run local linters | YES |
| Analyze dependencies | YES |
| Send HTTP requests | NO |
| Test external systems | NO |
| Active exploitation | NO |
| Modify production systems | NO |

---

## Resources

### Internal Learning Materials

| Resource | Path | Purpose |
|----------|------|---------|
| Test scripts | `tools/security-tests/` | Example security testing code |
| Methodology docs | `memories/knowledge/aixblock-testing-methodology.md` | Full methodology from AIxBlock analysis |
| Codebase analysis | `memories/knowledge/aixblock-codebase-analysis.md` | Example analysis output |
| Audit patterns | `.claude/memory/agent-learnings/coder/20251218-security-audit-patterns.md` | Learned patterns |

### External References

- OWASP Top 10: https://owasp.org/www-project-top-ten/
- Solana Security: https://github.com/coral-xyz/sealevel-attacks
- Node.js Security: https://nodejs.org/en/docs/guides/security/

---

## Integration with Agents

This skill can be used by:

- **coder**: Pre-commit security review of own code
- **reviewer**: Security-focused code review
- **tester**: Security test planning
- **researcher**: Vulnerability research patterns

### Delegation Example

```
Task(coder):
  Skill: security-analysis
  Target: arcx-marketplace-v2/src/
  Focus: Solana/Anchor patterns
  Output: Security report with findings
```

---

## Quick Reference Card

### Start Every Analysis With

```bash
# 1. Find exposed secrets
grep -rE "(password|secret|api_key|token).*=" . --include="*.env*" --include="*.json" --include="*.yaml"

# 2. Find dangerous functions
grep -rE "eval\(|exec\(|dangerouslySetInnerHTML|shell=True|raw\(" .

# 3. Find user input handling
grep -rE "request\.(params|query|body)|req\.(params|query|body)|user_input" .

# 4. Check dependency versions
cat package.json 2>/dev/null || cat requirements.txt 2>/dev/null || cat Cargo.toml 2>/dev/null
```

### Severity Guide

| Severity | Impact | Example |
|----------|--------|---------|
| Critical | RCE, full compromise | Command injection, deserialization |
| High | Data breach, privilege escalation | SQL injection, broken auth |
| Medium | Limited data exposure | XSS, CSRF, IDOR |
| Low | Information disclosure | Verbose errors, version disclosure |

---

## Session Checklist

Starting a security analysis:

- [ ] Identify target scope (files, directories, components)
- [ ] Determine technology stack
- [ ] Run initial reconnaissance patterns
- [ ] Apply framework-specific checks
- [ ] Document all findings with evidence
- [ ] Prioritize by severity
- [ ] Provide remediation guidance
- [ ] Write memory entry for learnings

---

*This skill is for INTERNAL static analysis only. Never test external systems without explicit authorization.*
