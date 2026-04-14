# Security Assessment Template

---

## Security Assessment: [System/Application Name]

**Assessor:** Security Engineer
**Date:** [YYYY-MM-DD]
**Version:** 1.0
**Classification:** Internal / Confidential

---

## 1. Executive Summary

### 1.1 Assessment Overview
[Brief description of what was assessed and why]

### 1.2 Risk Summary

| Severity | Count | Status |
|----------|-------|--------|
| 🔴 Critical | 0 | |
| 🟠 High | 0 | |
| 🟡 Medium | 0 | |
| 🟢 Low | 0 | |
| ℹ️ Informational | 0 | |

### 1.3 Overall Risk Rating
- [ ] 🔴 **Critical** - Immediate action required
- [ ] 🟠 **High** - Address within 7 days
- [ ] 🟡 **Medium** - Address within 30 days
- [ ] 🟢 **Low** - Address in next release cycle

---

## 2. Scope

### 2.1 In Scope
- [Application/URL 1]
- [Application/URL 2]
- [Infrastructure component]

### 2.2 Out of Scope
- [What was excluded and why]

### 2.3 Assessment Type
- [ ] Penetration Test (Black Box)
- [ ] Penetration Test (Gray Box)
- [ ] Penetration Test (White Box)
- [ ] Vulnerability Scan
- [ ] Code Review
- [ ] Configuration Review
- [ ] Architecture Review

---

## 3. Methodology

### 3.1 Standards Referenced
- OWASP Top 10 (2021)
- OWASP ASVS v4.0
- CWE Top 25
- [Industry-specific: PCI-DSS, HIPAA, etc.]

### 3.2 Tools Used
| Tool | Purpose |
|------|---------|
| Burp Suite | Web app testing |
| Nmap | Network scanning |
| OWASP ZAP | Automated scanning |
| [Tool] | [Purpose] |

### 3.3 Testing Approach
[Describe methodology - reconnaissance, scanning, exploitation, reporting]

---

## 4. Findings

### 4.1 Critical Findings

#### FINDING-001: [Vulnerability Title]

| Attribute | Value |
|-----------|-------|
| **Severity** | 🔴 Critical |
| **CVSS Score** | 9.8 |
| **CWE** | CWE-89: SQL Injection |
| **OWASP** | A03:2021 – Injection |
| **Status** | Open / Remediated / Accepted |

**Location:**
```
URL: https://app.example.com/api/users
Parameter: id
Method: GET
```

**Description:**
[Detailed description of the vulnerability]

**Evidence:**
```
[Proof of concept - sanitized]
Request:
GET /api/users?id=1' OR '1'='1

Response:
[Evidence of exploitation]
```

**Impact:**
[What an attacker could achieve - data breach, unauthorized access, etc.]

**Remediation:**
1. [Step 1 to fix]
2. [Step 2 to fix]
3. [Step 3 to fix]

**References:**
- [CVE if applicable]
- [OWASP reference]
- [Vendor documentation]

---

### 4.2 High Findings

#### FINDING-002: [Vulnerability Title]

[Same structure as above]

---

### 4.3 Medium Findings

| ID | Title | Location | Status |
|----|-------|----------|--------|
| FINDING-003 | [Title] | [Location] | Open |
| FINDING-004 | [Title] | [Location] | Open |

[Brief descriptions for each]

---

### 4.4 Low / Informational Findings

| ID | Title | Recommendation |
|----|-------|----------------|
| FINDING-005 | [Title] | [Brief fix] |
| FINDING-006 | [Title] | [Brief fix] |

---

## 5. Positive Observations

[Security controls that ARE working well]

- ✅ [Positive finding 1]
- ✅ [Positive finding 2]
- ✅ [Positive finding 3]

---

## 6. Recommendations Summary

### 6.1 Immediate Actions (Critical/High)
1. [ ] [Action 1]
2. [ ] [Action 2]

### 6.2 Short-Term Actions (Medium)
1. [ ] [Action 1]
2. [ ] [Action 2]

### 6.3 Long-Term Improvements
1. [ ] [Improvement 1]
2. [ ] [Improvement 2]

---

## 7. Remediation Verification

### 7.1 Verification Process
[How fixes will be verified]

### 7.2 Retest Schedule
| Finding | Remediation Date | Retest Date | Status |
|---------|------------------|-------------|--------|
| FINDING-001 | | | Pending |
| FINDING-002 | | | Pending |

---

## 8. Appendix

### 8.1 CVSS Scoring Reference

| Score | Severity |
|-------|----------|
| 9.0-10.0 | Critical |
| 7.0-8.9 | High |
| 4.0-6.9 | Medium |
| 0.1-3.9 | Low |

### 8.2 Raw Scan Results
[Attach or link to detailed scan outputs]

### 8.3 Screenshots
[Attach evidence screenshots]

---

## 9. Sign-Off

| Role | Name | Date |
|------|------|------|
| Security Engineer | | |
| CTO | | |
| Stakeholder | | |

---

**Confidentiality Notice:** This document contains sensitive security information. Distribution should be limited to authorized personnel only.
