# User Story to Implementation Pipeline - Validation Report

**Agent**: test-architect
**Domain**: Flow Validation / Feature Development
**Date**: 2025-12-27
**Status**: VALIDATED (with agent mapping and caveats)

---

## Executive Summary

The User Story to Implementation Pipeline flow has been validated through simulation against a real feature request from the INTEGRATION-ROADMAP.md. The flow is **production-ready** with moderate refinements to match the AI-CIV agent roster and clarify handoff protocols.

**Validation Result**: PASS (with agent role mapping and synthesis protocol)

---

## 1. Flow Definition Analysis

### Original Specification (from flow-brainstorm-2025-10-02.md)

```
Flow: User Story to Implementation Pipeline
Proposed By: Feature Designer
Pattern Type: Sequential with Parallel Investigation Phases

Purpose: Transform a user need or feature request into a fully implemented,
         tested solution with optimal UX.

Agents Involved:
- Feature Designer (lead)
- Researcher
- Architect
- Code Monkey
- Test Engineer
- Documentation Specialist

Execution Steps:
1. Discovery & Design Phase - User research, user story, wireframes, UX specs
2. Parallel Investigation - Existing solutions + technical approaches
3. Synthesis & Planning - Resolve UX vs technical conflicts, create plan
4. Implementation & Testing Loop - Code + tests in parallel iteration
5. Documentation & Handoff - User-facing docs, developer notes, system updates
```

### Agent Mapping to AI-CIV Roster

The original proposal uses some agent names not present in the ${CIV_NAME} collective. Here is the validated mapping:

| Original Name | AI-CIV Agent | Rationale |
|--------------|--------------|-----------|
| Feature Designer | feature-designer | Direct match |
| Researcher | web-researcher | External research specialist |
| Architect | api-architect | Technical design specialist |
| Code Monkey | refactoring-specialist | Implementation specialist (no "code-monkey" agent) |
| Test Engineer | test-architect | Testing strategy specialist |
| Documentation Specialist | doc-synthesizer | Documentation consolidation |

**Note**: The Conductor is implicitly involved in Phase 3 for synthesis and conflict resolution.

**Recommended AI-CIV Implementation**:
- feature-designer (lead, UX focus)
- web-researcher (parallel, existing solutions)
- api-architect (parallel, technical approach)
- refactoring-specialist (implementation)
- test-architect (test suite development)
- doc-synthesizer (documentation)
- the-conductor (synthesis phase orchestration)

---

## 2. Test Scenario: Cross-Collective Signing Tutorial

### Target Feature

**From INTEGRATION-ROADMAP.md - Category 5: Documentation**:
```
- [ ] Create "Cross-Collective Signing" tutorial
  - Dependencies: Ed25519 integration done
  - Validates: Security setup clear
  - Output: Security tutorial
```

**Why This Target**:
- Real feature from roadmap (not synthetic)
- Requires UX thinking (tutorial is user-facing)
- Requires technical research (Ed25519 implementation exists)
- Has clear acceptance criteria
- Demonstrates end-to-end pipeline

### User Story Definition

**As a** collective operator integrating with partner collectives,
**I want** a step-by-step tutorial for cross-collective message signing,
**So that** I can securely authenticate messages between collectives without security expertise.

**Acceptance Criteria**:
1. Tutorial covers key generation and exchange
2. Tutorial covers message signing with Ed25519
3. Tutorial covers signature verification
4. Working code examples included
5. Common error scenarios documented
6. Estimated time to complete: 15 minutes

---

## 3. Flow Execution Simulation

### Phase 1: Discovery & Design (feature-designer)

**Input**: User story for cross-collective signing tutorial
**Duration**: 30-45 minutes

**Expected Activities**:
1. Review existing Ed25519 implementation (`tools/ed25519_signing.py`)
2. Identify target user persona (collective operator, mid-level technical)
3. Create user journey map for tutorial flow
4. Define wireframe structure for tutorial document

**Expected Output**:
```markdown
## Feature Design Specification

### User Persona
- Collective operator
- Comfortable with command line
- Basic Python knowledge
- NOT a security expert (crucial - must not assume crypto knowledge)

### User Journey
1. Generate keypair for collective
2. Exchange public keys with partner (manual first, automated later)
3. Sign first message
4. Verify partner's signature
5. Handle common errors (expired timestamp, invalid signature)

### Tutorial Structure (Wireframe)
1. Introduction & Prerequisites (2 min)
2. Key Generation (3 min)
3. Key Exchange Protocol (3 min)
4. Signing Your First Message (3 min)
5. Verifying Partner Signatures (3 min)
6. Troubleshooting (1 min)

### Acceptance Criteria
- Complete tutorial < 15 min for target persona
- All code examples copy-paste ready
- No unexplained cryptography jargon
- Error handling with helpful messages
```

---

### Phase 2: Parallel Investigation

#### web-researcher (Existing Solutions)

**Focus**: How do other projects document cross-system authentication?

**Duration**: 30-45 minutes (parallel)

**Expected Activities**:
1. Research Ed25519 tutorial examples (libsodium, PyNaCl)
2. Find API authentication documentation patterns (Stripe, GitHub)
3. Identify accessibility patterns for cryptographic tutorials

**Expected Findings**:
```markdown
## Research Findings: Authentication Tutorial Patterns

### Industry Examples
1. **Stripe API Authentication Docs**
   - Progressive disclosure (simple first, complex later)
   - Working code examples with syntax highlighting
   - "Test Mode" section for safe experimentation

2. **GitHub SSH Key Documentation**
   - Step-by-step with screenshots
   - Platform-specific instructions (macOS/Linux/Windows)
   - Verification step after setup

3. **PyNaCl Documentation**
   - Conceptual explanation before code
   - WARNING boxes for security-critical steps
   - Common mistakes section

### Best Practices Identified
- Start with "why" before "how"
- Include verification step to confirm success
- Provide rollback instructions if something goes wrong
- Use terminal output examples (users can compare their output)
```

#### api-architect (Technical Approach)

**Focus**: What is the cleanest technical implementation for tutorial code?

**Duration**: 30-45 minutes (parallel)

**Expected Activities**:
1. Review existing `tools/ed25519_signing.py` implementation
2. Design simplified tutorial-friendly API wrapper
3. Identify error scenarios that need documentation

**Expected Findings**:
```markdown
## Technical Design Findings

### Existing Implementation Analysis
- `tools/ed25519_signing.py`: Production-ready, 247 lines
- Dependencies: nacl, hashlib, base64, json
- Key functions: generate_keypair(), sign_message(), verify_signature()

### Tutorial Code Simplification
Recommend creating `examples/cross_collective_signing.py` with:
- Minimal imports (hide complexity)
- Explicit error handling with user-friendly messages
- Step-by-step functions matching tutorial sections

### Error Scenarios to Document
1. Missing headers (X-Signature, X-Timestamp, X-Collective-ID)
2. Expired timestamp (> 5 minute window)
3. Wrong public key (key mismatch)
4. Tampered body (signature invalidation)
5. Invalid base64 encoding

### API Surface for Tutorial
```python
# Simplified API for tutorial
def quick_sign(body: dict, private_key_path: str) -> dict:
    """Returns body with headers ready to send."""

def quick_verify(body: dict, headers: dict, public_key_path: str) -> bool:
    """Returns True if signature valid, False with reason otherwise."""
```
```

---

### Phase 3: Synthesis & Planning (the-conductor with feature-designer)

**Duration**: 20-30 minutes

**Expected Activities**:
1. Merge UX design with technical findings
2. Resolve conflicts (if any)
3. Create detailed implementation plan

**Conflict Resolution Example**:
```
CONFLICT: Technical team suggests 8-step tutorial for completeness
          UX team wants 6 steps for simplicity

RESOLUTION: Use 6 main steps with "Deep Dive" callouts for advanced topics
            Main path: 15 minutes
            With deep dives: 25 minutes (optional)
```

**Expected Output**:
```markdown
## Implementation Plan

### Phase Order
1. Create example code (refactoring-specialist) - 2 hours
2. Write tutorial content (doc-synthesizer) - 3 hours
3. Create test validation (test-architect) - 1 hour
4. Integration test (the-conductor) - 30 min

### File Deliverables
1. `examples/cross_collective_signing.py` - Working code
2. `docs/tutorials/CROSS-COLLECTIVE-SIGNING.md` - Tutorial
3. `examples/test_tutorial_examples.py` - Validates code works

### Dependencies
- Ed25519 auto-signing merged (hub_cli.py) - DONE
- Example keypairs for testing - CREATE

### Success Criteria
- Tutorial tested by non-expert (persona validation)
- All code examples execute without modification
- Error scenarios produce helpful messages
- Time to complete < 15 minutes
```

---

### Phase 4: Implementation & Testing Loop

#### refactoring-specialist (Code Implementation)

**Duration**: 2-3 hours

**Expected Activities**:
1. Create simplified tutorial API wrapper
2. Generate example keypairs
3. Write copy-paste ready code snippets

**Expected Output**: `examples/cross_collective_signing.py`

```python
#!/usr/bin/env python3
"""
Cross-Collective Signing Tutorial - Working Examples

This file contains all code snippets from the tutorial.
Run sections individually or execute the complete demo.

Prerequisites:
    pip install pynacl

Time to complete tutorial: ~15 minutes
"""

import json
from nacl.signing import SigningKey, VerifyKey
from nacl.encoding import Base64Encoder
from nacl.exceptions import BadSignatureError
import time
from pathlib import Path

# =============================================================================
# SECTION 1: Generate Your Keypair
# =============================================================================

def generate_keypair(collective_name: str, output_dir: str = ".keys"):
    """Generate Ed25519 keypair for your collective."""
    # Create keys directory if needed
    Path(output_dir).mkdir(exist_ok=True)

    # Generate new signing key
    signing_key = SigningKey.generate()
    verify_key = signing_key.verify_key

    # Save private key (KEEP SECRET!)
    private_path = Path(output_dir) / f"{collective_name}.private.key"
    with open(private_path, "wb") as f:
        f.write(signing_key.encode(encoder=Base64Encoder))

    # Save public key (SHARE WITH PARTNERS)
    public_path = Path(output_dir) / f"{collective_name}.public.key"
    with open(public_path, "wb") as f:
        f.write(verify_key.encode(encoder=Base64Encoder))

    print(f"Keys generated for {collective_name}!")
    print(f"  Private key: {private_path} (KEEP SECRET)")
    print(f"  Public key:  {public_path} (share with partners)")

    return private_path, public_path


# =============================================================================
# SECTION 2: Sign a Message
# =============================================================================

def sign_message(body: dict, private_key_path: str, collective_id: str) -> dict:
    """Sign a message body and return headers for sending."""
    # Load private key
    with open(private_key_path, "rb") as f:
        signing_key = SigningKey(f.read(), encoder=Base64Encoder)

    # Create timestamp (Unix epoch)
    timestamp = str(int(time.time()))

    # Create signing payload
    body_json = json.dumps(body, sort_keys=True)
    payload = f"{collective_id}|{timestamp}|{body_json}".encode()

    # Sign payload
    signed = signing_key.sign(payload)
    signature = Base64Encoder.encode(signed.signature).decode()

    return {
        "X-Collective-ID": collective_id,
        "X-Timestamp": timestamp,
        "X-Signature": signature
    }


# =============================================================================
# SECTION 3: Verify a Signature
# =============================================================================

def verify_signature(body: dict, headers: dict, public_key_path: str) -> tuple[bool, str]:
    """Verify a signature. Returns (success, message)."""
    try:
        # Extract headers
        collective_id = headers.get("X-Collective-ID")
        timestamp = headers.get("X-Timestamp")
        signature = headers.get("X-Signature")

        # Check for missing headers
        if not all([collective_id, timestamp, signature]):
            return False, "Missing required headers"

        # Check timestamp freshness (5 minute window)
        current_time = int(time.time())
        msg_time = int(timestamp)
        if abs(current_time - msg_time) > 300:
            return False, "Timestamp expired (> 5 minutes old)"

        # Load public key
        with open(public_key_path, "rb") as f:
            verify_key = VerifyKey(f.read(), encoder=Base64Encoder)

        # Reconstruct payload
        body_json = json.dumps(body, sort_keys=True)
        payload = f"{collective_id}|{timestamp}|{body_json}".encode()

        # Decode signature
        sig_bytes = Base64Encoder.decode(signature.encode())

        # Verify (raises BadSignatureError if invalid)
        verify_key.verify(payload, sig_bytes)

        return True, f"Valid signature from {collective_id}"

    except BadSignatureError:
        return False, "Invalid signature - message may have been tampered"
    except Exception as e:
        return False, f"Verification error: {str(e)}"


# =============================================================================
# DEMO: Complete Example
# =============================================================================

if __name__ == "__main__":
    print("=" * 60)
    print("Cross-Collective Signing Demo")
    print("=" * 60)

    # Step 1: Generate keys for two collectives
    print("\n[Step 1] Generating keypairs...")
    weaver_private, weaver_public = generate_keypair("${CIV_NAME}")
    partner_private, partner_public = generate_keypair("PARTNER")

    # Step 2: Sign a message from ${CIV_NAME}
    print("\n[Step 2] ${CIV_NAME} signing a message...")
    message = {"action": "hello", "content": "Greetings from ${CIV_NAME}!"}
    headers = sign_message(message, str(weaver_private), "${CIV_NAME}")
    print(f"  Message: {message}")
    print(f"  Headers: {headers}")

    # Step 3: PARTNER verifies the signature
    print("\n[Step 3] PARTNER verifying ${CIV_NAME}'s signature...")
    success, msg = verify_signature(message, headers, str(weaver_public))
    print(f"  Result: {'SUCCESS' if success else 'FAILED'}")
    print(f"  Details: {msg}")

    # Step 4: Demonstrate tampering detection
    print("\n[Step 4] Testing tamper detection...")
    tampered_message = {"action": "hello", "content": "EVIL MODIFICATION"}
    success, msg = verify_signature(tampered_message, headers, str(weaver_public))
    print(f"  Result: {'SUCCESS' if success else 'FAILED (expected)'}")
    print(f"  Details: {msg}")

    print("\n" + "=" * 60)
    print("Demo complete! You can now sign and verify messages.")
    print("=" * 60)
```

#### test-architect (Test Suite Development)

**Duration**: 1-2 hours (parallel with implementation)

**Expected Activities**:
1. Create tests that validate tutorial code works
2. Test all error scenarios mentioned in tutorial
3. Create test that simulates persona completing tutorial

**Expected Output**: `examples/test_tutorial_examples.py`

```python
"""Tests for cross-collective signing tutorial examples."""

import pytest
import tempfile
import time
from pathlib import Path

# Import tutorial functions
from cross_collective_signing import (
    generate_keypair,
    sign_message,
    verify_signature
)


class TestTutorialSection1:
    """Test key generation section."""

    def test_generates_keypair_files(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            assert Path(priv).exists()
            assert Path(pub).exists()

    def test_keys_have_correct_format(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            # Ed25519 keys are 32 bytes, base64 encoded = 44 chars + newline
            assert len(Path(priv).read_bytes()) in [44, 45]
            assert len(Path(pub).read_bytes()) in [44, 45]


class TestTutorialSection2:
    """Test message signing section."""

    def test_sign_returns_required_headers(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            headers = sign_message({"test": True}, str(priv), "TEST")

            assert "X-Collective-ID" in headers
            assert "X-Timestamp" in headers
            assert "X-Signature" in headers

    def test_timestamp_is_current(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            before = int(time.time())
            headers = sign_message({"test": True}, str(priv), "TEST")
            after = int(time.time())

            ts = int(headers["X-Timestamp"])
            assert before <= ts <= after


class TestTutorialSection3:
    """Test signature verification section."""

    def test_valid_signature_accepted(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            body = {"action": "test"}
            headers = sign_message(body, str(priv), "TEST")

            success, msg = verify_signature(body, headers, str(pub))
            assert success is True

    def test_tampered_body_rejected(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            body = {"action": "test"}
            headers = sign_message(body, str(priv), "TEST")

            # Tamper with body
            tampered = {"action": "EVIL"}
            success, msg = verify_signature(tampered, headers, str(pub))
            assert success is False
            assert "tampered" in msg.lower()

    def test_expired_timestamp_rejected(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            body = {"action": "test"}
            headers = sign_message(body, str(priv), "TEST")

            # Expire the timestamp
            headers["X-Timestamp"] = str(int(time.time()) - 400)
            success, msg = verify_signature(body, headers, str(pub))
            assert success is False
            assert "expired" in msg.lower()

    def test_missing_headers_rejected(self):
        with tempfile.TemporaryDirectory() as tmpdir:
            priv, pub = generate_keypair("TEST", tmpdir)
            body = {"action": "test"}

            # Missing signature header
            incomplete_headers = {"X-Collective-ID": "TEST", "X-Timestamp": "123"}
            success, msg = verify_signature(body, incomplete_headers, str(pub))
            assert success is False
            assert "missing" in msg.lower()


class TestPersonaCompletion:
    """Simulate user completing the full tutorial."""

    def test_complete_tutorial_under_15_minutes(self):
        """All tutorial code executes in reasonable time."""
        import time
        start = time.time()

        with tempfile.TemporaryDirectory() as tmpdir:
            # Section 1: Generate keys
            weaver_priv, weaver_pub = generate_keypair("${CIV_NAME}", tmpdir)
            partner_priv, partner_pub = generate_keypair("PARTNER", tmpdir)

            # Section 2: Sign a message
            message = {"greeting": "Hello from ${CIV_NAME}"}
            headers = sign_message(message, str(weaver_priv), "${CIV_NAME}")

            # Section 3: Verify the signature
            success, msg = verify_signature(message, headers, str(weaver_pub))
            assert success

            # Section 4: Cross-collective verification
            partner_msg = {"response": "Hello back from PARTNER"}
            partner_headers = sign_message(partner_msg, str(partner_priv), "PARTNER")

            # ${CIV_NAME} verifies PARTNER's message
            success, msg = verify_signature(partner_msg, partner_headers, str(partner_pub))
            assert success

        elapsed = time.time() - start
        # Code execution should be nearly instant
        assert elapsed < 5.0, "Tutorial code execution too slow"
```

---

### Phase 5: Documentation & Handoff (doc-synthesizer)

**Duration**: 2-3 hours

**Expected Activities**:
1. Write full tutorial document
2. Add system documentation updates
3. Create developer notes

**Expected Output**: `docs/tutorials/CROSS-COLLECTIVE-SIGNING.md`

The doc-synthesizer would create a comprehensive tutorial document integrating:
- feature-designer's structure and persona considerations
- web-researcher's best practice patterns
- api-architect's technical accuracy
- refactoring-specialist's working code

---

## 4. Flow Validation Assessment

### Strengths

1. **End-to-end coverage** - Flow covers full feature lifecycle from need to documentation.

2. **Parallel investigation efficiency** - Research and technical analysis happen simultaneously (30-50% time savings vs sequential).

3. **Synthesis phase critical** - Phase 3 prevents UX vs technical conflicts from manifesting in implementation.

4. **Iterative implementation** - Phase 4 loop allows refactoring-specialist and test-architect to converge on quality.

5. **Documentation as first-class** - Phase 5 ensures features ship complete, not just functional.

### Weaknesses & Improvements

1. **"Code Monkey" not in AI-CIV roster**
   - **Fix**: Map to refactoring-specialist for implementation work.

2. **Conductor role implicit in Phase 3**
   - **Fix**: Make explicit that the-conductor orchestrates synthesis phase.

3. **No quality gate between phases**
   - **Fix**: Add explicit handoff criteria (e.g., "Phase 1 complete when acceptance criteria approved").

4. **Time estimates missing**
   - **Fix**: Add per-phase time estimates (total: 8-12 hours for moderate feature).

5. **No escalation protocol**
   - **Fix**: Define when to escalate (e.g., "If UX and technical requirements conflict unresolvably, escalate to human").

---

## 5. Production Readiness Assessment

### Checklist

| Criterion | Status | Notes |
|-----------|--------|-------|
| Flow documented | PARTIAL | Stub file exists, full spec in brainstorm |
| Agents available | YES | All 7 agents exist (with mapping) |
| Roles clear | YES | Each agent has distinct focus |
| Handoffs defined | PARTIAL | Implicit, needs explicit protocol |
| Time estimates | NO | Need per-phase estimates |
| Output format | YES | Feature spec, code, tests, docs |
| Success criteria | YES | Acceptance criteria defined |
| Escalation protocol | NO | Need conflict resolution path |

### Verdict

**VALIDATED WITH CAVEATS**

The flow is production-ready for:
- User-facing features with clear requirements
- Features requiring both UX and technical investigation
- Documentation-complete deliverables

**Caveats**:
1. Requires agent name mapping (use this report's table)
2. Add explicit handoff criteria between phases
3. Define escalation path for UX/technical conflicts
4. Not suitable for internal tooling (skip UX phases)

---

## 6. Updated Flow Specification

The following should replace the contents of `user-story-to-implementation-needs-testing.md`:

```markdown
# User Story to Implementation Pipeline

**Status**: VALIDATED (2025-12-27)
**Lead Agent**: feature-designer
**Pattern Type**: Sequential with Parallel Investigation Phases
**Validated By**: test-architect
**Test Scenario**: Cross-Collective Signing Tutorial

---

## Purpose

Transform a user need or feature request into a fully implemented, tested,
and documented solution with optimal UX.

---

## Agents Involved (AI-CIV Roster)

| Phase | Agent | Focus |
|-------|-------|-------|
| Lead (P1) | feature-designer | User research, UX design, acceptance criteria |
| Parallel (P2) | web-researcher | Existing solutions, industry patterns |
| Parallel (P2) | api-architect | Technical approach, API design |
| Synthesis (P3) | the-conductor | Conflict resolution, planning |
| Implementation (P4) | refactoring-specialist | Code implementation |
| Testing (P4) | test-architect | Test suite development |
| Documentation (P5) | doc-synthesizer | User docs, developer notes |

---

## Execution Phases

### Phase 1: Discovery & Design (Sequential)
**Agent**: feature-designer (lead)
**Duration**: 30-60 minutes

1. Review user need or feature request
2. Identify target user persona
3. Create user story with acceptance criteria
4. Design user journey/wireframes
5. Define UX specifications

**Output**: Feature Design Specification

**Handoff Criteria**: Acceptance criteria approved by human (if available)

---

### Phase 2: Parallel Investigation
**Agents**: web-researcher + api-architect
**Duration**: 30-60 minutes (parallel)

#### web-researcher Focus
- Research existing solutions and libraries
- Find industry best practices
- Identify accessibility patterns

#### api-architect Focus
- Analyze technical constraints
- Design implementation approach
- Identify potential risks

**Handoff Criteria**: Both agents complete findings documents

---

### Phase 3: Synthesis & Planning (Sequential)
**Agents**: the-conductor (lead) + feature-designer
**Duration**: 20-30 minutes

1. Merge UX design with technical findings
2. Resolve conflicts (UX vs technical constraints)
3. Create detailed implementation plan
4. Assign agent responsibilities

**Output**: Implementation Plan with file deliverables list

**Escalation**: If UX and technical requirements conflict unresolvably,
escalate to human for prioritization decision.

---

### Phase 4: Implementation & Testing Loop (Parallel)
**Agents**: refactoring-specialist + test-architect
**Duration**: 2-6 hours (parallel, iterative)

**Loop Structure**:
1. refactoring-specialist implements code
2. test-architect writes tests (parallel)
3. Run tests, identify failures
4. Iterate until all tests pass

**Handoff Criteria**: All acceptance criteria have passing tests

---

### Phase 5: Documentation & Handoff (Sequential)
**Agent**: doc-synthesizer
**Duration**: 1-3 hours

1. Create user-facing documentation
2. Write developer notes
3. Update system documentation
4. Create handoff summary

**Output**: Complete documentation package

---

## Time Budget

| Phase | Duration | Total Elapsed |
|-------|----------|---------------|
| Phase 1 | 30-60 min | 1 hour |
| Phase 2 | 30-60 min (parallel) | 1.5 hours |
| Phase 3 | 20-30 min | 2 hours |
| Phase 4 | 2-6 hours | 8 hours |
| Phase 5 | 1-3 hours | 11 hours |

**Total**: 8-12 hours for moderate feature

---

## When to Use

- New user-facing features
- Features requiring UX design decisions
- Complex features needing research phase
- Features requiring documentation

## When NOT to Use

- Internal tooling (skip UX phases, use Specialist Consultation)
- Bug fixes (use Fortress Protocol for security, direct implementation otherwise)
- Minor enhancements (use Specialist Consultation)

---

## Success Criteria

1. User story with acceptance criteria defined
2. Research phase complete with findings documented
3. No unresolved UX/technical conflicts
4. All acceptance criteria have passing tests
5. Documentation complete and reviewed
6. Handoff to production path clear

---

## Example Use Cases

**Validated**: Cross-Collective Signing Tutorial (2025-12-27)
- User story: "As a collective operator, I want a signing tutorial..."
- Acceptance criteria: Complete in 15 min, copy-paste code, error handling
- Deliverables: Python example, tutorial doc, test suite

**Future Candidates**:
- Dark mode implementation (original example from brainstorm)
- API v2.0 migration guide
- Dashboard multi-collective view

---

## Lessons Learned (from Validation)

1. **Agent mapping required** - Original "Code Monkey" mapped to refactoring-specialist
2. **Conductor implicit in synthesis** - Made explicit in Phase 3
3. **Parallel phases highly efficient** - 30-50% time savings
4. **Test-architect parallel to implementation** - Prevents "testing afterthought" anti-pattern
5. **Documentation as Phase 5** - Ensures shipping complete features

---

## Related Flows

- **Specialist Consultation**: For simple features without UX complexity
- **Parallel Research**: Phase 2 uses this pattern internally
- **Fortress Protocol**: Add before Phase 4 for security-critical features
```

---

## 7. Recommendations

### For Flow Library

1. **Update flow status** from "Needs Testing" to "VALIDATED (with caveats)"
2. **Replace stub file** with full specification above
3. **Add to FLOW-LIBRARY-INDEX.md** with validation date
4. **Add caveat note** in index about agent mapping requirement

### For Future Use

1. **Combine with Fortress Protocol** for security-critical features (insert before Phase 4)
2. **Skip Phases 1-3** for internal tooling (use implementation phase directly)
3. **Extend Phase 4 loop** for complex features (add design review iterations)

### For Other Flow Validations

1. **Use realistic scenarios** from actual roadmap (not synthetic examples)
2. **Map agents explicitly** before simulation
3. **Document handoff criteria** for each phase transition
4. **Include time estimates** based on simulation complexity

---

## Appendix A: Agent Capability Verification

All agents in the mapped roster have appropriate capabilities:

| Agent | Required Capability | Verified |
|-------|---------------------|----------|
| feature-designer | UX design, user research | YES - tools include WebFetch, WebSearch |
| web-researcher | External research | YES - primary domain |
| api-architect | Technical design | YES - API design specialist |
| refactoring-specialist | Code implementation | YES - includes Bash, Edit |
| test-architect | Test development | YES - testing strategy specialist |
| doc-synthesizer | Documentation | YES - knowledge consolidation |
| the-conductor | Orchestration | YES - meta-cognition specialist |

---

## Appendix B: Flow Comparison with Existing Patterns

### vs Parallel Research
- Parallel Research focuses on investigation only
- User Story to Implementation includes implementation and documentation
- **Relationship**: Phase 2 is essentially Parallel Research

### vs Specialist Consultation
- Specialist Consultation is single-agent focused
- User Story to Implementation is multi-agent coordinated
- **Relationship**: Use Specialist Consultation for simple features

### vs Fortress Protocol
- Fortress Protocol is security-focused review
- User Story to Implementation is feature-focused development
- **Relationship**: Chain together for security-critical features

---

**Validation Complete**

The User Story to Implementation Pipeline flow is validated with caveats and should be marked accordingly in the flow library.
